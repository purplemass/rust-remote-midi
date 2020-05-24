# Rust Remote Midi

Based on a client/server chat program written by [Tensor Flow](https://github.com/tensor-programming/Rust_client-server_chat), this application will create a virtual midi port to send and receive midi messages across a network.

*Note: this is highly experimental work which is in no way production ready.*

## Requirements

The only requirement for this is to set an environment variable of the server for the client app like this (tested on macOS):

    export REMOTE_MIDI_SERVER="xxx.xxx.xxx.xxx"

where `xxx.xxx.xxx.xxx` is the IP address of the server.

The port number is set in the source code as `6000`.

## Run

Run the server first:

    cd server
    cargo run

Then run the client:

    cd ../client
    cargo run

## Midi

The application will create a virtual port called "REMOTE_MIDI".
