extern crate chrono;
extern crate midir;
extern crate rand;

use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::process;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use midir::{MidiOutput};
use midir::os::unix::{VirtualOutput};

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

    let midi_out = MidiOutput::new("RemoteMidiOutput").unwrap();
    let conn_out = midi_out.create_virtual(midi_port).unwrap();
    let conn_out_shared = Arc::new(Mutex::new(conn_out));

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
                        "1" => play_note(conn_out.clone(), 12, 1),
                        "2" => play_note(conn_out.clone(), 15, 1),
                        _ => play_note(conn_out.clone(), rng.gen_range(50, 80), 1),
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

fn play_note(conn_out: std::sync::Arc<std::sync::Mutex<midir::MidiOutputConnection>>, note: u8, duration: u64) {
    // https://people.carleton.edu/~jellinge/m208w14/pdf/02MIDIBasics_doc.pdf
    // channel 1:  0x90 off 0x80 on
    // channel 16: 0x9F off 0x8F on
    const NOTE_ON_MSG: u8 = 0x9E;
    const NOTE_OFF_MSG: u8 = 0x8E;
    // const VELOCITY: u8 = 0x64;
    // We're ignoring errors in here
    let mut conn_out_shared = conn_out.lock().unwrap();
    let _ = conn_out_shared.send(&[NOTE_ON_MSG, note, 127]);
    thread::sleep(Duration::from_millis(duration * 150));
    let _ = conn_out_shared.send(&[NOTE_OFF_MSG, note, 0]);
    // print_log(&format!("play note {}", note).to_string());
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
