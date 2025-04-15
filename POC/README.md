# [POC] openMSN

Two dockerized services are running at each SHS site:
* One that listens in on the EMSN multicast traffic and forwards packets to a zenoh session
* One that listens on the zenoh session and pushes packets from other sites as local multicast traffic

See docker-compose.yml for details