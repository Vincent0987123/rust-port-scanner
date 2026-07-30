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


pub fn smart_scanning(tx: Option<mpsc::SyncSender<ScanResult>>) -> Result<BTreeSet<ScanResult>, std::io::Error> {
    let port_range = &PORT_RANGE.lock().unwrap().clone();
    let mode = *WORKING_MODE.lock().unwrap();
    let ip = TARGET_IP.lock().unwrap().clone();
    
    // Safe mode: longer timeout, sequential scanning with delays
    // Fast mode: shorter timeout, parallel scanning
    let timeout_ms = if mode == OperatingMode::Safe { 2000 } else { 500 };
    
    if mode == OperatingMode::Safe {
        // Sequential scanning with delays for safe mode
        for port in port_range {
            let (result, error_msg) = scan::check_port(&port, &ip, timeout_ms);
            println!("Port {} is {:?}", port, result);

            if !*IS_TERMINAL.lock().unwrap() {
                if let Err(e) = tx.as_ref().unwrap().send(ScanResult { port: *port, result, additional_info: error_msg.clone() }) {
                    println!("Error sending scan result to GUI: {:?}", e);
                }
            }

            
            RESULTS.lock().unwrap().insert(ScanResult { port: *port, result, additional_info: error_msg });
            
            // Add delay between port checks in safe mode
            thread::sleep(Duration::from_millis(100));
        }
    } else {
        // Fast mode: parallel scanning
        if port_range.len() > 10 {
            port_range.par_iter().for_each(|port| {
                let (result, error_msg) = scan::check_port(&port, &ip, timeout_ms);
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
                let (result, error_msg) = scan::check_port(&port, &ip, timeout_ms);
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
        if mode == OperatingMode::Safe {
            if result.result == ResultType::Error {
                println!("{:?}", result);
            }
        }
        else {
            let display = if result.result == ResultType::Error {
                result.to_string()
            } else {
                format!("Port {} is {:?}", result.port, result.result)
            };
            println!("{}", display);
        }

    }
    Ok(RESULTS.lock().unwrap().clone())
}