use std::result::Result;
use std::collections::BTreeSet;
use std::sync::{mpsc, Mutex};
use crate::{IS_TERMINAL, PORT_RANGE};
use crate::subfiles::scan;
use rayon::prelude::*;
use crate::TARGET_IP;
use crate::WORKING_MODE;
use crate::subfiles::gui::OperatingMode;
use std::thread;
use std::time::Duration;
use rand;
use indicatif::{ProgressBar, ProgressStyle};

pub static RESULTS: Mutex<BTreeSet<ScanResult>> = Mutex::new(BTreeSet::new());

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ScanResult {
    pub port: u16,
    pub result: ResultType,
    pub additional_info: Option<String>
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum ResultType{
    Open,
    Closed,
    Filtered,
    Error
}

impl ScanResult {
    pub fn to_string(&self) -> String {
        match &self.additional_info {
            Some(msg) => format!("Port {} is {:?}: {}", self.port, self.result, msg),
            None => format!("Port {} is {:?}", self.port, self.result),
        }
    }
}

pub fn reset_results() {
    RESULTS.lock().unwrap().clear();
}

fn measure_rtt(ip: &String, port: &u16) -> Option<u64> {
    use std::time::Instant;
    use std::io::ErrorKind;
    
    let address = format!("{}:{}", ip, port);
    let socket_addr = address.parse::<std::net::SocketAddr>().ok()?;
    
    let mut rtts = Vec::new();
    
    for _ in 0..3 {
        let start = Instant::now();
        match std::net::TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1000)) {
            Ok(_) => {
                // println!("RTT measured: {}ms", start.elapsed().as_millis());
                let rtt = start.elapsed().as_millis() as u64;
                rtts.push(rtt);
            }
            Err(e) => match e.kind() {
                ErrorKind::ConnectionRefused => {
                    // println!("Port {} is closed but host is reachable", port);
                    let rtt = start.elapsed().as_millis() as u64;
                    rtts.push(rtt);
                }
                ErrorKind::TimedOut => {
                    // println!("Connection timed out for port {} - host may be unreachable", port);
                    continue;
                }
                _ => {
                    // println!("Error connecting to port {}: {}", port, e);
                    continue;
                }
            },
        }
    }
    
    if rtts.is_empty() {
        None
    } else {
        rtts.sort();
        Some(rtts[rtts.len() / 2])

    }
}

fn calculate_dynamic_timeout(ip: &String, port_range: &[u16]) -> u64 {
    let mut measured_rtts = Vec::new();
    
    let preferred_ports = [22, 80, 443];
    
    let mut sample_ports: Vec<&u16> = preferred_ports.iter()
        .filter(|p| port_range.contains(p))
        .collect();
    
    if sample_ports.len() < 3 {
        sample_ports.extend(port_range.iter().take(3 - sample_ports.len()));
    }
    
    if sample_ports.is_empty() {
        sample_ports = port_range.iter().collect();
    }

    for port in sample_ports {
        if let Some(rtt) = measure_rtt(ip, port) {
            measured_rtts.push(rtt);
        }
    }
    
    if measured_rtts.is_empty() {
        println!("No RTT measurements succeeded, using fallback timeout of 2000ms");
        2000
    } else {
        measured_rtts.sort();
        let median_rtt = measured_rtts[measured_rtts.len() / 2];
        let dynamic_timeout = median_rtt * 3;
        println!("Median RTT: {}ms, Dynamic timeout: {}ms", median_rtt, dynamic_timeout);
        dynamic_timeout.max(50).min(2000)
    }
}

