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

// #[allow(unused_variables, unused_mut, dead_code)]
#[allow(unused_variables)]
fn debounce_message(
    uuid: uuid::Uuid,
    mut now: Instant,
    tx: &Sender<String>,
    message: &[u8],
) -> Instant {
    if now.elapsed() > Duration::new(0, DEBOUNCE_RATE) {
        let compound_msg = format!("{}{}MIDI:{:?}", uuid, crate::MSG_SEPARATOR, message);
        // tx.send(compound_msg).unwrap();
        now = Instant::now();
    }
    now
}

#[derive(Clone)]
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

    fn len(&mut self) -> usize {
        self.queue.len()
    }

    fn add(&mut self, _tx: &Sender<String>, message: &[u8]) {
        // let compound_msg = format!("{}{}MIDI:{:?}", self.uuid, crate::MSG_SEPARATOR, message);
        let compound_msg = format!("{:?}", message);
        println!("=======================================");
        println!("NEW ==> [{:?}]", self.queue.len());
        if self.last_call.elapsed() < Duration::new(1, 0) {
            println!("COM ==> ADD");
            self.queue.push(compound_msg.clone());
        } else {
            println!("COM ==> RESET");
            self.reset();
        }
        println!("MSG ==> {}", compound_msg);
        println!("END ==> [{:?}]", self.queue.len());
    }
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

        let buffer = Buffer::new(uuid);

        let test = Arc::new(Mutex::new(buffer));

        // monitor buffer
        let cloned_buffer = test.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::new(0, DEBOUNCE_RATE - 20000));
            let buffer = &mut cloned_buffer.lock().unwrap();
            let buffer_queue = &mut buffer.queue;

            if buffer_queue.len() > 0 {
                println!("[1] [{:?}] [{:?}]", buffer_queue.len(), buffer_queue.last());
                buffer.reset();
                let buffer_queue = &mut buffer.queue;
                println!("[2] [{:?}] [{:?}]", buffer_queue.len(), buffer_queue.last());
            }
        });

        // monitor midi
        let cloned_buffer = test.clone();
        let _conn_in = midi_in.connect(
            port,
            "ConnIn",
            move |_stamp, message, _| {
                cloned_buffer.lock().unwrap().add(&tx, message);
                now = debounce_message(uuid, now, &tx, message);
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
