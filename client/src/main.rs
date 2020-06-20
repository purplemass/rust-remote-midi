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
    let uuid = Uuid::new_v4();
    let midi_in = Arc::new(midi::create_midi_input());
    loop {
        run(uuid, Arc::clone(&midi_in));
        println!("{:#<52}", "");
        utils::sleep(1000);
    }
}

fn run(uuid: Uuid, midi_in: Arc<midir::MidiInput>) {
    let (server_address, output_id, input_id) = match get_vars() {
        Some((server_address, output_id, input_id)) => (server_address, output_id, input_id),
        None => {
            print_error();
            exit(1)
        }
    };

    print_welcome(uuid, &server_address, &output_id, &input_id);

    // create Midi in/out
    let midi_out = midi::create_midi_output();
    let (in_ports, out_ports) = match midi::get_ports(&midi_in, &midi_out) {
        Ok((in_ports, out_ports)) => (in_ports, out_ports),
        Err(err) => {
            println!("Error: {}", err);
            (vec![], vec![])
        }
    };

    // check selected ports
    let which: usize = output_id.parse().unwrap();
    if out_ports.len() == 0 {
        panic!("No MIDI devices found")
    }
    if which > out_ports.len() {
        panic!("You cannot pick MIDI output device [{}]", which)
    }
    if input_id != "" {
        let which: usize = input_id.parse().unwrap();
        if which > in_ports.len() {
            panic!("You cannot pick MIDI input device [{}]", which)
        }
    }

    // create midi out connection
    let port = &out_ports[which];

    utils::print_thin_separator();
    println!("Output picked >\t{}", midi_out.port_name(&port).unwrap());

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

    // create input port listeners
    let mut senders: Vec<std::sync::mpsc::Sender<String>> = vec![];
    let mut n = 0;
    for in_port in in_ports {
        if input_id == "" || n == input_id.parse().unwrap() {
            let (_tx, _rx) = mpsc::channel::<String>();
            senders.push(_tx.clone());
            midi::create_in_port_listener(uuid, in_port, &tx, _rx);
        }
        n += 1;
    }
    socket_handle.join().unwrap();

    // terminate midi listeners
    for sender in senders {
        let _ = sender.send(String::new());
    }
}

fn get_vars() -> Option<(String, String, String)> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => Some((args[1].to_string(), args[2].to_string(), "".to_string())),
        4 => Some((
            args[1].to_string(),
            args[2].to_string(),
            args[3].to_string(),
        )),
        _ => None,
    }
}

fn print_welcome(uuid: Uuid, server_address: &str, output_id: &str, input_id: &str) {
    utils::print_separator();
    println!("UUID\t\t{}", uuid);
    println!("Server\t\t{}:{}", server_address, SERVER_PORT);
    println!("Output ID\t{}", output_id);
    if input_id != "" {
        println!("Input ID\t{}", input_id);
    }
    utils::print_separator();
}

fn print_error() {
    println!("{:#<52}", "");
    println!("Error:\t\tIncorrect/missing arguments");
    println!("Arguments:\t<SERVER> <OUTPUT_ID> <INPUT_ID>");
    println!("Example:\t./client 127.0.0.1 2");
    println!("{:#<52}", "");
}
