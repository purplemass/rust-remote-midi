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
const SERVER_IP_KEY: &str = "REMOTE_MIDI_SERVER";
const MIDI_OUTPORT: &str = "REMOTE_MIDI";
const MSG_SEPARATOR: char = '|';
const MSG_SIZE: usize = 256;

fn main() {
    let (error, server_address) = get_vars();
    if error {process::exit(1)}

    let uuid = Uuid::new_v4();

    print_welcome(uuid, &server_address);

    let midi_out = MidiOutput::new("RemoteMidiOutput").unwrap();
    let conn_out = midi_out.create_virtual(MIDI_OUTPORT).unwrap();
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

fn get_vars() -> (bool, String) {
    let mut server_address = String::new();
    let mut error = false;
    match env::var(SERVER_IP_KEY) {
        Ok(env_var) => server_address = format!("{}:{}", env_var, SERVER_PORT),
        Err(e) => {
            println!("Error: {}\n", e);
            println!("Set the required variable like this:\n");
            println!("export {}=\"xxx.xxx.xxx.xxx\"\n", SERVER_IP_KEY);
            println!("Exiting...\n");
            error = true;
        },
    }
    (error, server_address)
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
                    let mut rng = thread_rng();
                    play_note(conn_out.clone(), rng.gen_range(50, 80), 1);
                    print_log(&format!("< {}", get_msg(&msg)).to_string());
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
    const NOTE_ON_MSG: u8 = 0x90;
    const NOTE_OFF_MSG: u8 = 0x80;
    const VELOCITY: u8 = 0x64;
    // We're ignoring errors in here
    let mut conn_out_shared = conn_out.lock().unwrap();
    let _ = conn_out_shared.send(&[NOTE_ON_MSG, note, VELOCITY]);
    thread::sleep(Duration::from_millis(duration * 150));
    let _ = conn_out_shared.send(&[NOTE_OFF_MSG, note, VELOCITY]);
    // print_log(&format!("play note {}", note).to_string());
}

fn get_time() -> chrono::DateTime<chrono::Utc> {
    Utc::now()
}

fn print_log(msg: &str) {
    println!("{} | {}", get_time(), msg);
}

fn print_welcome(uuid: Uuid, server_address: &String) {
    println!("{:♥<52}", "");
    println!("UUID:\t\t{}", uuid);
    println!("Server:\t\t{}", server_address);
    println!("Midi port:\t{}", MIDI_OUTPORT);
    println!("{:♥<52}", "");
}

fn get_msg<'a>(msg: &'a str) -> &'a str {
    let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
    msg_vec[1]
}
