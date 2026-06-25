use common::*;
use client::*;
use std::net::{UdpSocket, IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration};
use std::io::{Read, Write};
use tun::{Configuration, Device, AbstractDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let comm_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), COMM_PORT));
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), PORT))?;
    socket.connect(SocketAddr::new(IpAddr::V4(SERVER_ADDR), PORT))?;
    //socket.set_read_timeout(Duration::from_secs(5))?;
    let state: State = load_or_register()?;
    println!("State registered! ID = {}, IP = {}.", state.id, state.ip);
    keep_alive(&socket, &state)?;
    let mut config = Configuration::default();
    config.address(state.ip).netmask((255, 255, 255, 0)).up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let mut dev = tun::create(&config)?;
    Ok(())
}
