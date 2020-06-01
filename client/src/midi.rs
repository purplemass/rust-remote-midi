use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use midir::os::unix::VirtualOutput;
use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort};

use super::utils::print_separator;

const BUFFER_TIME: Duration = Duration::from_millis(1000);
const MONITOR_DELAY: Duration = Duration::from_millis(100);

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

struct Buffer {
    uuid: uuid::Uuid,
    queue: Vec<String>,
    last_call: Instant,
}

impl Buffer {
    fn new(uuid: uuid::Uuid) -> Buffer {
        Buffer {
            uuid,
            queue: Vec::new(),
            last_call: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.last_call = Instant::now();
        self.queue = Vec::new();
    }

    fn add(&mut self, tx: &Sender<String>, message: &[u8]) {
        let compound_msg = format!("{}{}MIDI:{:?}", self.uuid, crate::MSG_SEPARATOR, message);
        if self.last_call.elapsed() < BUFFER_TIME {
            self.queue.push(compound_msg.clone());
        } else {
            tx.send(compound_msg).unwrap();
            self.reset();
        }
    }
}

pub fn create_in_port_listener(uuid: uuid::Uuid, port: MidiInputPort, tx: &Sender<String>) {
    let port_shared = Arc::new(port);
    let tx_clone1 = tx.clone();
    let tx_clone2 = tx.clone();

    thread::spawn(move || {
        let midi_in = create_midi_input();
        let port_name = midi_in.port_name(&port_shared);
        let buffer = Arc::new(Mutex::new(Buffer::new(uuid)));

        println!("Monitoring {}.", port_name.unwrap());

        // monitor buffer
        let cloned_buffer = buffer.clone();
        thread::spawn(move || loop {
            thread::sleep(MONITOR_DELAY);
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
                cloned_buffer.lock().unwrap().add(&tx_clone2, message);
            },
            (),
        );

        loop {
            thread::sleep(Duration::from_millis(1000));
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
    let _ = conn_out_shared.send(&[note_msg, note, velocity]);
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
