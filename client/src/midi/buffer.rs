use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

const BUFFER_TIME: Duration = Duration::from_millis(1000);

pub struct Buffer {
    pub queue: Vec<String>,
    uuid: uuid::Uuid,
    last_call: Instant,
}

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

    pub fn add(&mut self, tx: &Sender<String>, message: &[u8]) {
        let compound_msg = format!("{}{}MIDI:{:?}", self.uuid, crate::MSG_SEPARATOR, message);
        if self.last_call.elapsed() < BUFFER_TIME {
            self.queue.push(compound_msg.clone());
        } else {
            tx.send(compound_msg).unwrap();
            self.reset();
        }
    }
}
