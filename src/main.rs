use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use zenoh::{open, Config};

#[derive(Parser, Debug)]
#[command(name = "omsn")]
#[command(author = "")]
#[command(version = "0.1.0")]
#[command(about = "Proxies data between Zenoh and UDP Multicast")]
struct Args {
    #[arg(long, help = "Simulation ID for grouping omsn clients")]
    simulation_id: String,
    #[arg(long, help = "Site ID for Zenoh key space")]
    site_id: String,
    #[arg(long, help = "Application ID for Zenoh key space")]
    application_id: String,
    #[arg(long, help = "Multicast group IPv4 address")]
    group: Ipv4Addr,
    #[arg(long, help = "Multicast port")]
    port: u16,
    #[arg(
        long,
        default_value = "0.0.0.0",
        help = "Network interface IPv4 address"
    )]
    interface: Ipv4Addr,
    #[arg(long, help = "Output stats every 10 seconds")]
    stats: bool,
    #[arg(long, help = "Output all payloads proxied")]
    verbose: bool,
    #[arg(long, help = "Path to zenoh configuration file (optional)")]
    zenoh_config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Starting omsn");
    println!(
        "SITE_ID: {} | APPLICATION_ID: {}",
        args.site_id, args.application_id
    );
    println!(
        "Multicast group: {} | Port: {} | Interface: {}",
        args.group, args.port, args.interface
    );
    if args.stats {
        println!("Stats enabled");
    }
    if args.verbose {
        println!("Verbose enabled");
    }

    let zenoh_pub_key = format!(
        "omsn/@v1/{}/{}/{}",
        args.simulation_id, args.site_id, args.application_id
    );
    let zenoh_sub_key = format!("omsn/@v1/{}/**", args.simulation_id);
    println!("Zenoh publish key: {}", zenoh_pub_key);
    println!("Zenoh subscribe key: {}", zenoh_sub_key);

    // Stats counters
    let udp_to_zenoh_counter = Arc::new(AtomicUsize::new(0));
    let zenoh_to_udp_counter = Arc::new(AtomicUsize::new(0));
    let per_sender_stats = Arc::new(std::sync::Mutex::new(
        HashMap::<(String, String), usize>::new(),
    ));

    // Zenoh session
    let z_config = if let Some(ref path) = args.zenoh_config {
        Config::from_file(path).expect("Failed to load zenoh config file")
    } else {
        Config::default()
    };
    let z_session = open(z_config).await.expect("Failed to open Zenoh session");

    // UDP socket setup for receiving using socket2

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .expect("Failed to create socket");
    socket
        .set_reuse_address(true)
        .expect("Failed to set SO_REUSEADDR");
    // TTL
    socket
        .set_multicast_ttl_v4(1)
        .expect("Failed to set IP_MULTICAST_TTL");
    // Interface
    socket
        .set_multicast_if_v4(&args.interface)
        .expect("Failed to set IP_MULTICAST_IF");
    // Bind
    socket
        .bind(&SocketAddrV4::new(args.group, args.port).into())
        .expect("Failed to bind socket");
    // Join group
    socket
        .join_multicast_v4(&args.group, &args.interface)
        .expect("Failed to join multicast group");
    socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let recv_socket: UdpSocket = socket.into();

    // UDP socket setup for sending using socket2
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .expect("Failed to create socket");
    socket
        .set_reuse_address(true)
        .expect("Failed to set SO_REUSEADDR");
    // TTL
    socket
        .set_multicast_ttl_v4(1)
        .expect("Failed to set IP_MULTICAST_TTL");
    // Loopback
    socket
        .set_multicast_loop_v4(false)
        .expect("Failed to set IP_MULTICAST_LOOP");
    // Interface
    socket
        .set_multicast_if_v4(&args.interface)
        .expect("Failed to set IP_MULTICAST_IF");

    socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let send_socket: UdpSocket = socket.into();

    // UDP -> Zenoh task
    let publisher = z_session
        .declare_publisher(zenoh_pub_key.clone())
        .allowed_destination(zenoh::sample::Locality::Remote)
        .await
        .expect("Failed to create Zenoh publisher");
    let receiver = recv_socket.try_clone().expect("Failed to clone UDP socket");
    let counter = udp_to_zenoh_counter.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 65535];
        loop {
            match receiver.recv_from(&mut buf) {
                Ok((size, src)) => {
                    counter.fetch_add(1, Ordering::Relaxed);
                    if args.verbose {
                        println!(
                            "UDP -> Zenoh [{} bytes] from {}: {:?}",
                            size,
                            src,
                            &buf[..size]
                        );
                    }
                    publisher
                        .put(buf[..size].to_vec())
                        .await
                        .expect("Failed to publish to Zenoh");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    eprintln!("UDP recv error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    });

    // Zenoh -> UDP task
    let subscriber = z_session
        .declare_subscriber(zenoh_sub_key.clone())
        .allowed_origin(zenoh::sample::Locality::Remote)
        .await
        .expect("Failed to subscribe to Zenoh");
    let sender = send_socket.try_clone().expect("Failed to clone UDP socket");
    let counter = zenoh_to_udp_counter.clone();
    let per_sender_stats_task = per_sender_stats.clone();
    tokio::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            counter.fetch_add(1, Ordering::Relaxed);
            // Parse sender SITE_ID and APPLICATION_ID from sample key
            let key_str = sample.key_expr().to_string();
            let parts: Vec<&str> = key_str.split('/').collect();
            let (site_id, application_id) = if parts.len() >= 5 {
                (parts[3].to_string(), parts[4].to_string())
            } else {
                ("unknown".to_string(), "unknown".to_string())
            };
            {
                let mut stats = per_sender_stats_task.lock().unwrap();
                *stats.entry((site_id, application_id)).or_insert(0) += 1;
            }
            let payload = sample.payload().to_bytes();
            if args.verbose {
                println!("Zenoh -> UDP [{} bytes]: {:?}", payload.len(), payload);
            }
            sender
                .send_to(&payload, SocketAddrV4::new(args.group, args.port))
                .expect("Failed to send UDP packet");
        }
    });

    // Stats thread
    let udp_to_zenoh_counter = udp_to_zenoh_counter.clone();
    let zenoh_to_udp_counter = zenoh_to_udp_counter.clone();
    let per_sender_stats = per_sender_stats.clone();
    if args.stats {
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(10));
            let udp_count = udp_to_zenoh_counter.load(Ordering::Relaxed);
            let zenoh_count = zenoh_to_udp_counter.load(Ordering::Relaxed);
            println!(
                "[STATS] SITE_ID: {} | APPLICATION_ID: {} | UDP->Zenoh: {} | Zenoh->UDP: {}",
                args.site_id, args.application_id, udp_count, zenoh_count
            );
            let stats = per_sender_stats.lock().unwrap();
            println!("[PER-SENDER STATS]");
            for ((site_id, app_id), count) in stats.iter() {
                println!(
                    "From SITE_ID: {} | APPLICATION_ID: {} => {} datagrams",
                    site_id, app_id, count
                );
            }
        });
    }

    // Block main thread
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
