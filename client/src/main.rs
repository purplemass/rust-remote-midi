extern crate midir;

use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::process;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use uuid::Uuid;

use midir::{MidiOutput};
use midir::os::unix::{VirtualOutput};

const SERVER_PORT: &str = "6000";
const SERVER_IP_KEY: &str = "REMOTE_MIDI_SERVER";
const MIDI_OUTPORT: &str = "REMOTE-MIDI";
const MSG_SIZE: usize = 256;
const MSG_SEPARATOR: char = '|';

fn main() {
    let (error, server_address) = get_vars();
    if error {process::exit(1)}

    let uuid = Uuid::new_v4();

    print_welcome(uuid, &server_address);

    let midi_out = MidiOutput::new("RemoteMidiOutput").unwrap();
    let mut conn_out = midi_out.create_virtual(MIDI_OUTPORT).unwrap();
    play_sample_notes(&mut conn_out);

    let tx = check_stream(uuid, &server_address);

    println!("\nWrite a message or type \":q\" to exit:");

    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        let compound_msg = format!("{}{}{}", uuid, MSG_SEPARATOR, msg);
        if msg == ":q" || tx.send(compound_msg).is_err() {break}
    }

    println!("\nExisiting...\n");
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

fn check_stream(uuid: Uuid, server_address: &String) -> std::sync::mpsc::Sender<std::string::String> {
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
                    print_msg("Rx", &msg);
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        // transmit
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
       print_msg("Tx", &msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    tx
}

fn play_note(conn_out: &mut midir::MidiOutputConnection, note: u8, duration: u64) {
    const NOTE_ON_MSG: u8 = 0x90;
    const NOTE_OFF_MSG: u8 = 0x80;
    const VELOCITY: u8 = 0x64;
    println!("Playing note {:?}", note);
    // We're ignoring errors in here
    let _ = conn_out.send(&[NOTE_ON_MSG, note, VELOCITY]);
    thread::sleep(Duration::from_millis(duration * 150));
    let _ = conn_out.send(&[NOTE_OFF_MSG, note, VELOCITY]);
}

fn play_sample_notes(mut conn_out: &mut midir::MidiOutputConnection) {
    println!("Playing sample notes...\n");

    for _ in 1..3 {
        play_note(&mut conn_out, 66, 4);
        play_note(&mut conn_out, 65, 3);
        play_note(&mut conn_out, 63, 1);
        play_note(&mut conn_out, 61, 6);
        play_note(&mut conn_out, 59, 2);
        play_note(&mut conn_out, 58, 4);
        play_note(&mut conn_out, 56, 4);
        play_note(&mut conn_out, 54, 4);
    }
    thread::sleep(Duration::from_millis(4 * 150));
}

fn print_welcome(uuid: Uuid, server_address: &String) {
    println!("<><><><><><><><><><><><><><><><><><><><><><>");
    println!("UUID:\t{}", uuid);
    println!("Server:\t{}", server_address);
    println!("");
}

fn print_msg(msg_type: &str, msg: &str) {
    let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
    println!("{}: {}", msg_type, msg_vec[1]);
}
