extern crate fern;

use std::thread;

use chrono::{DateTime, Utc};
use log::info;

pub fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

pub fn get_time() -> DateTime<Utc> {
    Utc::now()
}

pub fn print_log(address: &std::net::SocketAddr, msg: &str) {
    let msg = format!("{} | {}", address, msg);
    info!("{}", msg);
}

pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} | {} | {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%f"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(crate::LOG_FILE)?)
        .apply()?;
    Ok(())
}
