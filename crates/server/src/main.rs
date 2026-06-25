use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use common::*;
use server::*;
use anyhow::{Result};

fn main() -> Result<()> {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), SERVER_PORT))?;
    let mut mappings = load_mapping().unwrap();
    serve(&socket, &mut mappings);
    Ok(())
}
