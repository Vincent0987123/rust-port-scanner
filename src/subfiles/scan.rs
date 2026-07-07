use crate::TARGET_IP;
use crate::PORT_RANGE;
use crate::WORKING_MODE;
use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io::ErrorKind;


pub fn check_port(&port: &u16, ip: &String) -> String {
    let address = format!("{}:{}", &ip, port);
    let timeout = Duration::from_millis(500);

    match TcpStream::connect_timeout(&address.parse::<SocketAddr>().unwrap(), timeout) {
        Ok(_) => "open".to_string(),
        Err(e) => match e.kind() {
            ErrorKind::ConnectionRefused => "closed".to_string(),
            ErrorKind::TimedOut => "filtered".to_string(),
            _ => format!("Exception: {:?}", e),
        },
    }
}

pub fn run() {
    let mode: String = WORKING_MODE.lock().unwrap().clone();
    let ip: String = TARGET_IP.lock().unwrap().clone();
    let port_range = PORT_RANGE.lock().unwrap().clone();


    if mode == "Safe" { println!("Safe mode is not implemented yet"); }
    if mode == "Fast" {
        for port in port_range[0]..=port_range[port_range.len() - 1] {
            println!("Port {} is {}", port, check_port(&port, &ip));
        }
    }
}