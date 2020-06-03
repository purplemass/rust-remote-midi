use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

const BUFFER_TIME: Duration = Duration::from_millis(1000);
const NOTES: [u8; 3] = [128, 144, 192];

pub struct Buffer {
    pub queue: Vec<String>,
    uuid: uuid::Uuid,
    last_call: Instant,
    last_message: String,
}

impl Buffer {
    pub fn new(uuid: uuid::Uuid) -> Buffer {
        Buffer {
            uuid,
            queue: Vec::new(),
            last_call: Instant::now(),
            last_message: String::from(""),
        }
    }

    pub fn reset(&mut self) {
        self.last_call = Instant::now();
        self.queue = Vec::new();
    }

    pub fn add(&mut self, tx: &Sender<String>, message: &[u8]) {
        let compound_msg = format!("{}{}MIDI:{:?}", self.uuid, crate::MSG_SEPARATOR, message);
        if compound_msg != self.last_message {
            self.last_message = compound_msg.clone();
            if self.last_call.elapsed() < BUFFER_TIME && !self.is_a_note(message) {
                self.queue.push(compound_msg.clone());
            } else {
                tx.send(compound_msg).unwrap();
                self.reset();
            }
        }
    }

    fn is_a_note(&mut self, message: &[u8]) -> bool {
        NOTES.contains(&message[0])
    }
}
