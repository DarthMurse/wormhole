// Handling UDP packet
// Registering request: "REGISTER REQUEST\r\n[ID]\r\n"
// Registering respond: "REGISTER RESPOND\r\n[CODE]\r\n[IP]\r\n"
// Keepalive request: "KEEPALIVE\r\n[IP]\r\n"
// Forwarding request: "FORWARD\r\n"
// Forwarding respond: "FORWARD\r\n"
use std::fs;
use std::io::{self, Read, Write};
use std::net::Ipv4Addr;

pub const SERVER_ADDR: Ipv4Addr = Ipv4Addr::new(120, 27, 129, 226);
pub const LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
pub const SERVER_PORT: u16 = 4000;
pub const LOCAL_PORT: u16 = 0;
pub const MTU: usize = 2000;
pub const STATE_PATH: &str = "state.bin";

const REGISTER_REQUEST_PREFIX: &[u8] = b"REGISTER REQUEST\r\n";
const REGISTER_RESPOND_PREFIX: &[u8] = b"REGISTER RESPOND\r\n";
const KEEPALIVE_PREFIX: &[u8] = b"KEEPALIVE\r\n";
const FORWARD_PREFIX: &[u8] = b"FORWARD\r\n";

#[derive(PartialEq, Eq, Debug)]
pub struct State {
    pub id: u64,
    pub ip: Ipv4Addr,
}

impl State {
    pub fn read_from_file(path: &str) -> io::Result<State> {
        let mut file = fs::File::open(path)?;
        let mut id_bytes = [0u8; 8];
        let mut ip_bytes = [0u8; 4];

        file.read_exact(&mut id_bytes)?;
        file.read_exact(&mut ip_bytes)?;

        let id = u64::from_be_bytes(id_bytes);
        let ip = Ipv4Addr::from(ip_bytes);

        Ok(State { id, ip })
    }

    pub fn write_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = fs::File::create(path)?;

        file.write_all(&self.id.to_be_bytes())?;
        file.write_all(&self.ip.octets())?;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum RegisterStatus {
    Success,
    IdConflict,
    IpMaxLimit,
    UndefinedError,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Respond {
    RegisterRequest,
    RegisterRespond(RegisterStatus),
    Keepalive,
    Forward,
}

pub fn get_packet_type(packet: &[u8]) -> Option<Respond> {
    if packet.starts_with(REGISTER_REQUEST_PREFIX) {
        Some(Respond::RegisterRequest)
    } else if packet.starts_with(REGISTER_RESPOND_PREFIX) {
        let text = std::str::from_utf8(packet).ok()?;
        let mut lines = text.split("\r\n");
        lines.next();
        let code = lines.next()?.parse::<u8>().ok()?;
        match code {
            0 => Some(Respond::RegisterRespond(RegisterStatus::Success)),
            1 => Some(Respond::RegisterRespond(RegisterStatus::IdConflict)),
            2 => Some(Respond::RegisterRespond(RegisterStatus::IpMaxLimit)),
            _ => Some(Respond::RegisterRespond(RegisterStatus::UndefinedError)),
        }
    } else if packet.starts_with(KEEPALIVE_PREFIX) {
        Some(Respond::Keepalive)
    } else if packet.starts_with(FORWARD_PREFIX) {
        Some(Respond::Forward)
    } else {
        None
    }
}

// Only for Respond::RequestRespond(true)
pub fn parse_register_packet(packet: &[u8]) -> Option<Ipv4Addr> {
    if let Some(Respond::RegisterRespond(RegisterStatus::Success)) = get_packet_type(packet) {
        let text = std::str::from_utf8(packet).ok()?;
        let mut lines = text.split("\r\n");
        lines.next();
        lines.next();
        lines.next()?.parse::<Ipv4Addr>().ok()
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
        assert_eq!(
            result,
            Some(Respond::RegisterRespond(RegisterStatus::Success))
        );
    }
    #[test]
    pub fn test_parse_register_packet() {
        let packet = b"REGISTER RESPOND\r\n0\r\n172.30.0.2";
        let result = parse_register_packet(&packet[..]);
        assert_eq!(result, Some(Ipv4Addr::new(172, 30, 0, 2)));
    }
    #[test]
    pub fn test_read_write_file() -> io::Result<()> {
        let state = State {
            id: 0x1253e8b9dc2386e2,
            ip: Ipv4Addr::new(172, 30, 0, 2),
        };
        let path = std::env::temp_dir().join(format!("wormhole-state-{}.bin", std::process::id()));
        let path = path.to_string_lossy();

        state.write_to_file(&path)?;
        let new_state = State::read_from_file(&path)?;
        std::fs::remove_file(path.as_ref())?;

        assert_eq!(state, new_state);
        Ok(())
    }
}
