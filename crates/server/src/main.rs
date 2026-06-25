use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use common::*;
use server::*;
use anyhow::{Result};

fn main() -> Result<()> {
    let alive_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), ALIVE_PORT))?;
    let comm_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), COMM_PORT))?;
    let mut mappings = Mappings::new();
    keepalive(&alive_socket, &mut mappings);
    Ok(())
}
