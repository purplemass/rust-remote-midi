extern crate chrono;
extern crate midir;

use std::env;
use std::process::exit;

use uuid::Uuid;

mod midi;
mod socket;
mod utils;

const SERVER_PORT: &str = "6000";
const VIRTUAL_PORT_NAME: &str = "REMOTE-MIDI";
const MSG_SEPARATOR: char = '|';

fn main() {
    let (server_address, _midi_port_id) = match get_vars() {
        Some((server_address, midi_port_id)) => (server_address, midi_port_id),
        None => {
            print_error();
            exit(1)
        }
    };

    let uuid = Uuid::new_v4();
    print_welcome(uuid, &server_address);

    // create Midi in/out
    let midi_in = midi::create_midi_input();
    let midi_out = midi::create_midi_output();
    let (in_ports, out_ports) = match midi::get_ports(&midi_in, &midi_out) {
        Ok((in_ports, out_ports)) => (in_ports, out_ports),
        Err(err) => {
            println!("Error: {}", err);
            (vec![], vec![])
        }
    };

    // create midi out connection
    let midi_out_conn;
    if out_ports.len() == 0 {
        panic!("No MIDI devices found")
    } else {
        let port = &out_ports[0];
        println!("----> using:\t{}", midi_out.port_name(&port).unwrap());
        midi_out_conn = midi::create_out_port(&port, midi_out);
    }

    utils::print_separator();

    // create tcp socket
    let (socket_handle, tx) = match socket::check_tcp_stream(uuid, &server_address, midi_out_conn) {
        Ok((socket_handle, tx)) => {
            println!("Connected to server");
            (socket_handle, tx)
        }
        Err(err) => {
            println!("\nFailed to connect");
            println!("Error: {}", err);
            panic!("No server");
        }
    };

    utils::print_separator();

    for in_port in in_ports {
        midi::create_in_port_listener(uuid, in_port, &tx);
    }

    socket_handle.join().unwrap();
}

fn get_vars() -> Option<(String, String)> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => Some((args[1].to_string(), "".to_string())),
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
    println!("Arguments:\t<SERVER_IP_ADDRESS> <MIDI_PORT_NUMBER>");
    println!("Example:\t./client 127.0.0.1 2");
    println!("{:☠<52}", "");
}
