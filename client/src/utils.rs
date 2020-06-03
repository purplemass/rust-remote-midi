use std::thread;

use chrono::{DateTime, Utc};

pub fn sleep(ms: u64) {
    thread::sleep(::std::time::Duration::from_millis(ms));
}

pub fn get_msg(msg: &str) -> &str {
    let msg_vec: Vec<&str> = msg.split(crate::MSG_SEPARATOR).collect();
    msg_vec[1]
}

pub fn print_log(msg: &str) {
    println!("{} | {}", get_time(), msg);
}

pub fn print_separator() {
    println!("{:♥<52}", "");
}

fn get_time() -> DateTime<Utc> {
    Utc::now()
}
