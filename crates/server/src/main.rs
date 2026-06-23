use std::net::UdpSocket;
use common::{describe_ipv4_packet, handle_ipv4_packet};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("0.0.0.0:4000")?;

    println!("Relay server listening on 0.0.0.0:4000");

    let mut buf = [0u8; 2048];

    loop {
        let (n, client_addr) = socket.recv_from(&mut buf)?;
        let packet = &buf[..n];

        println!("Received {} bytes from {}", n, client_addr);

        if let Some((src, dst, proto)) = describe_ipv4_packet(packet) {
            println!("Inner IPv4 packet: {src} -> {dst}, protocol={proto}");
            if let Some(back_packet) = handle_ipv4_packet(packet) {
                socket.send_to(&back_packet[..], client_addr);
                println!("Send packet to {}", client_addr);
            }
        } else {
            println!("Invalid or non-IPv4 packet");
        }
        
    }
}

