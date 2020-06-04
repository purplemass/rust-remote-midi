extern crate midir;

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Sender, TryRecvError};
// use std::sync::{Arc, Mutex};
use std::thread;

use uuid::Uuid;

use super::midi;
use super::utils;

const MSG_SIZE: usize = 64;

pub fn check_tcp_stream(
    uuid: Uuid,
    server_address: &str,
    midi_port: &str,
) -> (thread::JoinHandle<()>, Sender<String>) {
    let conn_out = midi::create_virtual_port(&midi_port);
    let server_address = format!("{}:{}", server_address, crate::SERVER_PORT);
    let mut client = TcpStream::connect(server_address).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    let handle = thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        // send midi msg received from server
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                let msg_vec: Vec<&str> = msg.split(crate::MSG_SEPARATOR).collect();
                if msg_vec[0] != uuid.to_string() {
                    utils::print_log(&format!("< {}", utils::get_msg(&msg)).to_string());
                    let (d1, d2, d3) = parse_message(msg_vec[1]);
                    midi::send_midi_message(conn_out.clone(), d1, d2, d3)
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                utils::print_log("connection severed");
                break;
            }
        }

        // transmit midi msg received from broker
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                utils::print_log(&format!("> {}", utils::get_msg(&msg)).to_string());
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        utils::sleep(1);
    });

    (handle, tx)
}

fn parse_message(msg: &str) -> (u8, u8, u8) {
    let mut msg_midi: Vec<&str> = msg.split('[').collect();
    msg_midi = msg_midi[1].split(']').collect();
    msg_midi = msg_midi[0].split(',').collect();
    let d1: u8 = msg_midi[0].trim().parse().unwrap();
    let d2: u8 = msg_midi[1].trim().parse().unwrap();
    let d3: u8 = msg_midi[2].trim().parse().unwrap();
    (d1, d2, d3)
}
