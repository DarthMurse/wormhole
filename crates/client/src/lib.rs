use std::{io, io::{Read, Write}};
use std::{fs, path::Path};
use std::net::{UdpSocket, Ipv4Addr, IpAddr, SocketAddr};
use std::time::{Duration};
use std::thread::sleep;
use common::*;
use std::process::Command;
use tun::AbstractDevice;
use rand::Rng;

// Forward actual packet in udp packet and send to host
pub fn send_to_host(socket: &UdpSocket, tun: &tun::Device) {

}

// Receive the packet from the host
pub fn receive_from_host(socket: &UdpSocket, tun: &tun::Device) {

}

// Keep alive socket does not need to be shared.
pub fn keep_alive(state: &State) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; MTU];
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), ALIVE_PORT))?;
    socket.connect(SocketAddr::new(IpAddr::V4(SERVER_ADDR), ALIVE_PORT))?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    loop {
        let message = format!("KEEPALIVE\r\n{}\r\n", state.ip.to_string());
        socket.send(message.as_bytes())?;
        println!("Waiting for message");
        match socket.recv(&mut buf) {
            Ok(n) => {
                let packet = &buf[..n];
                let echo = std::str::from_utf8(packet)?;
                println!("The echo message: {echo}");
            },
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut {
                    println!("no reply in 5 seconds, resending...");
                    continue;
                } else {
                    return Err(Box::new(e));
                }
            }
        };
        sleep(Duration::from_secs(10));
    }
}

pub fn load_or_register() -> Result<State, Box<dyn std::error::Error>> {
    println!("Getting client states ...");
    if fs::exists(STATE_PATH)? {
        println!("Reading from existing states ...");
        return Ok(State::read_from_file(STATE_PATH));
    } else {
        println!("Registering new states from the server ...");
        let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(LOCAL_ADDR), ALIVE_PORT))?;
        socket.connect(SocketAddr::new(IpAddr::V4(SERVER_ADDR), ALIVE_PORT))?;
        let mut rng = rand::rng();
        let id: u64 = rng.random();
        let request = String::from("REGISTER REQUEST\r\n");
        let message1 = format!("REGISTER REQUEST\r\n{}\r\n", id);
        socket.send(message1.as_bytes())?;
        let mut buf = [0u8; MTU];
        loop {
            let n = socket.recv(&mut buf)?;
            let packet = &buf[..n];
            match get_packet_type(packet) {
                Some(Respond::RegisterRespond(RegisterStatus::Success)) => {
                    let ip: Ipv4Addr = parse_register_packet(&packet).unwrap();
                    let state: State = State { id, ip };
                    state.write_to_file(STATE_PATH);
                    return Ok::<common::State, Box<dyn std::error::Error>>(state);
                },
                Some(Respond::RegisterRespond(RegisterStatus::IpMaxLimit)) => {
                    panic!("Server IP addresses are full!");
                },
                Some(Respond::RegisterRespond(RegisterStatus::IdConflict)) => {
                    println!("ID conflict, trying the next ID.");
                    let message2 = format!("REGISTER REQUEST\r\n{}\r\n", id+1);
                    socket.send(message2.as_bytes());
                    continue;
                },
                _ => { continue; }
            };
        }
    }
}

