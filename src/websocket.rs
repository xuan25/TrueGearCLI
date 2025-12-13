
use std::error::Error;

use std::{
    net::SocketAddr,
};

use futures_util::{StreamExt};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::{WebSocketStream, tungstenite};

use crate::command::{ControlCommand};
use crate::controller::TrueGearController;

#[derive(Clone)]
pub struct TureGearWebsocketServer {
    addr: String,
    true_gear_controller: TrueGearController,
}


impl TureGearWebsocketServer {

    pub fn new(addr: String, true_gear_controller: TrueGearController) -> Self {
        TureGearWebsocketServer {
            addr,
            true_gear_controller,
        }
    }

    async fn accept_async_with_path<T: AsyncRead + AsyncWrite + Unpin>(
        socket: T
    ) -> (Result<WebSocketStream<T>, tungstenite::Error>, Option<String>) {
        let mut path = None;
        let callback = |req: &Request, res: Response| -> Result<Response, ErrorResponse> {
            path = Some(req.uri().path().to_string());
            Ok(res)
        };
        (tokio_tungstenite::accept_hdr_async(socket, callback).await, path)
    }

    async fn handle_v1(self, ws_stream: WebSocketStream<TcpStream>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        tracing::info!("Handling v1 connection from: {}", addr);

        let (_, mut incoming) = ws_stream.split();

        while let Some(msg) = incoming.next().await {
            
            let msg = msg.unwrap().to_text().unwrap().to_string();

            tracing::debug!("Received a raw message from {}: {}", addr, msg);

            let control_command: ControlCommand = serde_json::from_str(&msg)?;

            tracing::info!("Received a message from {}: {:?}", addr, control_command);

            match self.true_gear_controller.send_command(control_command).await {
                Ok(_) => tracing::info!("Command sent successfully"),
                Err(e) => tracing::error!("Failed to send command: {}", e),
            }

        }
        tracing::info!("{} disconnected", &addr);

        Ok(())
    }

    async fn handle_connection(self, raw_stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        tracing::info!("Incoming TCP connection from: {}", addr);

        let (ws_stream_result, path) = TureGearWebsocketServer::accept_async_with_path(raw_stream)
            .await;
        let mut ws_stream = ws_stream_result.expect("Error during the websocket handshake occurred");

        tracing::info!("WebSocket connection established: {}", addr);

        match path.as_deref() {
            Some("/v1/tact/") => {
                self.handle_v1(ws_stream, addr).await?;
            }
            Some(p) => {
                tracing::warn!("Unknown path: {}", p);
                ws_stream.close(Some(CloseFrame { 
                    code: tungstenite::protocol::frame::coding::CloseCode::Policy, 
                    reason: "Unknown path".into() 
                })).await?;
                tracing::warn!("Connection closed: {}", addr);
                return Ok(());
            }
            None => {
                tracing::warn!("No path in the request");
                ws_stream.close(Some(CloseFrame { 
                    code: tungstenite::protocol::frame::coding::CloseCode::Policy, 
                    reason: "No path in the request".into() 
                })).await?;
                tracing::warn!("Connection closed: {}", addr);
                return Ok(());
            }
        }
        
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&self.addr).await;
        let listener = try_socket.expect("Failed to bind");
        tracing::info!("Listening on: {}", self.addr);

        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            let server_clone = self.clone();
            tokio::spawn(server_clone.handle_connection(stream, addr));
        }

        Ok(())
    }
}

