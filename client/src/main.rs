extern crate chrono;
extern crate midir;
extern crate rand;

use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::process;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use rand::{thread_rng, Rng};
use uuid::Uuid;

mod midi;

const SERVER_PORT: &str = "6000";
const MIDI_OUTPORT_ID: &str = "REMOTE_MIDI";
const MSG_SEPARATOR: char = '|';
const MSG_SIZE: usize = 256;

fn main() {
    let (error, server_address, midi_port_number) = get_vars();
    if error {process::exit(1)}

    let uuid = Uuid::new_v4();
    let midi_port = &format!("{}{}", MIDI_OUTPORT_ID, midi_port_number);

    print_welcome(uuid, &server_address, &midi_port);

    let conn_out_shared = midi::create_port(midi_port);

    let tx = check_stream(uuid, &server_address, conn_out_shared);

    println!("\nWrite a message or type \":q\" to exit:");

    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        let compound_msg = format!("{}{}{}", uuid, MSG_SEPARATOR, msg);
        if msg == ":q" || tx.send(compound_msg).is_err() {break}
    }

    println!("\nExiting...\n");
}

fn get_vars() -> (bool, String, String) {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            (false, format!("{}:{}", args[1], SERVER_PORT), args[2].to_string())
        },
        _ => {
            println!("{:☠<52}", "");
            println!("Error:\t\tIncorrect/missing arguments");
            println!("Arguments:\t<SERVER_IP_ADDRESS> <MIDI_PORT_NUMBER>");
            println!("Example:\t./client 127.0.0.1 2");
            println!("{:☠<52}", "");
            (true, String::new(), String::new())
        },
    }
}

fn check_stream(uuid: Uuid, server_address: &String, conn_out: std::sync::Arc<std::sync::Mutex<midir::MidiOutputConnection>>) -> std::sync::mpsc::Sender<std::string::String> {
    let mut client = TcpStream::connect(server_address).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

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
                    print_log(&format!("< {}", get_msg(&msg)).to_string());
                    let mut rng = thread_rng();
                    match msg_vec[1] {
                        "a" => midi::play_note(conn_out.clone(), 12, 1),
                        "b" => midi::play_note(conn_out.clone(), 15, 1),
                        "1" => midi::play_single_note(conn_out.clone(), 0x9E, 12, 127),
                        "2" => midi::play_single_note(conn_out.clone(), 0x8E, 12, 0),
                        "3" => midi::play_single_note(conn_out.clone(), 0x9E, 15, 127),
                        "4" => midi::play_single_note(conn_out.clone(), 0x8E, 15, 0),
                        _ => midi::play_note(conn_out.clone(), rng.gen_range(50, 80), 1),
                    }
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                print_log("connection severed");
                break;
            }
        }

        // transmit
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                print_log(&format!("> {}", get_msg(&msg)).to_string());
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    tx
}

fn get_time() -> chrono::DateTime<chrono::Utc> {
    Utc::now()
}

fn print_log(msg: &str) {
    println!("{} | {}", get_time(), msg);
}

fn print_welcome(uuid: Uuid, server_address: &String, midi_port: &String) {
    println!("{:♥<52}", "");
    println!("UUID:\t\t{}", uuid);
    println!("Server:\t\t{}", server_address);
    println!("Midi port:\t{}", midi_port);
    println!("{:♥<52}", "");
}

fn get_msg<'a>(msg: &'a str) -> &'a str {
    let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
    msg_vec[1]
}
