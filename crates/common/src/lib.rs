// Handling UDP packet
// Registering request: "REGISTER REQUEST\r\n[ID]\r\n"
// Registering respond: "REGISTER RESPOND\r\n[CODE]\r\n[IP]\r\n"
// Keepalive request: "KEEPALIVE\r\n[IP]\r\n"
// Keepalive respond: "KEEPALIVE\r\n"
// Forwarding request: "FORWARD\r\n"
// Forwarding respond: "FORWARD\r\n"
use std::net::Ipv4Addr;
use std::io::{Read, Write};
use std::fs;

pub const SERVER_ADDR: Ipv4Addr = Ipv4Addr::new(120, 27, 129, 226);
pub const LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
pub const PORT: u16 = 4000;
pub const MTU: usize = 2000;
pub const STATE_PATH: &str = "state.bin";

#[derive(PartialEq, Debug)]
pub struct State {
    pub id: u64,
    pub ip: Ipv4Addr
}

impl State {
    pub fn read_from_file(path: &str) -> State {
        let mut file = fs::File::open(path).unwrap();
        let mut id_bytes = [0u8; 8];
        let mut ip_bytes = [0u8; 4];

        file.read_exact(&mut id_bytes);
        file.read_exact(&mut ip_bytes);

        let id = u64::from_be_bytes(id_bytes);
        let ip = Ipv4Addr::from(ip_bytes);

        State { id, ip }
    }

    pub fn write_to_file(&self, path: &str) {
        let mut file = fs::File::create(path).unwrap();

        file.write_all(&self.id.to_be_bytes());
        file.write_all(&self.ip.octets());
    }
}

#[derive(PartialEq, Debug)]
pub enum RegisterStatus {
    Success,
    IdConflict,
    IpMaxLimit,
    UndefinedError
}

#[derive(PartialEq, Debug)]
pub enum Respond {
    RegisterRequest,
    RegisterRespond(RegisterStatus),
    Keepalive,
    Forward,
}

pub fn get_packet_type(packet: &[u8]) -> Option<Respond> {
    let mut result = None;
    if packet.starts_with(b"REGISTER REQUEST\r\n") { 
        result = Some(Respond::RegisterRequest);
    } else if packet.starts_with(b"REGISTER RESPOND\r\n") {
        let text = std::str::from_utf8(packet).unwrap();
        let mut lines = text.split("\r\n");
        lines.next();
        let code: u8 = lines.next().unwrap().parse::<u8>().unwrap();
        match code {
            0 => { result = Some(Respond::RegisterRespond(RegisterStatus::Success)); },
            1 => { result = Some(Respond::RegisterRespond(RegisterStatus::IdConflict)); },
            2 => { result = Some(Respond::RegisterRespond(RegisterStatus::IpMaxLimit)); },
            _ => { result = Some(Respond::RegisterRespond(RegisterStatus::UndefinedError)); }
        }
    } else if packet.starts_with(b"KEEPALIVE\r\n") {
        result = Some(Respond::Keepalive);
    } else if packet.starts_with(b"FORWARD\r\n") {
        result = Some(Respond::Forward);
    } else {
        result = None;
    }
    result
}

// Only for Respond::RequestRespond(true)
pub fn parse_register_packet(packet: &[u8]) -> Option<Ipv4Addr> {
    if let Some(Respond::RegisterRespond(RegisterStatus::Success)) = get_packet_type(packet) {
        let text = std::str::from_utf8(packet).unwrap();
        println!("{text}");
        let mut lines = text.split("\r\n");
        lines.next();
        lines.next();
        let ip: Ipv4Addr = lines.next().unwrap().parse::<Ipv4Addr>().unwrap();
        Some(ip)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_get_packet_type() {
        let packet = b"REGISTER RESPOND\r\n0\r\nsdfsd";
        let result = get_packet_type(&packet[..]);
        assert_eq!(result, Some(Respond::RegisterRespond(RegisterStatus::Success)));
    }
    #[test]
    pub fn test_parse_register_packet() {
        let packet = b"REGISTER RESPOND\r\n0\r\n172.30.0.2";
        let result = parse_register_packet(&packet[..]);
        assert_eq!(result, Some(Ipv4Addr::new(172, 30, 0, 2)));
    }
    #[test]
    pub fn test_read_write_file() {
        let state = State { id: 0x1253e8b9dc2386e2, ip: Ipv4Addr::new(172, 30, 0, 2)};
        state.write_to_file("test.bin");
        let new_state = State::read_from_file("test.bin");
        assert_eq!(state, new_state);
    }
}
