extern crate chrono;
extern crate midir;

use std::env;
use std::process::exit;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use uuid::Uuid;

mod midi;
mod socket;
mod utils;

const SERVER_PORT: &str = "6000";
const MSG_SEPARATOR: char = '|';

fn main() {
    let midi_in = Arc::new(midi::create_midi_input());
    loop {
        run(Arc::clone(&midi_in));
        println!("{:☠<52}", "");
        utils::sleep(1000);
    }
}

fn run(midi_in: Arc<midir::MidiInput>) {
    let (server_address, midi_device_num) = match get_vars() {
        Some((server_address, midi_device_num)) => (server_address, midi_device_num),
        None => {
            print_error();
            exit(1)
        }
    };

    let uuid = Uuid::new_v4();
    print_welcome(uuid, &server_address);

    // create Midi in/out
    let midi_out = midi::create_midi_output();
    let (in_ports, out_ports) = match midi::get_ports(&midi_in, &midi_out) {
        Ok((in_ports, out_ports)) => (in_ports, out_ports),
        Err(err) => {
            println!("Error: {}", err);
            (vec![], vec![])
        }
    };

    // create midi out connection
    let which: usize = midi_device_num.parse().unwrap();
    if out_ports.len() == 0 {
        panic!("No MIDI devices found")
    }
    if which > out_ports.len() {
        panic!("You cannot pick MIDI device [{}]", which)
    }

    let port = &out_ports[which];
    println!("----> using:\t{}", midi_out.port_name(&port).unwrap());

    let midi_out_conn = midi::create_out_port(&port, midi_out);

    utils::print_separator();

    let (tx, rx) = mpsc::channel::<String>();

    // create tcp socket
    let socket_handle = match socket::check_tcp_stream(uuid, &server_address, midi_out_conn, rx) {
        Ok(socket_handle) => {
            println!("Connected to server");
            socket_handle
        }
        Err(err) => {
            println!("\nFailed to connect");
            println!("Error: {}", err);
            thread::spawn(|| {})
        }
    };

    utils::print_separator();

    let mut senders: Vec<std::sync::mpsc::Sender<String>> = vec![];
    for in_port in in_ports {
        let (_tx, _rx) = mpsc::channel::<String>();
        senders.push(_tx.clone());
        midi::create_in_port_listener(uuid, in_port, &tx, _rx);
    }

    socket_handle.join().unwrap();

    for sender in senders {
        println!("{:?}", sender);
        let _ = sender.send(String::new());
    }
}

fn get_vars() -> Option<(String, String)> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => Some((args[1].to_string(), args[2].to_string())),
        _ => None,
    }
}

fn print_welcome(uuid: Uuid, server_address: &str) {
    utils::print_separator();
    println!("UUID:\t\t{}", uuid);
    println!("Server:\t\t{}:{}", server_address, SERVER_PORT);
    utils::print_separator();
}

fn print_error() {
    println!("{:☠<52}", "");
    println!("Error:\t\tIncorrect/missing arguments");
    println!("Arguments:\t<SERVER_IP_ADDRESS> <MIDI_DEVICE_ID>");
    println!("Example:\t./client 127.0.0.1 2");
    println!("{:☠<52}", "");
}
