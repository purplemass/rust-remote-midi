# Rust Remote Midi

Based on a client/server chat program written by [Tensor Flow](https://github.com/tensor-programming/Rust_client-server_chat), this application will create a virtual midi port to send and receive midi messages across a network.

*Note: this is highly experimental work which is in no way production ready.*

## Requirements

You need to have [Rust](https://www.rust-lang.org/tools/install) to build or run the client or server applications:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

On Ubuntu you may have to install `build-essential`:

    sudo apt install build-essential

The only requirement are for the `client` applications which needs to be called with 2 arguments:

    cargo run <SERVER_IP_ADDRESS> <MIDI_PORT_NUMBER>

or if calling the executable:

    ./client <SERVER_IP_ADDRESS> <MIDI_PORT_NUMBER>

where `SERVER_IP_ADDRESS` is the IP address of the server (the port number is set in the source code to `6000`) and `MIDI_PORT_NUMBER` is an integer to append to the in-build Midi Port identifier set to `REMOTE_MIDI`.

For example:

    ./client 127.0.0.1 23

will connect to a server running locally and open a virtual Midi port called `REMOTE_MIDI23`.

## Run

Run the server first:

    cd server
    cargo run

Then run the client:

    cd ../client
    cargo run 127.0.0.1 1

## Midi

The application will create a virtual port called `REMOTE_MIDIx` where `x` is a number set in the argument when running the application.
