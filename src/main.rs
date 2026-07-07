mod subfiles;
use std::sync::{Mutex};
use subfiles::io;
use regex::Regex;
use crate::subfiles::mt::{get_results, smart_scanning, ScanResult, RESULTS};
use crate::subfiles::os::check_os_p22;

pub static WORKING_MODE: Mutex<String> = Mutex::new(String::new());
pub static PORT_RANGE: Mutex<Vec<u16>> = Mutex::new(Vec::new());
pub static TARGET_IP: Mutex<String> = Mutex::new(String::new());


struct AllowedInput{
    string_array: Vec<String>
}

fn main() {
    io::print_output("Ready to Start!");
    io::print_output("Which mode should be used to scan?");
    get_working_mode();
    ask_target_ip();
    get_port_range();
    smart_scanning();
    if get_results().contains(&ScanResult{port: 22, result: "open".to_string() }) {
        check_os_p22()
    }
}

pub fn set_working_mode(mode: String) {
    let valid_input = AllowedInput { string_array: vec!["Safe".to_string(), "Fast".to_string()] };
    if check_for_valid_input(valid_input, &mode) {
        let mut mode_lock = WORKING_MODE.lock().unwrap();
        *mode_lock = mode;
    } else {
        io::print_output("Invalid mode. Mode not set.");
    }
}

pub fn set_port_range(ports: (String, String)) {
    let (start_port_str, end_port_str) = ports;

    if !check_for_valid_port(&start_port_str) {
        io::print_output("Invalid start port. Port range not set.");
        return;
    }
    if !check_for_valid_port(&end_port_str) {
        io::print_output("Invalid end port. Port range not set.");
        return;
    }

    let start_port = start_port_str.parse::<u16>().unwrap();
    let end_port = end_port_str.parse::<u16>().unwrap();

    if start_port > end_port {
        io::print_output("Start port must be less than or equal to end port. Port range not set.");
        return;
    }
    let mut port_range_lock = PORT_RANGE.lock().unwrap();
    for port in start_port..=end_port {
        port_range_lock.push(port);
    }
}

pub fn set_target_ip(ip: String) {
    if !check_for_valid_ip(&ip) {
        io::print_output("Invalid IP address. Target IP not set.");
        return;
    }
    let mut target_ip_lock = TARGET_IP.lock().unwrap();
    *target_ip_lock = ip;
}

fn check_for_valid_input(allowed_input: AllowedInput, input: &str) -> bool {
    for string in allowed_input.string_array {
        if input == string { return true; }
    }
    false
}

fn check_for_valid_port(port: &str) -> bool {
    let port_regex = Regex::new(r"^\d+$").unwrap();
    if !port_regex.is_match(port) {
        return false;
    }

    match port.parse::<u16>() {
        Ok(port_num) if port_num >= 1 && port_num <= 65535 => true,
        _ => false,
    }
}

fn check_for_valid_ip(ip: &String) -> bool {
    let ip_regex = Regex::new(r"^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
    ip_regex.is_match(ip)
}

fn get_working_mode() {
    io::print_output("Please enter the mode: \n 1. Safe \n 2. Fast \n");
    let mut mode = io::get_input();
    if !check_working_mode(&mode) {
        get_working_mode()
    } else {
        if mode == "1" { mode = "Safe".parse().unwrap() }
        if mode == "2" { mode = "Fast".parse().unwrap() }
        let mut mode_lock = WORKING_MODE.lock().unwrap();
        *mode_lock = String::from(mode);
    }
}

fn check_working_mode(mode: &str) -> bool {
    let valid_input = AllowedInput{string_array: vec!["1".to_string(), "2".to_string(), "os".to_string()]};
    match mode {
        "1" => io::print_output("Safe mode selected."),
        "2" => io::print_output("Fast mode selected."),
        "os" => io::print_output("OS mode selected."),
        _ => io::print_output("Invalid mode selected."),
    }
    check_for_valid_input(valid_input, mode)
}

fn get_port_range() {
    io::print_output("Please enter the port range:");
    io::print_output("Start Port:");
    let start_port = io::get_input();
    if !check_for_valid_port(&start_port) {
        io::print_output("Invalid port selected.");
        get_port_range();
        return
    }
    io::print_output("End Port:");
    let end_port = io::get_input();
    if !check_for_valid_port(&end_port) {
        io::print_output("Invalid port selected.");
        get_port_range();
        return
    }
    let string = format!("Port range selected: {start_port} - {end_port}");
    io::print_output(&string);
    let mut port_range_lock = PORT_RANGE.lock().unwrap();
    for port in start_port.parse::<u16>().unwrap()..=end_port.parse::<u16>().unwrap() {
        port_range_lock.push(port);
    }
}

fn ask_target_ip(){
    io::print_output("Please enter the target IP:");
    let target_ip = io::get_input();
    if !check_for_valid_ip(&target_ip) {
        io::print_output("Invalid IP selected.");
        ask_target_ip()
    }
    let mut target_ip_lock = TARGET_IP.lock().unwrap();
    *target_ip_lock = target_ip;
}

pub fn get_target_ip() -> String {
    let ip = TARGET_IP.lock().unwrap();
    ip.clone()
}