fn check_port_with_retries(port: &u16, ip: &String, timeout_ms: u64, retries: u32) -> (ResultType, Option<String>) {
    let mut results = Vec::new();
    let mut error_msgs = Vec::new();

    for _ in 0..=retries {
        let (result, error_msg) = scan::check_port(port, ip, timeout_ms);
        results.push(result);
        if let Some(msg) = error_msg {
            error_msgs.push(msg);
        }

        if result == ResultType::Open || result == ResultType::Closed {
            break;
        }
    }

    let open_count = results.iter().filter(|&&r| r == ResultType::Open).count();
    let closed_count = results.iter().filter(|&&r| r == ResultType::Closed).count();
    let error_count = results.iter().filter(|&&r| r == ResultType::Error).count();

    let all_timeouts = error_msgs.iter().all(|msg| msg.contains("timed out") || msg.contains("TimedOut"));

    if open_count > closed_count && open_count > error_count {
        (ResultType::Open, None)
    } else if closed_count > open_count && closed_count > error_count {
        (ResultType::Closed, None)
    } else if error_count > 0 && all_timeouts {
        // All retries timed out - host may be unreachable
        (ResultType::Filtered, Some("All connection attempts timed out".to_string()))
    } else if error_count > open_count && error_count > closed_count {
        // Errors dominate but not all timeouts
        (ResultType::Error, error_msgs.first().cloned())
    } else {
        // Tie situation - prefer Closed over Open over Error
        if closed_count >= open_count {
            (ResultType::Closed, None)
        } else if open_count >= error_count {
            (ResultType::Open, None)
        } else {
            (ResultType::Filtered, None)
        }
    }
}


pub fn smart_scanning(tx: Option<mpsc::SyncSender<ScanResult>>) -> Result<BTreeSet<ScanResult>, std::io::Error> {
    let port_range = &PORT_RANGE.lock().unwrap().clone();
    let mode = *WORKING_MODE.lock().unwrap();
    let ip = TARGET_IP.lock().unwrap().clone();
    
    // Smart scanning: use safe mode for single port scans even when fast is selected
    let effective_mode = if port_range.len() == 1 {
        OperatingMode::Safe
    } else {
        mode
    };

    let timeout_ms = if effective_mode == OperatingMode::Safe { 
        2000 
    } else { 
        let dynamic_timeout = calculate_dynamic_timeout(&ip, port_range);
        println!("Dynamic timeout calculated: {}ms", dynamic_timeout);
        dynamic_timeout
    };

    //TODO Animation starten
    let pb = ProgressBar::new_spinner();

    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏") // Zeichen für den Spinner (Punkt-Muster)
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    pb.set_message("Scan is running");
    pb.enable_steady_tick(Duration::from_millis(80));
    
    if effective_mode == OperatingMode::Safe {
        for port in port_range {
            let (result, error_msg) = scan::check_port(&port, &ip, timeout_ms);

            if !*IS_TERMINAL.lock().unwrap() {
                if let Err(e) = tx.as_ref().unwrap().send(ScanResult { port: *port, result, additional_info: error_msg.clone() }) {
                    println!("Error sending scan result to GUI: {:?}", e);
                }
            }


            RESULTS.lock().unwrap().insert(ScanResult { port: *port, result, additional_info: error_msg });


            // Randomize sleep time to avoid overwhelming the target
            let sleep_time = rand::random::<u64>() % 100 + 50;
            thread::sleep(Duration::from_millis(sleep_time));
        }
    } else {
        if port_range.len() > 10 {
            port_range.par_iter().for_each(|port| {
                let (result, error_msg) = check_port_with_retries(&port, &ip, timeout_ms, 2);
                if !*IS_TERMINAL.lock().unwrap() {
                    if let Err(e) = tx.as_ref().unwrap().send(ScanResult { port: *port, result, additional_info: error_msg.clone() }) {
                        println!("Error sending scan result to GUI: {:?}", e);
                    }
                }
                RESULTS.lock().unwrap().insert(ScanResult { port: *port, result, additional_info: error_msg });
            });
        }
        else {
            for port in port_range {
                let (result, error_msg) = check_port_with_retries(&port, &ip, timeout_ms, 2);
                println!("Port {} is {:?}", port, result);
                if !*IS_TERMINAL.lock().unwrap() {
                    if let Err(e) = tx.as_ref().unwrap().send(ScanResult { port: *port, result, additional_info: error_msg.clone() }) {
                        println!("Error sending scan result to GUI: {:?}", e);
                    }
                }
                RESULTS.lock().unwrap().insert(ScanResult { port: *port, result, additional_info: error_msg });
            }
        }
    }
    
    for result in RESULTS.lock().unwrap().iter() {
        println!("{}", result.to_string());
    }
    pb.finish_and_clear();
    Ok(RESULTS.lock().unwrap().clone())
}