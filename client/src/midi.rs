use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::error::Error;

use midir::{MidiInput, MidiOutput, MidiOutputConnection, Ignore};
use midir::os::unix::{VirtualOutput};

use super::utils::print_separator;


pub fn create_port(midi_port: &str) -> Arc<Mutex<MidiOutputConnection>> {
    let midi_out = MidiOutput::new("RemoteMidiOutput").unwrap();
    let conn_out = midi_out.create_virtual(midi_port).unwrap();
    Arc::new(Mutex::new(conn_out))
}

pub fn play_single_note(conn_out: Arc<Mutex<MidiOutputConnection>>, note_msg: u8, note: u8, velocity: u8) {
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

pub fn get_ports() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let mut midi_in_ports: Vec<String> = Vec::new();
    let mut midi_out_ports: Vec<String> = Vec::new();
    let mut midi_in = MidiInput::new("midir test input")?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("midir test output")?;

    println!("Available input ports:");
    for (i, p) in midi_in.ports().iter().enumerate() {
        if !midi_in.port_name(p)?.contains(crate::MIDI_OUTPORT_ID) {
            println!("{}: {}", i, midi_in.port_name(p)?);
            midi_in_ports.push(midi_in.port_name(p)?);
        }
    }
    print_separator();
    println!("Available output ports:");
    for (i, p) in midi_out.ports().iter().enumerate() {
        if !midi_out.port_name(p)?.contains(crate::MIDI_OUTPORT_ID) {
            println!("{}: {}", i, midi_out.port_name(p)?);
            midi_out_ports.push(midi_out.port_name(p)?);
        }
    }

    print_separator();
    Ok((midi_in_ports, midi_out_ports))
}
