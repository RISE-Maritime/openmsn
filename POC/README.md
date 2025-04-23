# [POC] openMSN

Two dockerized services are running at each SHS site:
* One that listens in on the EMSN multicast traffic and forwards packets to a zenoh session
* One that listens on the zenoh session and pushes packets from other sites as local multicast traffic

See docker-compose.yml for details

Tested on 2025-04-23, works as expected. The following are notes for the future:
* udp-multiast has loopback disabled by default, this means that the openmsn "gateway" cannot operate on the same host as the simulator. If loopback is enabled, there will be "rundgång" in the system since the gateway is listening to itself.
* zenoh-cli outputs an extra blank line on stdout. This creates some errors downstream in the processing, mitigated by filtering blank lines using grep.
* Currently, the zenoh subscriber receives also the packages from the same openmsn "gateway". These messages are filtered using grep, but preferably they should not be received at all to save processing power.