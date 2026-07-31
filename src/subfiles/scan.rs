use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io::ErrorKind;
use crate::subfiles::mt::ResultType;

pub fn check_port(&port: &u16, ip: &String, timeout_ms: u64) -> (ResultType, Option<String>) {
    let address = format!("{}:{}", &ip, port);
    let timeout = Duration::from_millis(timeout_ms);

    match TcpStream::connect_timeout(&address.parse::<SocketAddr>().unwrap(), timeout) {
        Ok(_) => (ResultType::Open, None),
        Err(e) => match e.kind() {
            ErrorKind::ConnectionRefused => (ResultType::Closed, None),
            ErrorKind::TimedOut => (ResultType::Error, Some("Connection timed out - host may be unreachable".to_string())),
            _ => (ResultType::Error, Some(e.to_string())),
        },
    }
}