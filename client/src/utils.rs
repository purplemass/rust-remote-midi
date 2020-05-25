use chrono::prelude::*;

pub fn get_msg<'a>(msg: &'a str) -> &'a str {
    let msg_vec: Vec<&str> = msg.split(crate::MSG_SEPARATOR).collect();
    msg_vec[1]
}

pub fn print_log(msg: &str) {
    println!("{} | {}", get_time(), msg);
}

fn get_time() -> chrono::DateTime<chrono::Utc> {
    Utc::now()
}
