# TrueGear-CLI

[English](README.md) | [简体中文](README_zh.md)

TrueGear-CLI is a third-party command-line interface tool for communicating with TrueGear devices via Bluetooth Low Energy (BLE).

It is capable of accepting and forwarding incoming messages via WebSocket protocols.

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
5. Turn on your TrueGear device and ensure Bluetooth is enabled on your computer.
6. Run TrueGear-CLI:
   ```sh
   cargo run --release
   ```
   It will then connect to the TrueGear device via BLE automatically.

   You may see the following output:
   ```sh
   Successfully connected to peripheral: "Name: Truegear_C*"
   Listening on: 127.0.0.1:18233
   ``` 
7. Connect to the WebSocket server at `ws://127.0.0.1:18233/v1/tact/` and send JSON-formatted effect commands.

   See the [WebSocket Protocol](doc/websocket_protocol.md) for more details on the WebSocket API.

## Command Line Options

You can run `truegearcli --help` to see all available command-line options:

```
Usage: truegear-cli [OPTIONS]

Options:
  -l, --listen-addr <LISTEN_ADDR>
          Address to listen on for WebSocket connections [default: 127.0.0.1:18233]
  -e, --electical-effect-factor <ELECTICAL_EFFECT_FACTOR>
          Strength factor of the Electical effect (usually between 0.0 to 1.5) [default: 1]
  -v, --verbose
          Enable verbose logging
  -h, --help
          Print help
  -V, --version
          Print version
```
