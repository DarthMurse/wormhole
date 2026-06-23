pub fn describe_ipv4_packet(packet: &[u8]) -> Option<(String, String, u8)> {
    if packet.len() < 20 {
        return None;
    }

    let version = packet[0] >> 4;
    let ihl = packet[0] & 0x0f;
    let ip_header_len = ihl as usize * 4;

    if version != 4 || ip_header_len < 20 || packet.len() < ip_header_len {
        return None;
    }

    let total_len = u16::from_be_bytes([packet[2], packet[3]]) as usize;
    if packet.len() < total_len {
        return None;
    }

    let protocol = packet[9];

    let src = format!(
        "{}.{}.{}.{}",
        packet[12], packet[13], packet[14], packet[15]
    );

    let dst = format!(
        "{}.{}.{}.{}",
        packet[16], packet[17], packet[18], packet[19]
    );

    Some((src, dst, protocol))
}

pub fn handle_ipv4_packet(packet: &[u8]) -> Option<Vec<u8>> {
    if packet.len() < 20 {
        return None;
    }

    let version = packet[0] >> 4;
    let ihl = packet[0] & 0x0f;
    let ip_header_len = ihl as usize * 4;

    if version != 4 || ip_header_len < 20 || packet.len() < ip_header_len {
        return None;
    }

    let protocol = packet[9];

    // ICMP = 1
    if protocol != 1 {
        println!("not ICMP, protocol={}", protocol);
        return None;
    }

    let total_len = u16::from_be_bytes([packet[2], packet[3]]) as usize;

    if packet.len() < total_len {
        return None;
    }

    let icmp_offset = ip_header_len;

    // ICMP echo request = 8
    if packet[icmp_offset] != 8 {
        println!("not echo request, icmp_type={}", packet[icmp_offset]);
        return None;
    }

    let src_ip = [packet[12], packet[13], packet[14], packet[15]];
    let dst_ip = [packet[16], packet[17], packet[18], packet[19]];

    let mut reply = packet[..total_len].to_vec();

    // Swap IPv4 addresses.
    reply[12..16].copy_from_slice(&dst_ip);
    reply[16..20].copy_from_slice(&src_ip);

    // Set TTL.
    reply[8] = 64;

    // Recalculate IPv4 header checksum.
    reply[10] = 0;
    reply[11] = 0;
    let ip_sum = checksum(&reply[..ip_header_len]);
    reply[10..12].copy_from_slice(&ip_sum.to_be_bytes());

    // Change ICMP echo request to echo reply.
    reply[icmp_offset] = 0;

    // Recalculate ICMP checksum.
    reply[icmp_offset + 2] = 0;
    reply[icmp_offset + 3] = 0;
    let icmp_sum = checksum(&reply[icmp_offset..total_len]);
    reply[icmp_offset + 2..icmp_offset + 4].copy_from_slice(&icmp_sum.to_be_bytes());

    Some(reply)
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    let mut chunks = data.chunks_exact(2);

    for chunk in &mut chunks {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
        sum += word;
    }

    if let [last] = chunks.remainder() {
        sum += u16::from_be_bytes([*last, 0]) as u32;
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as u16)
}
