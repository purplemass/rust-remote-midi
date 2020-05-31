extern crate chrono;
extern crate midir;

use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::process;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use uuid::Uuid;

mod midi;
mod utils;

const SERVER_PORT: &str = "6000";
const MIDI_OUTPORT_ID: &str = "REMOTE_MIDI";
const MSG_SEPARATOR: char = '|';
const MSG_SIZE: usize = 256;

fn main() {
    let (error, server_address, midi_port_number) = get_vars();
    if error {
        process::exit(1)
    }

    let uuid = Uuid::new_v4();
    let midi_port = &format!("{}{}", MIDI_OUTPORT_ID, midi_port_number);

    print_welcome(uuid, &server_address, &midi_port);

    // create Midi in/out and virtual port
    let midi_in = midi::create_midi_input();
    let midi_out = midi::create_midi_output();
    let (in_ports, _out_ports) = match midi::get_ports(&midi_in, &midi_out) {
        Ok((in_ports, out_ports)) => (in_ports, out_ports),
        Err(err) => {
            println!("Error: {}", err);
            (vec![], vec![])
        }
    };

    let conn_out_shared = midi::create_virtual_port(midi_port);

    let tx = check_tcp_stream(uuid, &server_address, conn_out_shared);

    for in_port in in_ports {
        midi::create_in_port_listener(uuid, in_port, &tx);
    }

    println!("\nWrite a message or type \":q\" to exit:");

    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        let compound_msg = format!("{}{}{}", uuid, MSG_SEPARATOR, msg);
        if msg == ":q" || tx.send(compound_msg).is_err() {
            break;
        }
    }

    println!("\nExiting...\n");
}

fn get_vars() -> (bool, String, String) {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => (
            false,
            format!("{}:{}", args[1], SERVER_PORT),
            args[2].to_string(),
        ),
        _ => {
            println!("{:☠<52}", "");
            println!("Error:\t\tIncorrect/missing arguments");
            println!("Arguments:\t<SERVER_IP_ADDRESS> <MIDI_PORT_NUMBER>");
            println!("Example:\t./client 127.0.0.1 2");
            println!("{:☠<52}", "");
            (true, String::new(), String::new())
        }
    }
}

fn check_tcp_stream(
    uuid: Uuid,
    server_address: &str,
    conn_out: Arc<Mutex<midir::MidiOutputConnection>>,
) -> Sender<String> {
    let mut client = TcpStream::connect(server_address).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        // receive
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
                if msg_vec[0] != uuid.to_string() {
                    utils::print_log(&format!("< {}", utils::get_msg(&msg)).to_string());
                    let msg = msg_vec[1];
                    let mut msg_midi: Vec<&str> = msg.split('[').collect();
                    if msg_midi.len() == 2 {
                        msg_midi = msg_midi[1].split(']').collect();
                        msg_midi = msg_midi[0].split(',').collect();
                        let my_int1: u8 = msg_midi[0].trim().parse().unwrap();
                        let my_int2: u8 = msg_midi[1].trim().parse().unwrap();
                        let my_int3: u8 = msg_midi[2].trim().parse().unwrap();
                        midi::send_midi_message(conn_out.clone(), my_int1, my_int2, my_int3)
                    }
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                utils::print_log("connection severed");
                break;
            }
        }

        // transmit
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

        thread::sleep(Duration::from_millis(100));
    });

    tx
}

fn print_welcome(uuid: Uuid, server_address: &str, midi_port: &str) {
    utils::print_separator();
    println!("UUID:\t\t{}", uuid);
    println!("Server:\t\t{}", server_address);
    println!("Virtual port:\t{}", midi_port);
    utils::print_separator();
}
