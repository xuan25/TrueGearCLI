use std::error::Error;

mod ble;
pub mod websocket;

mod controller;

mod true_gear_message;

use crate::websocket::TureGearWebsocketServer;

use clap::{Parser};
use tokio::signal;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Address to listen on
    #[arg(short, long, default_value_t = String::from("127.0.0.1:18233"), help = "Address to listen on for WebSocket connections")]
    listen_addr: String,

    // Electical effect ratio
    #[arg(short, long, default_value_t = 1 as f32, help = "Electical effect ratio (0.0 to 1.0)")]
    electical_effect_ratio: f32,

    // show debug logs
    #[arg(short, long, default_value_t = false, help = "Enable verbose logging")]
    verbose: bool,
}

fn setup_logging(log_level: Level) {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(log_level)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let args = Args::parse();

    let mut log_level = Level::INFO;

    if args.verbose {
        log_level = Level::TRACE;
    }

    setup_logging(log_level);

    let mut true_gear_controller = controller::TrueGearController::new();
    true_gear_controller.set_electical_effect_ratio(args.electical_effect_ratio);
    true_gear_controller.auto_connect().await?;

    let websocket_server = TureGearWebsocketServer::new(args.listen_addr, true_gear_controller.clone());
    tokio::spawn(async move {
        if let Err(e) = websocket_server.run().await {
            tracing::error!("WebSocket server error: {}", e);
        }
    });

    signal::ctrl_c().await.expect("failed to listen for event");

    tracing::info!("Ctrl-C received, shutting down.");

    true_gear_controller.disconnect().await?;

    Ok(())
}
