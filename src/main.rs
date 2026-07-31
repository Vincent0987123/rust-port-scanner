#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod subfiles;

use std::env;
use std::sync::{Arc, Mutex};
use subfiles::io;
use regex::Regex;
use crate::subfiles::gui;
use crate::subfiles::gui::OperatingMode;
use crate::subfiles::mt::{smart_scanning, ScanResult};

pub static WORKING_MODE: Mutex<OperatingMode> = Mutex::new(OperatingMode::Safe);
pub static PORT_RANGE: Mutex<Vec<u16>> = Mutex::new(Vec::new());
pub static TARGET_IP: Mutex<String> = Mutex::new(String::new());
pub static IS_TERMINAL: Mutex<bool> = Mutex::new(false);


struct AllowedInput{
    string_array: Vec<String>
}

fn main() -> eframe::Result<()>{
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--term"{
        *IS_TERMINAL.lock().unwrap() = true;
        #[cfg(windows)]
        attach_console();
        io::print_output("Ready to Start!");
        io::print_output("Which mode should be used to scan?");
        get_working_mode();
        ask_target_ip();
        get_port_range();
        let _ = smart_scanning(None);
        // if get_results().contains(&ScanResult{port: 22, result: ResultType::Open }) {
        //     check_os_p22()
        // }
        io::print_output("Scan completed.");
        Ok(())
    }
    else {
        #[cfg(target_os = "linux")]
        ensure_linux_desktop_entry();

        let icon_bytes = include_bytes!("../rust-logo-512x512.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to load icon image")
            .to_rgba8();

        let (width, height) = image.dimensions();
        let icon_data = egui::IconData {
            rgba: image.into_raw(),
            width,
            height,
        };

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_app_id("rust-portscanner")
                .with_inner_size([1000.0, 500.0])
                .with_min_inner_size([300.0, 220.0])
                .with_icon(Arc::new(icon_data)),
            ..Default::default()
        };

        eframe::run_native(
            "Rust PortScanner",
            native_options,
            Box::new(|cc| Ok(Box::new(gui::Gui::new(cc)))),
        )
    }
}

#[cfg(target_os = "linux")]
fn ensure_linux_desktop_entry() {
    if let Some(home) = dirs::home_dir() {
        let apps_dir = home.join(".local/share/applications");
        let icons_dir = home.join(".local/share/icons");

        let _ = std::fs::create_dir_all(&apps_dir);
        let _ = std::fs::create_dir_all(&icons_dir);

        let icon_path = icons_dir.join("rust-portscanner.png");
        let desktop_path = apps_dir.join("rust-portscanner.desktop");
        
        if !icon_path.exists() {
            let icon_bytes = include_bytes!("../rust-logo-512x512.png");
            let _ = std::fs::write(&icon_path, icon_bytes);
        }
        
        if let Ok(current_exe) = std::env::current_exe() {
            let desktop_entry = format!(
                "[Desktop Entry]\n\
                Type=Application\n\
                Name=Rust PortScanner\n\
                Exec={}\n\
                Icon={}\n\
                StartupWMClass=rust-portscanner\n\
                Terminal=false\n",
                current_exe.to_string_lossy(),
                icon_path.to_string_lossy()
            );

            let _ = std::fs::write(desktop_path, desktop_entry);
        }
    }
}

#[cfg(target_os = "windows")]
fn attach_console() {
    unsafe {
     use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};


        // Hängt STDOUT/STDERR an das Terminal an, aus dem die EXE gestartet wurde
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

pub fn set_working_mode(mode: &OperatingMode) {
    let mut mode_lock = WORKING_MODE.lock().unwrap();
    *mode_lock = *mode;
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
    port_range_lock.clear();
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
        Ok(port_num) if port_num >= 1 && port_num <= u16::MAX => true,
        _ => false,
    }
}

fn check_for_valid_ip(ip: &String) -> bool {
    let ip_regex = Regex::new(r"^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
    ip_regex.is_match(ip)
}

fn get_working_mode() {
    io::print_output("Please enter the mode: \n 1. Safe \n 2. Fast \n");
    let mode = io::get_input();
    if !check_working_mode(&mode) {
        get_working_mode()
    } else {
        let operating_mode = match mode.as_str() {
            "1" => OperatingMode::Safe,
            "2" => OperatingMode::Fast,
            _ => OperatingMode::Safe,
        };
        let mut mode_lock = WORKING_MODE.lock().unwrap();
        *mode_lock = operating_mode;
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
