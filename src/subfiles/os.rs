// use std::io::Read;
// use std::net::TcpStream;
// use std::time::Duration;
// use crate::{get_target_ip};
// 
// pub fn check_os_p22(){
//     let target = format!("{}:22", get_target_ip());
// 
//     
//     if let Ok(mut stream) = TcpStream::connect_timeout(&target.parse().unwrap(), Duration::from_secs(3)) {
//         let mut buffer = [0; 1024];
//         
//         if let Ok(size) = stream.read(&mut buffer) {
//             let response = String::from_utf8_lossy(&buffer[..size]);
//             println!("Response from server:\n{}", response);
// 
//             // Einfaches Pattern Matching für das OS
//             if response.contains("Ubuntu") {
//                 println!("OS: Ubuntu Linux");
//             } else if response.contains("Debian") {
//                 println!("OS: Debian Linux");
//             } else if response.contains("Windows") {
//                 println!("OS: Windows (SSH Server)");
//             }
//         }
//     } else {
//         println!("Port 22 closed or not reachable.");
//     }
// }