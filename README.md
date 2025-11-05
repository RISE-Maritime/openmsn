# openMSN

**Open Maritime Simulation Network** - A reference implementation for integrating geo-distributed (E)MSN-capable simulators using an open-source communication stack.

## Overview

openMSN (`omsn`) is a high-performance proxy that bridges UDP multicast and [Zenoh](https://zenoh.io), enabling geo-distributed maritime simulators to communicate across different sites. It solves the problem of extending local multicast networks (commonly used in maritime simulation environments) across wide-area networks by leveraging Zenoh's pub/sub infrastructure.

### What is EMSN?

The **European Maritime Simulator Network (EMSN)** is a large-scale geo-distributed simulator network that connects maritime simulation centers across Europe. Inaugurated in November 2014 and developed through the EU-funded [Sea Traffic Management (STM) Validation project](https://www.seatrafficmanagement.info/), EMSN represents the world's largest commercial ship bridge simulator network.

**Key EMSN Characteristics:**
- Connects up to 29 manned ship bridges across 10 simulation centers
- Enables realistic multi-ship scenarios for testing maritime safety and Sea Traffic Management services
- Uses UDP multicast for local communication between simulators at each site
- Allows testing complex scenarios (port approaches, confined waters, search and rescue) in a controlled environment

**openMSN's Role:**
Locally, EMSN rely on local UDP multicast communication, which doesn't work across wide-area networks. Traditionally, the connection of multiple geo-distributed sites involves the use of VPN connections that are centrally administered using closed-source, commercial software and hardware. openMSN provides geo-distributed simulator deployments using open-source technology.

**Learn More:**
- [EMSN Simulations - STM](https://www.seatrafficmanagement.info/emsn-simulations/)
- [Fraunhofer CML - EMSN/APMSN](https://www.cml.fraunhofer.de/en/research-projects/EMSN-APMSN.html)
- [EMSN Technical Description (PDF)](https://stm-stmvalidation.s3.eu-west-1.amazonaws.com/uploads/20190403113947/STM_ID3.2.1-EMSN-Technical-Description.pdf)

## Key Features

- **Bidirectional Proxying**: Seamlessly proxies data between local UDP multicast and remote Zenoh sessions
- **Geo-Distribution**: Connect simulators across different physical sites using Zenoh's distributed pub/sub
- **Low Overhead**: Built in Rust for high performance and minimal latency
- **Flexible Topology**: Support for multiple sites and applications with configurable key spaces
- **Statistics**: Built-in metrics for monitoring traffic flow
- **Docker Support**: Ready-to-deploy container images for easy deployment
- **Multicast Loop Prevention**: Automatically prevents receiving self-sent messages

## Architecture

```
Site A                          Zenoh Network                       Site B
┌─────────────┐                                                ┌─────────────┐
│  Simulator  │                                                │  Simulator  │
│             │                                                │             │
│ UDP:239.0.1 │                                                │ UDP:239.0.2 │
└──────┬──────┘                                                └──────┬──────┘
       │ multicast                                                    │ multicast
       ↓                                                              ↓
┌─────────────┐                ┌────────────┐                ┌─────────────┐
│    omsn     │ ←────────────→ │   Zenoh    │ ←────────────→ │    omsn     │
│   (proxy)   │                │  Router/   │                │   (proxy)   │
│             │                │   Peer     │                │             │
└─────────────┘                └────────────┘                └─────────────┘
```

Each `omsn` instance:
1. Listens to local UDP multicast traffic and publishes to Zenoh
2. Subscribes to Zenoh messages from remote sites and forwards to local multicast

## Prerequisites

- **Rust**: 1.87 or later (if building from source)
- **Docker**: For containerized deployment (optional)
- **Zenoh**: A Zenoh router or peer network (can be configured via config file)

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/RISE-Maritime/openmsn.git
cd openmsn

# Build the release binary
cargo build --release

# The binary will be available at ./target/release/omsn
```

### Using Docker

```bash
# Build the Docker image
docker build -t omsn .

# Or pull from registry (when available)
# docker pull <registry>/omsn:latest
```

## Usage

### Command Line Options

```bash
omsn --help
```

**Required Arguments:**
- `--simulation-id <ID>`: Simulation ID for grouping omsn clients (all sites in the same simulation should use the same ID)
- `--site-id <ID>`: Unique identifier for this site (e.g., "siteA", "siteB")
- `--application-id <ID>`: Application identifier for Zenoh key space (e.g., "bridge", "navigation")
- `--group <IP>`: Multicast group IPv4 address (e.g., "239.0.0.1")
- `--port <PORT>`: Multicast port number (e.g., 50000)

**Optional Arguments:**
- `--interface <IP>`: Network interface IPv4 address (default: "0.0.0.0")
- `--stats`: Enable statistics output every 10 seconds
- `--verbose`: Output all payloads being proxied (for debugging)
- `--zenoh-config <PATH>`: Path to Zenoh configuration file

### Examples

**Basic Usage (Single Site):**

```bash
./target/release/omsn \
  --simulation-id "sim001" \
  --site-id "siteA" \
  --application-id "bridge" \
  --group "239.0.0.1" \
  --port 50000
```

**With Statistics:**

```bash
./target/release/omsn \
  --simulation-id "sim001" \
  --site-id "siteA" \
  --application-id "bridge" \
  --group "239.0.0.1" \
  --port 50000 \
  --stats
```

**Multi-Site Setup (Site A and Site B):**

Site A:
```bash
./target/release/omsn \
  --simulation-id "sim001" \
  --site-id "siteA" \
  --application-id "bridge" \
  --group "239.0.0.1" \
  --port 50000 \
  --stats
```

Site B:
```bash
./target/release/omsn \
  --simulation-id "sim001" \
  --site-id "siteB" \
  --application-id "navigation" \
  --group "239.0.0.2" \
  --port 50000 \
  --stats
```

**Using Docker:**

```bash
docker run --rm --network host omsn \
  --simulation-id "sim001" \
  --site-id "siteA" \
  --application-id "bridge" \
  --group "239.0.0.1" \
  --port 50000 \
  --stats
```

### Zenoh Key Space

Messages are published and subscribed using the following key pattern:

```
omsn/@v1/{simulation_id}/{site_id}/{application_id}
```

- **Publishing**: Each instance publishes to its own key (combination of its site_id and application_id)
- **Subscribing**: Each instance subscribes to `omsn/@v1/{simulation_id}/**` (all sites in the same simulation, except its own messages due to Zenoh's `allowed_origin/destination` settings)

## Configuration

### Zenoh Configuration

By default, `omsn` uses Zenoh's default configuration (peer-to-peer mode). For production deployments, you may want to use a custom Zenoh configuration file.

Create a `zenoh.json5` file:

```json5
{
  mode: "client",
  connect: {
    endpoints: ["tcp/zenoh-router.example.com:7447"]
  }
}
```

Then run omsn with the config:

```bash
./target/release/omsn \
  --zenoh-config zenoh.json5 \
  --simulation-id "sim001" \
  --site-id "siteA" \
  --application-id "bridge" \
  --group "239.0.0.1" \
  --port 50000
```

See [Zenoh documentation](https://zenoh.io/docs/manual/configuration/) for more configuration options.

## Testing

The project includes end-to-end tests using [bats](https://github.com/bats-core/bats-core).

### Running Tests

```bash
# Install bats and helpers (if not using devcontainer)
sudo apt-get install bats
bash .devcontainer/install-bats-helpers.sh

# Build the project
cargo build --release

# Run end-to-end tests
bats tests/end2end-docker.bats
```

### Test Architecture

The E2E tests use Docker Compose to:
1. Start two `omsn` instances on different multicast groups
2. Send a UDP multicast packet to one group
3. Verify it's received on the other group (via Zenoh proxying)

This validates the complete data path: UDP → Zenoh → UDP

### Continuous Integration

Tests run automatically on GitHub Actions for all PRs and pushes to main. See `.github/workflows/e2e-tests.yml`.

## Development

### Using VS Code Devcontainer (Recommended)

This project includes a complete devcontainer setup with all dependencies pre-installed:

1. Open the project in VS Code
2. Click "Reopen in Container" when prompted (or use Command Palette: "Dev Containers: Reopen in Container")
3. The devcontainer includes:
   - Rust (latest stable)
   - Docker CLI
   - bats and test helpers
   - All required tools

### Manual Development Setup

If not using devcontainer:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install bats
sudo apt-get install bats

# Install bats helpers
bash .devcontainer/install-bats-helpers.sh

# Install Docker (for tests)
# Follow instructions at https://docs.docker.com/engine/install/
```

### Project Structure

```
.
├── src/
│   └── main.rs           # Main application code
├── tests/
│   ├── end2end-docker.bats      # E2E test suite
│   └── docker-compose-e2e.yml   # Test infrastructure
├── .devcontainer/         # VS Code devcontainer config
├── .github/workflows/     # CI/CD pipelines
├── POC/                   # Original proof-of-concept (legacy)
├── Cargo.toml            # Rust dependencies
├── Dockerfile            # Container image definition
└── README.md             # This file
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run locally
cargo run -- --simulation-id "test" --site-id "dev" --application-id "app" --group "239.0.0.1" --port 50000

# Run tests
cargo test              # Unit tests (when available)
bats tests/end2end-docker.bats  # E2E tests
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo check
```

## Docker Deployment

### Building the Image

```bash
docker build -t omsn:latest .
```

The Docker image uses multi-stage builds:
1. **Builder stage**: Compiles the Rust binary
2. **Runtime stage**: Minimal Debian slim image with only the binary and tini init system

### Running with Docker Compose

Example `docker-compose.yml`:

```yaml
version: '3.8'

services:
  omsn:
    image: omsn:latest
    network_mode: host  # Required for multicast
    command:
      - --simulation-id=sim001
      - --site-id=siteA
      - --application-id=bridge
      - --group=239.0.0.1
      - --port=50000
      - --stats
    restart: unless-stopped
```

Run with:
```bash
docker-compose up -d
```

## Troubleshooting

### Multicast Issues

**Problem**: Not receiving multicast packets

**Solutions**:
- Ensure your network interface supports multicast
- Check firewall rules allow UDP traffic on your multicast port
- Verify the multicast group address is in the valid range (224.0.0.0 to 239.255.255.255)
- Use `--verbose` flag to see all traffic
- Check the network interface with `--interface` flag

### Zenoh Connection Issues

**Problem**: Cannot connect to remote sites

**Solutions**:
- Verify Zenoh router is accessible (check firewall, network routing)
- Use a Zenoh configuration file with explicit router endpoints
- Check Zenoh logs for connection errors
- Ensure all sites use the same `simulation-id`

### Self-Loop Issues

**Problem**: Receiving own messages

**Solution**: The application automatically prevents this by:
- Setting `IP_MULTICAST_LOOP` to false on the send socket
- Using Zenoh's `allowed_destination: Remote` for publishers
- Using Zenoh's `allowed_origin: Remote` for subscribers

## Background

This project evolved from a proof-of-concept (see `POC/` folder) that used shell scripts and the Zenoh CLI. The current Rust implementation provides:
- Better performance and lower latency
- More robust error handling
- Production-ready stability
- Easier deployment and configuration

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`bats tests/end2end-docker.bats`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Support

For issues, questions, or contributions, please use the [GitHub issue tracker](https://github.com/RISE-Maritime/openmsn/issues).

## Acknowledgments

- Built on [Zenoh](https://zenoh.io) - A pub/sub/query protocol unifying data in motion, data at rest and computations
- Uses [Tokio](https://tokio.rs) for async runtime
- Developed by RISE Maritime

---

**Version**: 0.1.0
**Status**: Active Development
