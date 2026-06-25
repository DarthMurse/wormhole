use anyhow::{Context, Result};
use client::*;
use common::*;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::process::Command;
use std::thread;
use tun::{AbstractDevice, Configuration};

fn main() -> Result<()> {
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
    {
        let tun_name = dev.tun_name().context("failed to get TUN device name")?;
        Command::new("sudo")
            .args([
                "route",
                "-n",
                "add",
                "-net",
                "172.30.168.0/24",
                "-interface",
                tun_name.as_str(),
            ])
            .status()?;
    }

    let send_socket = socket.try_clone()?;
    let alive_socket = socket.try_clone()?;
    let recv_socket = socket;
    let (send_tun, recv_tun) = dev.split();

    let t_send = thread::spawn(move || send_to_host(send_socket, send_tun));
    let t_recv = thread::spawn(move || receive_from_host(recv_socket, recv_tun));
    let t_alive = thread::spawn(move || keep_alive(alive_socket, state));

    t_send.join().expect("send thread panicked")?;
    t_recv.join().expect("receive thread panicked")?;
    t_alive.join().expect("keepalive thread panicked")?;

    Ok(())
}
