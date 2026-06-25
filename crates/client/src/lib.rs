use anyhow::{Context, Result, bail};
use common::*;
use rand::Rng;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::thread::sleep;
use std::time::Duration;
use tun::{Reader, Writer};

// Forward actual packet in udp packet and send to host
pub fn send_to_host(socket: UdpSocket, mut dev: Reader) -> Result<()> {
    let mut buf = [0u8; MTU];
    loop {
        let n = dev.read(&mut buf)?;
        let packet = &buf[..n];
        let message = [b"FORWARD\r\n", packet].concat();
        println!("Send forward packet to host");
        socket.send(&message[..])?;
    }
}

// Receive the packet from the host
pub fn receive_from_host(socket: UdpSocket, mut dev: Writer) -> Result<()> {
    let mut buf = [0u8; MTU];
    loop {
        let n = socket.recv(&mut buf)?;
        let packet = &buf[..n];
        if let Some(Respond::Forward) = get_packet_type(packet) {
            let len = b"FORWARD\r\n".len();
            let orig_packet = &buf[len..n];
            dev.write_all(orig_packet)?;
            println!("Receive forward packet from host");
        }
    }
}

pub fn keep_alive(socket: UdpSocket, state: State) -> Result<()> {
    loop {
        let message = format!("KEEPALIVE\r\n{}\r\n", state.ip);
        socket.send(message.as_bytes())?;
        sleep(Duration::from_secs(10));
    }
}

pub fn load_or_register() -> Result<State> {
    println!("Getting client states ...");
    if fs::exists(STATE_PATH)? {
        println!("Reading from existing states ...");
        State::read_from_file(STATE_PATH).context("failed to read client state")
    } else {
        println!("Registering new states from the server ...");
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), LOCAL_PORT))?;
        socket.connect(SocketAddr::new(IpAddr::V4(SERVER_ADDR), SERVER_PORT))?;
        let mut rng = rand::rng();
        let id: u64 = rng.random();
        let message1 = format!("REGISTER REQUEST\r\n{}\r\n", id);
        socket.send(message1.as_bytes())?;
        let mut buf = [0u8; MTU];
        loop {
            let n = socket.recv(&mut buf)?;
            let packet = &buf[..n];
            match get_packet_type(packet) {
                Some(Respond::RegisterRespond(RegisterStatus::Success)) => {
                    let ip: Ipv4Addr =
                        parse_register_packet(packet).context("invalid register response")?;
                    let state: State = State { id, ip };
                    state
                        .write_to_file(STATE_PATH)
                        .context("failed to write client state")?;
                    return Ok(state);
                }
                Some(Respond::RegisterRespond(RegisterStatus::IpMaxLimit)) => {
                    bail!("server IP addresses are full");
                }
                Some(Respond::RegisterRespond(RegisterStatus::IdConflict)) => {
                    println!("ID conflict, trying the next ID.");
                    let message2 = format!("REGISTER REQUEST\r\n{}\r\n", id + 1);
                    socket.send(message2.as_bytes())?;
                    continue;
                }
                _ => {
                    continue;
                }
            };
        }
    }
}
