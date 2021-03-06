use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

const BUFFER_TIME: Duration = Duration::from_millis(100);
const NOTES: [u8; 5] = [128, 142, 144, 158, 192];

pub struct Buffer {
    pub queue: Vec<String>,
    uuid: uuid::Uuid,
    last_call: Instant,
}

#[allow(unused)]
impl Buffer {
    pub fn new(uuid: uuid::Uuid) -> Buffer {
        Buffer {
            uuid,
            queue: Vec::new(),
            last_call: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.last_call = Instant::now();
        self.queue = Vec::new();
    }

    pub fn send(&mut self, tx: &Sender<String>, message: &[u8]) {
        let compound_msg = format!("{}{}{:?}", self.uuid, crate::MSG_SEPARATOR, message);
        tx.send(compound_msg).unwrap();
    }

    pub fn add(&mut self, tx: &Sender<String>, message: &[u8]) {
        let compound_msg = format!("{}{}{:?}", self.uuid, crate::MSG_SEPARATOR, message);
        if self.last_call.elapsed() < BUFFER_TIME && !self.is_a_note(message) {
            self.queue.push(compound_msg.clone());
        } else {
            tx.send(compound_msg).unwrap();
            self.reset();
        }
    }

    fn is_a_note(&mut self, message: &[u8]) -> bool {
        NOTES.contains(&message[0])
    }
}
