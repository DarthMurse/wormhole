use anyhow::Result;
use common::*;
use server::*;
use std::net::{IpAddr, SocketAddr, UdpSocket};

fn main() -> Result<()> {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), SERVER_PORT))?;
    let mut mappings = load_mapping()?;
    serve(&socket, &mut mappings)?;
    Ok(())
}
