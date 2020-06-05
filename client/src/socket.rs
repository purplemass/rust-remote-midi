extern crate midir;

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use uuid::Uuid;

use super::midi;
use super::utils;

const MSG_SIZE: usize = 64;
const TIMEOUT: u64 = 3;

pub fn check_tcp_stream(
    uuid: Uuid,
    server_address: &str,
    midi_out_conn: midir::MidiOutputConnection,
) -> Result<(thread::JoinHandle<()>, Sender<String>), Box<dyn std::error::Error>> {
    let server_address = format!("{}:{}", server_address, crate::SERVER_PORT);
    let mut client = TcpStream::connect(server_address)?;
    let _ = client.set_read_timeout(Some(Duration::new(TIMEOUT, 0)))?;
    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();
    let midi_out_conn = Arc::new(Mutex::new(midi_out_conn));

    let handle = thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        // send midi msg received from server
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                let msg_vec: Vec<&str> = msg.split(crate::MSG_SEPARATOR).collect();
                if msg_vec[0] != uuid.to_string() {
                    utils::print_log(&format!("< {}", utils::get_msg(&msg)));
                    let (d1, d2, d3) = parse_message(msg_vec[1]);
                    midi::send_midi_message(midi_out_conn.clone(), d1, d2, d3)
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(err) => {
                println!("\nConnection severed");
                println!("Error: {}", err);
                break;
            }
        }

        // transmit midi msg received from broker
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                utils::print_log(&format!("> {}", utils::get_msg(&msg)));
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        utils::sleep(1);
    });

    Ok((handle, tx))
}

fn parse_message(msg: &str) -> (u8, u8, u8) {
    let mut msg_vec: Vec<&str> = msg.split('[').collect();
    msg_vec = msg_vec[1].split(']').collect();
    msg_vec = msg_vec[0].split(',').collect();
    let d1: u8 = msg_vec[0].trim().parse().unwrap();
    let d2: u8 = msg_vec[1].trim().parse().unwrap();
    let d3: u8 = msg_vec[2].trim().parse().unwrap();
    (d1, d2, d3)
}
