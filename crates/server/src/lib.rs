use common::*;
use std::net::{UdpSocket, Ipv4Addr, SocketAddr};
use std::collections::{HashMap};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

const SERVER_STATE_PATH: &str = "server.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Mappings {
    id_to_ip: HashMap<u64, Ipv4Addr>,
    ip_to_public: HashMap<Ipv4Addr, SocketAddr>,
    last_ip: Ipv4Addr
}

impl Mappings {
    pub fn update(&mut self, id: u64, ip: Ipv4Addr, public: SocketAddr) {
        self.last_ip = ip.clone();
        self.id_to_ip.insert(id.clone(), ip.clone());
        self.ip_to_public.insert(ip.clone(), public.clone());
    }

    pub fn new() -> Self {
        let mut id_to_ip: HashMap<u64, Ipv4Addr> = HashMap::new();
        let mut ip_to_public: HashMap<Ipv4Addr, SocketAddr> = HashMap::new();
        let mut last_ip: Ipv4Addr = Ipv4Addr::new(172, 30, 0, 0);
        Mappings {
            id_to_ip,
            ip_to_public,
            last_ip
        }
    }

    pub fn write_to_file(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    pub fn read_from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader(reader)?)
    }
}

fn check_ip(packet: &[u8]) -> Ipv4Addr {
    Ipv4Addr::new(
        packet[16],
        packet[17],
        packet[18],
        packet[19],
    )
}

// Process keepalive and register together
pub fn keepalive(socket: &UdpSocket, mappings: &mut Mappings) {
    let mut buf = [0u8; MTU];
    loop {
        let (n, addr) = socket.recv_from(&mut buf).unwrap();
        let packet = &buf[..n];
        match get_packet_type(packet) {
            Some(Respond::RegisterRequest) => {
                let text = std::str::from_utf8(packet).unwrap();
                let mut lines = text.split("\r\n");
                lines.next();
                let id: u64 = u64::from_str_radix(lines.next().unwrap(), 10).unwrap();
                let mut code: u8 = 0;
                if mappings.id_to_ip.contains_key(&id) {
                    code = 1;
                } else if mappings.last_ip == Ipv4Addr::new(172, 30, 0, 255) {
                    code = 2;
                }
                let ip = Ipv4Addr::from(u32::from(mappings.last_ip) + 1);
                mappings.update(id, ip, addr);
                let output = format!(
                    "REGISTER RESPOND\r\n{}\r\n{}\r\n",
                    code,
                    lines.collect::<Vec<&str>>().join("\r\n")
                );
                socket.send_to(output.as_bytes(), addr).unwrap();
            },
            Some(Respond::Keepalive) => {
                let mut buf = [0u8; MTU];
                let (n, addr) = socket.recv_from(&mut buf).unwrap();
                let packet = &buf[..n];
                let mut lines = std::str::from_utf8(packet).unwrap().split("\r\n");
                lines.next();
                let ip = lines.next().unwrap().parse::<Ipv4Addr>().unwrap();
                if let Some(old_public) = mappings.ip_to_public.get_mut(&ip) {
                    if *old_public != addr {
                        *old_public = addr;
                        mappings.write_to_file(SERVER_STATE_PATH);
                    }
                    let output: &str = "KEEPALIVE\r\n";
                    socket.send_to(output.as_bytes(), addr).unwrap();
                }
            },
            _ => {
                continue;
            }
        }
    }
}

pub fn forward(socket: &UdpSocket, mappings: &mut Mappings) {
    let mut buf = [0u8; MTU];
    loop {
        let (n, addr) = socket.recv_from(&mut buf).unwrap();
        let packet = &buf[..n];
        let dest_ip = check_ip(packet);
        let out_addr = mappings.ip_to_public.get(&dest_ip).unwrap();
        socket.send_to(packet, *out_addr);
    }
}

pub fn initialize() -> Result<()> {
    Ok(())
}
