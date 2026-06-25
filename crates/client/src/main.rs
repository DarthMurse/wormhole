use common::*;
use client::*;
use std::net::{UdpSocket, IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration};
use std::io::{Read, Write};
use std::thread;
use std::process::Command;
use tun::{Configuration, Device, AbstractDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let comm_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), COMM_PORT));
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), LOCAL_PORT))?;
    socket.connect(SocketAddr::new(IpAddr::V4(SERVER_ADDR), SERVER_PORT))?;
    //socket.set_read_timeout(Duration::from_secs(5))?;
    let state: State = load_or_register()?;
    println!("State registered! ID = {}, IP = {}.", state.id, state.ip);
    let mut config = Configuration::default();
    config.address(state.ip).netmask((255, 255, 255, 0)).up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let dev = tun::create(&config)?;
    #[cfg(target_os = "macos")]
    Command::new("sudo")
        .args([
            "route",
            "-n",
            "add",
            "-net",
            "172.30.0.0/24",
            "-interface",
            dev.tun_name().unwrap().as_str(),
        ]);
    
    let send_socket = socket.try_clone()?;
    let alive_socket = socket.try_clone()?;
    let recv_socket = socket;
    let (mut send_tun, mut recv_tun) = dev.split();

    let t_send = thread::spawn(move || -> () {
        send_to_host(send_socket, send_tun);
    });
    let t_recv = thread::spawn(move || -> () {
        receive_from_host(recv_socket, recv_tun);
    });
    let t_alive = thread::spawn(move || -> () {
        keep_alive(alive_socket, state);
    });
    t_send.join();
    t_recv.join();
    t_alive.join();
    Ok(())
}
