use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::process;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

const SERVER_PORT: &str = "6000";
const SERVER_IP_KEY: &str = "REMOTE_MIDI_SERVER";
const MSG_SIZE: usize = 256;
const MSG_SEPARATOR: char = '|';

#[allow(unused)]
fn main() {
    println!("<><><><><><><><><><><><><><><><><><><><><><>");

    let mut server_address: String = format!("127.0.0.1:{}", SERVER_PORT);
    match env::var(SERVER_IP_KEY) {
        Ok(env_var) => server_address = format!("{}:{}", env_var, SERVER_PORT),
        Err(e) => {
            println!("Error: {}\n", e);
            println!("Set the required variable like this:\n");
            println!("export {}=\"xxx.xxx.xxx.xxx\"\n", SERVER_IP_KEY);
            println!("Exiting...\n");
            process::exit(1);
        },
    }

    let uuid = Uuid::new_v4();
    println!("UUID:\t{}", uuid);
    println!("Server:\t{}", server_address);
    println!("");

    let mut client = TcpStream::connect(server_address).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        // receive
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
                if msg_vec[0] != uuid.to_string() {
                    print_msg("Rx", &msg);
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        // transmit
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                print_msg("Tx", &msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message:");

    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        let compound_msg = format!("{}{}{}", uuid, MSG_SEPARATOR, msg);
        if msg == ":quit" || tx.send(compound_msg).is_err() {break}
    }

    println!("bye bye!");
}

fn print_msg(msg_type: &str, msg: &str) {
    let msg_vec: Vec<&str> = msg.split(MSG_SEPARATOR).collect();
    println!("{}: {}", msg_type, msg_vec[1]);
}
