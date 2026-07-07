use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::transport::{transport_channel, TransportChannelType::Layer3};
use std::net::Ipv4Addr;
use pnet::packet::Packet;

pub fn run() {
    // 1. Transport-Kanal auf Layer 3 (IP-Ebene) öffnen, um Raw Packets zu empfangen
    let protocol = Layer3(IpNextHeaderProtocols::Tcp);
    let (_, mut rx) = transport_channel(4096, protocol)
        .expect("Fehler beim Öffnen des Transport-Kanals. (Root/Admin-Rechte vorhanden?)");

    println!("Warte auf TCP-Antworten für OS-Analyse...");

    let mut iter = pnet::transport::ipv4_packet_iter(&mut rx);
    loop {
        if let Ok((packet, _addr)) = iter.next() {
            // Analysiere das eingehende IPv4-Paket
            analyze_os(&packet);
        }
    }
}

fn analyze_os(ip_packet: &Ipv4Packet) {
    // Wir interessieren uns nur für TCP-Pakete
    if ip_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
        if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {

            // Nur SYN-ACK Antworten auswerten (Wert 18: SYN=2 + ACK=16)
            if tcp_packet.get_flags() == 18 {
                let ttl = ip_packet.get_ttl();
                let window_size = tcp_packet.get_window();
                let source_ip = ip_packet.get_source();

                println!("\n[+] Antwort von {} erhalten!", source_ip);
                println!(" -> TTL: {}", ttl);
                println!(" -> Window Size: {}", window_size);

                // Einfache Heuristik (Grobe Schätzung)
                let os_guess = match ttl {
                    64 => "Linux / Android / macOS",
                    128 => "Windows",
                    255 => "Cisco Router / Embedded OS / FreeBSD",
                    _ => "Unbekanntes OS (Möglicherweise Paket unterwegs modifiziert)",
                };

                println!(" -> Vermutetes Betriebssystem: **{}**", os_guess);
            }
        }
    }
}