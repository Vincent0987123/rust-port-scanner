//! I/O module for PortScanner
//! Handles input/output operations

use std::io;
    pub fn get_input() -> String {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().to_string()
    }

    pub fn print_output(message: &str) {
        println!("{}", message);
    }
