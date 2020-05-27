use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use midir::os::unix::VirtualOutput;
use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort};

use super::utils::print_separator;

const DEBOUNCE_RATE: u32 = 25000000;

pub fn create_midi_input() -> MidiInput {
    let mut midi_in = MidiInput::new("MidiInput").unwrap();
    midi_in.ignore(Ignore::None);
    midi_in
}

pub fn create_midi_output() -> MidiOutput {
    MidiOutput::new("MidiOutput").unwrap()
}

pub fn create_virtual_port(midi_port: &str) -> Arc<Mutex<MidiOutputConnection>> {
    let midi_out = MidiOutput::new("RemoteMidiOutput").unwrap();
    let conn_out = midi_out.create_virtual(midi_port).unwrap();
    Arc::new(Mutex::new(conn_out))
}

pub fn create_in_port_listener(uuid: uuid::Uuid, port: MidiInputPort, tx: &Sender<String>) {
    let port_shared = Arc::new(port);
    let tx = tx.clone();
    thread::spawn(move || {
        let midi_in = create_midi_input();
        let port = &port_shared;
        let port_name = midi_in.port_name(port);
        println!("Monitoring {}.", port_name.unwrap());
        let mut now = Instant::now();
        let _conn_in = midi_in.connect(
            port,
            "ConnIn",
            move |_stamp, message, _| {
                if now.elapsed() > Duration::new(0, DEBOUNCE_RATE) {
                    let compound_msg =
                        format!("{}{}MIDI:{:?}", uuid, crate::MSG_SEPARATOR, message);
                    tx.send(compound_msg).unwrap();
                    now = Instant::now();
                }
            },
            (),
        );
        loop {
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

pub fn play_single_note(
    conn_out: Arc<Mutex<MidiOutputConnection>>,
    note_msg: u8,
    note: u8,
    velocity: u8,
) {
    let mut conn_out_shared = conn_out.lock().unwrap();
    let _ = conn_out_shared.send(&[note_msg, note, velocity]);
}

pub fn play_note(conn_out: Arc<Mutex<MidiOutputConnection>>, note: u8, duration: u64) {
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

fn check_valid_port(port_name: String) -> bool {
    !(port_name.contains(crate::MIDI_OUTPORT_ID)
        || port_name.contains("Traktor Virtual Input")
        || port_name.contains("Traktor Virtual Output"))
}

pub fn get_ports(
    midi_in: &MidiInput,
    midi_out: &MidiOutput,
) -> Result<(Vec<MidiInputPort>, Vec<MidiOutputPort>), Box<dyn Error>> {
    let mut in_ports: Vec<MidiInputPort> = Vec::new();
    let mut out_ports: Vec<MidiOutputPort> = Vec::new();

    for port in midi_in.ports() {
        if check_valid_port(midi_in.port_name(&port).unwrap()) {
            println!("Input port:\t{}", midi_in.port_name(&port).unwrap());
            in_ports.push(port);
        }
    }
    if in_ports.is_empty() {
        println!("No input ports found");
    }
    print_separator();
    for port in midi_out.ports() {
        if check_valid_port(midi_out.port_name(&port).unwrap()) {
            println!("Output port:\t{}", midi_out.port_name(&port).unwrap());
            out_ports.push(port);
        }
    }
    if out_ports.is_empty() {
        println!("No output ports found");
    }
    print_separator();

    Ok((in_ports, out_ports))
}
