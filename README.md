# Wormhole
Using a relay server to build a virtual local network between different devices.

## Usage
On the server, run
```
cargo build -p server
./target/debug/server
```
On the client, run
```
cargo build -p client 
./target/debug/client
```
For now, the local network address and server public IP address is fixed, and the relay only support IPv4 packets. 

## TODO
1. Support configurable local network address and server public IP.
2. Support IPv6.
3. Support direct connection without relay through server.
