# Rust Remote Midi

Based on a client/server chat program written by [Tensor Flow](https://github.com/tensor-programming/Rust_client-server_chat), this application consists of a server and a client application to transmit MIDI messages across a network.

The job of the server is to create a TCP listener on port `6000` which any number of clients can connect to. Any messages received on the server will be forwarded to all connected clients.

The client application will connect to any number of MIDI devices attached to its host and forward all MIDI messages to and from the server.

If no MIDI devices are found, the client will create a virtual MIDI port called `REMOTE-MIDI` and communicate with that instead.

*Note: this is highly experimental work which is in not production ready.*

## Prerequisites

You need to have [Rust](https://www.rust-lang.org/tools/install) to build the client or server applications:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

On Ubuntu you may have to install `build-essential`:

    sudo apt install build-essential

## Run

### Server

    cd server
    cargo run

This will create a TCP listener on port `6000`.

It is expected that the server is run first before any clients are created but there is some flexibility built into the client to retry connecting if no server is found.

### Client

    cd client
    cargo run <SERVER_IP_ADDRESS> <MIDI_PORT_ID>

`SERVER_IP_ADDRESS` is a mandatory argument set the the server's IP address/domain name.

`<MIDI_PORT_ID>` is an optional argument appended to the default name of the virtual MIDI port (`REMOTE-MIDI`).

For example:

    cargo run 127.0.0.1

will connect to a server running locally and if no MIDI devices are attached, will open a virtual MIDI port called `REMOTE-MIDI`.

    cargo run 127.0.0.1 250

will connect to a server running locally and if no MIDI devices are attached, will open a virtual MIDI port called `REMOTE-MIDI250`.
