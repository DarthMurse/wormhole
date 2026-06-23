use std::io::{Read, Write};
use std::net::UdpSocket;
use common::describe_ipv4_packet;

const SERVER_ADDR: &str = "120.27.129.226:4000";

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = tun::Configuration::default();

    config
        .address((10, 123, 0, 1))
        .netmask((255, 255, 255, 0))
        .destination((10, 123, 0, 2))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let mut dev = tun::create(&config)?;

    println!("TUN device created");
    println!("Forwarding packets to {SERVER_ADDR}");

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(SERVER_ADDR)?;

    let mut buf = [0u8; 1504];

    loop {
        let n = dev.read(&mut buf)?;
        let packet = &buf[..n];

        println!("Read {} bytes from TUN", n);

        if let Some((src, dst, proto)) = describe_ipv4_packet(packet) {
            println!("IPv4 packet: {src} -> {dst}, protocol={proto}");
        } else {
            println!("Non-IPv4 or invalid packet");
        }

        match socket.send(packet) {
            Ok(_) => {
                println!("Sent {} bytes to relay", n);
            },
            Err(e) => {
                println!("The following error happends: {e}");
                continue;
            }
        }

        match socket.recv(&mut buf) {
            Ok(m) => {
                let recv_packet = &buf[..m];
                dev.write(recv_packet);
            },
            Err(e) => {
                println!("The following error happends: {e}");
            } 
        }

    }
}

