extern crate chrono;

use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use chrono::prelude::*;

const LOCAL: &str = "0.0.0.0:6000";
const MSG_SEPARATOR: char = '|';
const MSG_SIZE: usize = 256;

fn main() {
    print_welcome();

    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            print_log(&addr, "client connected");

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
                        print_log(&addr, msg_vec[1]);
                        tx.send(msg).expect("failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        print_log(&addr, "client closed");
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn get_time() -> chrono::DateTime<chrono::Utc> {
    Utc::now()
}

fn print_log(address: &std::net::SocketAddr, msg: &str) {
    println!("{} | {} | {}", get_time(), address, msg);
}

fn print_welcome() {
    println!("{:♥<52}", "");
    println!("Identifier:\t{}", "REMOTE MIDI SERVER");
    println!("Started on:\t{}", LOCAL);
    println!("Current time:\t{:?}", get_time());
    println!("{:♥<52}", "");
}
