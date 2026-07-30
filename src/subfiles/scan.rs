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
            ErrorKind::TimedOut => (ResultType::Filtered, None),
            _ => (ResultType::Error, Some(e.to_string())),
        },
    }
}

// pub fn run_scan() {
//     let mut local_results: BTreeSet<ScanResult> = BTreeSet::new();
//     let mode: OperatingMode = *WORKING_MODE.lock().unwrap();
//     let ip: String = TARGET_IP.lock().unwrap().clone();
//     let port_range = PORT_RANGE.lock().unwrap().clone();
// 
// 
//     if mode == OperatingMode::Safe { println!("Safe mode is not implemented yet"); }
//     if mode == OperatingMode::Fast {
//         for port in port_range[0]..=port_range[port_range.len() - 1] {
//             println!("Port {} is {}", port, check_port(&port, &ip));
//             let result: String = check_port(&port, &ip);
//             let scanresult: ScanResult = ScanResult { port, result };
//             local_results.insert(scanresult.clone());
//         }
//     }
//     // RESULTS.lock().unwrap().extend(local_results.clone());
//     println!("Local_result len: {}", local_results.len());
//     for result in local_results {
//         GUI_INSTANCE.lock().unwrap().results.insert(result);
//     }
//     println!("Scanned results: {:?}", GUI_INSTANCE.lock().unwrap().results.len());
// }