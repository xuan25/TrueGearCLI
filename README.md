# True Gear CLI

True Gear CLI is a third-party command-line interface tool for communicating True Gear devices via Bluetooth Low Energy (BLE).

It capable of accepting and forwarding incoming command via WebSocket protocols.

## Quick Start

1. Install Rust and Cargo from [rustup.rs](https://rustup.rs/).
2. Clone this repository:
   ```sh
   git clone https://github.com/xuan25/TrueGearCLI.git
   ```
3. Navigate to the project directory:
   ```sh
   cd TrueGearCLI
   ```
4. Build the project using Cargo:
   ```sh
   cargo build --release
   ```
5. Turn on your True Gear device and ensure Bluetooth is enabled on your computer.
6. Run the True Gear CLI:
   ```sh
   cargo run --release
   ```
   it will then connect to the True Gear device via BLE automatically.

   You may see the following output:
   ```sh
   Successfully connected to peripheral: "Name: Truegear_C*"
   Listening on: 127.0.0.1:18233
   ``` 
7. Connect to the WebSocket server at `ws://127.0.0.1:18233/v1/tact/` and send JSON-formatted effect commands.
   
   See [WebSocket Protocol](doc/websocket_protocol.md) for more details on the WebSocket API.

## Command Line Options

You can run `truegearcli --help` to see all available command line options:

```
Usage: truegearcli [OPTIONS]

Options:
  -l, --listen-addr <LISTEN_ADDR>
          Address to listen on for WebSocket connections [default: 127.0.0.1:18233]
  -e, --electical-effect-ratio <ELECTICAL_EFFECT_RATIO>
          Electical effect ratio (0.0 to 1.0) [default: 1]
  -v, --verbose
          Enable verbose logging
  -h, --help
          Print help
  -V, --version
          Print version
```

## TODO

- Battery level fetching
