use common::*;
use std::net::{UdpSocket, Ipv4Addr, SocketAddr};
use std::collections::{HashMap};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

const SERVER_STATE_PATH: &str = "server.json";

#[derive(Debug, Serialize, Deserialize)]
struct Mappings {
    id_to_ip: HashMap<u64, Ipv4Addr>,
    ip_to_public: HashMap<Ipv4Addr, SocketAddr>,
    last_ip: Ipv4Addr
}

impl Mappings {
    fn update(&mut self, id: u64, ip: Ipv4Addr, public: SocketAddr) {
        self.last_ip = ip.clone();
        self.id_to_ip.insert(id.clone(), ip.clone());
        self.ip_to_public.insert(ip.clone(), public.clone());
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
    Ipv4Addr::new(0, 0, 0, 0)
}

// Process keepalive and register together
pub fn keepalive(socket: &UdpSocket, mappings: &mut Mappings) {
    let mut buf = [0u8; MTU];
    loop {
        let (n, addr) = socket.recv_from(&mut buf)?;
        let packet = &buf[..n];
        match get_packet_type(packet) {
            Some(Respond::RegisterRequest) => {
                let text = std::str::from_utf8(packet).unwrap();
                let mut lines = text.split("\r\n");
                lines.next();
                let id: u64 = u64::from_str_radix(lines.next().unwrap(), 10).unwrap();
                let code: u8 = 0;
                if mappings.id_to_ip.contains_key(&id) {
                    code = 1;
                } else if mappings.last_ip == Ipv4Addr::new(172, 30, 0, 255) {
                    code = 2;
                }
                let ip = Ipv4Addr::from(u32::from(mappings.last_ip) + 1);
                mappings.update(id, ip, addr);
                let output: &str = "REGISTER RESPOND\r\n" + code.to_string() + "\r\n" + lines.collect::<Vec<&str>>().join("\r\n") + "\r\n";
                socket.send_to(output.as_bytes(), addr)?;
            },
            Some(Respond::Keepalive) => {
                let mut buf = [0u8; MTU];
                let (n, addr) = socket.recv_from(&mut buf)?;
                let packet = &buf[..n];
                let lines = std::str::from_utf8(packet).split("\r\n").next();
                let ip = lines.next().unwrap().parse::<Ipv4Addr>().unwrap();
                if let Some(old_public) = mappings.ip_to_public.get_mut(ip) {
                    if old_public != addr {
                        *old_public = addr;
                        mappings.write_to_file(SERVER_STATE_PATH);
                    }
                    let output: &str = "KEEPALIVE\r\n";
                    socket.send_to(output.as_bytes(), addr)?;
                }
            },
            _ => {
                continue;
            }
        }
    }
}

pub fn forward(socket: &UdpSocket, mappings: &mut Mappings) {

}

pub fn initialize() -> Result<()> {
    let alive_socket = UdpSocket::bind("0.0.0.0:"+ALIVE_PORT)?;
    let comm_socket = UdpSocket::bind("0.0.0.0:"+COMM_PORT)?;
    
    Ok(())
}
