use std::io::{Read, Write};
use std::{fs, path::Path};
use std::net::{UdpSocket, Ipv4Addr};
use std::time::Duration;
use common::*;
use std::process::Command;
use tun::AbstractDevice;
use anyhow::{Result, Context};
use rand::Rng;

// Forward actual packet in udp packet and send to host
fn send_to_host(socket: &UdpSocket, tun: &tun::Device) {

}

// Receive the packet from the host
fn receive_from_host(socket: &UdpSocket, tun: &tun::Device) {

}

// Initializing states: read states from a file
// This should only be called on startup
fn initialize(state_path: &str) -> Result<(&UdpSocket, &tun::Device)> {

    let mut config = tun::Configuration::default();
    config.netmask((255, 255, 255, 0))
        .address((172, 30, 0, 1))
        .up();
    let device = tun::Device::new(&config).context("tun initialization failed")?;
    let socket = UdpSocket::bind("0.0.0.0:"+COMM_PORT).context("socket bind failed")?;
    (socket, device)
}

fn keep_alive(socket: &UdpSocket, tun: &tun::Device) {
    
}

fn get_state() -> Result<State> {
    println!("Getting client states ...");
    if fs::exists(STATE_PATH)? {
        println!("Reading from existing states ...");
        Ok(State::read_from_file(STATE_PATH))
    } else {
        println!("Registering new states from the server ...");
        let socket = UdpSocket::bind(CLIENT_ADDR_ALIVE)?;
        socket.connect(SERVER_ADDR_ALIVE)?;
        let mut rng = rand::thread_rng();
        let id: u64 = rng.gen();
        let request = String::from("REGISTER REQUEST\r\n");
        socket.send(request.push_str(id.to_string()).push_str("\r\n").as_bytes())?;
        let mut buf = [0u8; MTU];
        loop {
            let n = socket.recv(&mut buf)?;
            let packet = &buf[..n];
            match get_packet_type(packet) {
                Some(Respond::RegisterRespond(RegisterStatus::Success)) => {
                    let ip: Ipv4Addr = parse_register_packet(&packet).unwrap();
                    let state: State = State { id, ip };
                    state.write_to_file(STATE_PATH);
                    Ok(state)
                },
                Some(Respond::RegisterRespond(RegisterStatus::IpMaxLimit)) => {
                    panic!("Server IP addresses are full!");
                },
                Some(Respond::RegisterRespond(RegisterStatus::IdConflict)) => {
                    println!("ID conflict, trying the next ID.");
                    socket.send(request.push_str((id+1).to_string()).push_str("\r\n").as_bytes());
                },
                _ => { continue; }
            };
        }
    }
}

