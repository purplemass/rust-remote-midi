extern crate chrono;
extern crate midir;

use std::env;
use std::os::unix::process::CommandExt;
use std::process::{exit, Command};

use uuid::Uuid;

mod midi;
mod socket;
mod utils;

const SERVER_PORT: &str = "6000";
const MIDI_OUTPORT_ID: &str = "REMOTE_MIDI";
const MSG_SEPARATOR: char = '|';

fn main() {
    let (server_address, midi_port_number) = match get_vars() {
        Some((server_address, midi_port_number)) => (server_address, midi_port_number),
        None => {
            print_error();
            exit(1)
        }
    };

    let uuid = Uuid::new_v4();
    print_welcome(uuid, &server_address);

    // create Midi in/out
    let midi_port = &format!("{}{}", MIDI_OUTPORT_ID, midi_port_number);
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
    if out_ports.len() > 0 {
        let port = &out_ports[0];
        println!("----> using:\t{}", midi_out.port_name(&port).unwrap());
        midi_out_conn = midi::create_out_port(&port, midi_out);
    } else {
        let port = midi_port;
        println!("----> using:\t{}", port);
        midi_out_conn = midi::create_virtual_port(&port, midi_out);
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
            restart(5, &server_address, &midi_port_number);
            panic!("No server");
        }
    };

    utils::print_separator();

    for in_port in in_ports {
        midi::create_in_port_listener(uuid, in_port, &tx);
    }

    socket_handle.join().unwrap();

    restart(3, &server_address, &midi_port_number);
}

fn restart(seconds: u64, server_address: &str, midi_port_number: &str) {
    println!("Restart in {} seconds", seconds);
    utils::sleep(1000 * seconds);
    let _ = Command::new("clear").spawn();
    Command::new("./client")
        .args(&[&server_address, &midi_port_number])
        .exec();
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
    println!("Arguments:\t<SERVER_IP_ADDRESS> <MIDI_PORT_NUMBER>");
    println!("Example:\t./client 127.0.0.1 2");
    println!("{:☠<52}", "");
}
