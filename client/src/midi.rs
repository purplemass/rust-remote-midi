use std::error::Error;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort};

use super::utils;

mod buffer;

const BUFFER_MONITOR_DELAY: u64 = 10;

pub fn create_midi_input() -> MidiInput {
    let mut midi_in = MidiInput::new("MidiInput").unwrap();
    midi_in.ignore(Ignore::None);
    midi_in
}

pub fn create_midi_output() -> MidiOutput {
    MidiOutput::new("MidiOutput").unwrap()
}

pub fn create_out_port(midi_port: &MidiOutputPort, midi_out: MidiOutput) -> MidiOutputConnection {
    midi_out.connect(&midi_port, "MidiOutput").unwrap()
}

pub fn create_in_port_listener(
    uuid: uuid::Uuid,
    port: MidiInputPort,
    tx: &Sender<String>,
    rx: Receiver<String>,
) {
    let port_shared = Arc::new(port);
    let tx_clone1 = tx.clone();
    let tx_clone2 = tx.clone();

    thread::spawn(move || {
        let midi_in = create_midi_input();
        let port_name = midi_in.port_name(&port_shared);
        let buffer = Arc::new(Mutex::new(buffer::Buffer::new(uuid)));

        println!("Monitoring:\t{}", port_name.unwrap());

        // monitor buffer
        let cloned_buffer = buffer.clone();
        thread::spawn(move || loop {
            utils::sleep(BUFFER_MONITOR_DELAY);
            let buffer = &mut cloned_buffer.lock().unwrap();
            let buffer_queue = &mut buffer.queue;

            if buffer_queue.len() > 0 {
                let last_message = buffer_queue.last();
                &tx_clone1.send(last_message.unwrap().to_string()).unwrap();
                buffer.reset();
            }
        });

        // monitor midi
        let cloned_buffer = buffer.clone();
        let _conn_in = midi_in.connect(
            &port_shared,
            "ConnIn",
            move |_stamp, message, _| {
                cloned_buffer.lock().unwrap().send(&tx_clone2, message);
                // if message[0] != 158 {
                //     cloned_buffer.lock().unwrap().add(&tx_clone2, message);
                // }
            },
            (),
        );

        loop {
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
            utils::sleep(1000);
        }
    });
}

pub fn send_midi_message(
    conn_out: Arc<Mutex<MidiOutputConnection>>,
    note_msg: u8,
    note: u8,
    velocity: u8,
) {
    let mut conn_out_shared = conn_out.lock().unwrap();
    let _ = conn_out_shared
        .send(&[note_msg, note, velocity])
        .unwrap_or_else(|_| println!("Error sending mid"));
}

fn check_valid_port(port_name: String) -> bool {
    !(port_name.contains("Traktor Virtual Output"))
}

pub fn get_ports(
    midi_in: &MidiInput,
    midi_out: &MidiOutput,
) -> Result<(Vec<MidiInputPort>, Vec<MidiOutputPort>), Box<dyn Error>> {
    let mut in_ports: Vec<MidiInputPort> = Vec::new();
    let mut out_ports: Vec<MidiOutputPort> = Vec::new();

    let mut count = 0;
    for port in midi_in.ports() {
        if check_valid_port(midi_in.port_name(&port).unwrap()) {
            println!(
                "Input port  {}\t{}",
                count,
                midi_in.port_name(&port).unwrap()
            );
            in_ports.push(port);
            count += 1;
        }
    }
    if in_ports.is_empty() {
        println!("No input ports found");
    }

    utils::print_separator();

    count = 0;
    for port in midi_out.ports() {
        if check_valid_port(midi_out.port_name(&port).unwrap()) {
            println!(
                "Output port {}\t{}",
                count,
                midi_out.port_name(&port).unwrap()
            );
            out_ports.push(port);
            count += 1;
        }
    }
    if out_ports.is_empty() {
        println!("No output ports found");
    }

    Ok((in_ports, out_ports))
}
