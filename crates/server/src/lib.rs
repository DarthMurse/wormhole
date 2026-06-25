use anyhow::{Result, bail};
use common::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::{fs, fs::File};

pub const SERVER_STATE_PATH: &str = "server.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Mappings {
    id_to_ip: HashMap<u64, Ipv4Addr>,
    ip_to_public: HashMap<Ipv4Addr, SocketAddr>,
    last_ip: Ipv4Addr,
}

impl Mappings {
    pub fn update(&mut self, id: u64, ip: Ipv4Addr, public: SocketAddr) {
        self.last_ip = ip;
        self.id_to_ip.insert(id, ip);
        self.ip_to_public.insert(ip, public);
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_to_file(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;
        println!("Save mapping to server.json.");
        Ok(())
    }

    pub fn read_from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        println!("Read mapping from server.json");
        Ok(serde_json::from_reader(reader)?)
    }
}

impl Default for Mappings {
    fn default() -> Self {
        Self {
            id_to_ip: HashMap::new(),
            ip_to_public: HashMap::new(),
            last_ip: Ipv4Addr::new(172, 30, 168, 0),
        }
    }
}

fn check_ip(packet: &[u8]) -> Result<Ipv4Addr> {
    let len = b"FORWARD\r\n".len();
    let inner = &packet[len..];
    if inner.len() < 20 {
        bail!("forward packet is too short");
    }

    Ok(Ipv4Addr::new(inner[16], inner[17], inner[18], inner[19]))
}

// Process all services together
pub fn serve(socket: &UdpSocket, mappings: &mut Mappings) -> Result<()> {
    let mut buf = [0u8; MTU];
    loop {
        let (n, addr) = socket.recv_from(&mut buf)?;
        let packet = &buf[..n];
        match get_packet_type(packet) {
            Some(Respond::RegisterRequest) => {
                let Ok(text) = std::str::from_utf8(packet) else {
                    continue;
                };
                let mut lines = text.split("\r\n");
                lines.next();
                let Some(Ok(id)) = lines.next().map(str::parse::<u64>) else {
                    continue;
                };
                let mut code: u8 = 0;
                if mappings.id_to_ip.contains_key(&id) {
                    code = 1;
                } else if mappings.last_ip == Ipv4Addr::new(172, 30, 0, 255) {
                    code = 2;
                }
                let ip = Ipv4Addr::from(u32::from(mappings.last_ip) + 1);
                println!("Register ID = {id}, IP = {ip}");
                mappings.update(id, ip, addr);
                mappings.write_to_file(SERVER_STATE_PATH)?;
                let output = format!("REGISTER RESPOND\r\n{code}\r\n{ip}\r\n");
                socket.send_to(output.as_bytes(), addr)?;
            }
            Some(Respond::Keepalive) => {
                let Ok(text) = std::str::from_utf8(packet) else {
                    continue;
                };
                let mut lines = text.split("\r\n");
                lines.next();
                let Some(Ok(ip)) = lines.next().map(str::parse::<Ipv4Addr>) else {
                    continue;
                };
                println!("Keepalive message from {ip}");
                if let Some(old_public) = mappings.ip_to_public.get_mut(&ip)
                    && *old_public != addr
                {
                    *old_public = addr;
                    mappings.write_to_file(SERVER_STATE_PATH)?;
                }
            }
            Some(Respond::Forward) => {
                let Ok(dest_ip) = check_ip(packet) else {
                    continue;
                };

                match mappings.ip_to_public.get(&dest_ip) {
                    Some(out_addr) => {
                        println!("Forward packet to {dest_ip}");
                        socket.send_to(packet, *out_addr)?;
                    }
                    None => {
                        continue;
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }
}

pub fn load_mapping() -> Result<Mappings> {
    if fs::exists(SERVER_STATE_PATH)? {
        Mappings::read_from_file(SERVER_STATE_PATH)
    } else {
        Ok(Mappings::new())
    }
}
