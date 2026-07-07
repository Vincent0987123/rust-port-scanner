use std::collections::BTreeSet;
use std::sync::Mutex;
use crate::PORT_RANGE;
use crate::subfiles::scan;
use rayon::prelude::*;
use crate::TARGET_IP;

pub static RESULTS: Mutex<BTreeSet<ScanResult>> = Mutex::new(BTreeSet::new());


#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ScanResult {
    pub port: u16,
    pub result: String,
}


pub fn smart_scanning(){
    let port_range = &PORT_RANGE.lock().unwrap().clone();
    // println!("port_range_len: {}", port_range.len());
    if port_range.len() > 15 {
        let ip = TARGET_IP.lock().unwrap().clone();
        port_range.par_iter().for_each(|port| {
            let result = scan::check_port(&port, &ip);
            RESULTS.lock().unwrap().insert(ScanResult { port: *port, result });
        });
    }
    else {
        panic!();
        scan::run();
    }
    for result in RESULTS.lock().unwrap().iter() {
        println!("{:?}", result);
    }
}

pub fn get_results() -> BTreeSet<ScanResult> {
    RESULTS.lock().unwrap().clone()
}