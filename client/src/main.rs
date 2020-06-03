extern crate chrono;
extern crate midir;

use std::env;
use std::io::stdin;
use std::process;

use uuid::Uuid;

mod midi;
mod socket;
mod utils;

const SERVER_PORT: &str = "6000";
const MIDI_OUTPORT_ID: &str = "REMOTE_MIDI";
const MSG_SEPARATOR: char = '|';

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

    let tx = socket::check_tcp_stream(uuid, &server_address, conn_out_shared);

    for in_port in in_ports {
        midi::create_in_port_listener(uuid, in_port, &tx);
    }

    println!("\nWrite a message or type \":q\" to exit:");

    loop {
        let mut buff = String::new();
        stdin()
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

fn print_welcome(uuid: Uuid, server_address: &str, midi_port: &str) {
    utils::print_separator();
    println!("UUID:\t\t{}", uuid);
    println!("Server:\t\t{}", server_address);
    println!("Virtual port:\t{}", midi_port);
    utils::print_separator();
}
