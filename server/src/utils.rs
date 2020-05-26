use std::thread;

use chrono::prelude::*;

pub fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

pub fn get_time() -> chrono::DateTime<chrono::Utc> {
    Utc::now()
}

pub fn print_log(address: &std::net::SocketAddr, msg: &str) {
    let msg = format!("{} | {} | {}", get_time(), address, msg);
    println!("{}", msg);
}
