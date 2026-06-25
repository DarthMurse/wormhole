use common::*;
use client::*;
use std::net::{UdpSocket, IpAddr, Ipv4Addr, SocketAddr};
use tun::{Configuration, Device, AbstractDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let comm_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), COMM_PORT));
    //let alive_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), COMM_PORT));
    let state: State = load_or_register()?;
    println!("State registered! ID = {}, IP = {}.", state.id, state.ip);
    keep_alive(&state)?;
    let config = Configuration::default();
    Ok(())
}
