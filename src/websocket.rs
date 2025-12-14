
use std::error::Error;

use std::sync::Arc;
use std::{
    net::SocketAddr,
};

use futures::SinkExt;
use futures::stream::SplitSink;
use futures_util::{StreamExt};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::{WebSocketStream, tungstenite};

use crate::controller::TrueGearController;

#[derive(Clone)]
pub struct TureGearWebsocketServer {
    addr: String,
    true_gear_controller: TrueGearController,
    connections_outgoings: Arc<Mutex<Vec<SplitSink<WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Message>>>>,
}

impl TureGearWebsocketServer {

    pub fn new(addr: String, true_gear_controller: TrueGearController) -> Self {
        TureGearWebsocketServer {
            addr,
            true_gear_controller,
            connections_outgoings: Arc::new(Mutex::new(Vec::new())),
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

    async fn handle_v1(mut self, ws_stream: WebSocketStream<TcpStream>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        tracing::info!("Handling v1 connection from: {}", addr);

        let (sink, mut source) = ws_stream.split();

        self.connections_outgoings.lock().await.push(sink);

        while let Some(msg) = source.next().await {

            let Ok(msg) = msg else {
                tracing::warn!("Received empty message from {}", addr);
                continue;
            };

            tracing::debug!("Received a raw message from {}: {}", addr, msg);

            match msg {
                tungstenite::Message::Text(text) => {
                    let Ok(control_message) = serde_json::from_str(text.as_str()) else {
                        tracing::error!("Failed to parse message from {}: {}", addr, text);
                        continue;
                    };

                    tracing::info!("Received a message from {}: {:?}", addr, control_message);

                    match self.true_gear_controller.send_message(control_message).await {
                        Ok(_) => tracing::info!("Command sent successfully"),
                        Err(e) => tracing::error!("Failed to send command: {}", e),
                    }
                }
                tungstenite::Message::Close(frame) => {
                    tracing::debug!("Received close message from {}: {:?}", addr, frame);
                    break;
                }
                _ => {
                    tracing::warn!("Received unsupported message type from {}", addr);
                    continue;
                }
            }
            
            // let msg = msg.unwrap().to_text().unwrap().to_string();
        }
        
        tracing::debug!("Closing connection: {}", addr);

        let mut connections_outgoings = self.connections_outgoings.lock().await;
        if let Some(sink_idx) = connections_outgoings.iter().position(|e| e.is_pair_of(&source)) {
            let mut sink = connections_outgoings.remove(sink_idx);
            tracing::debug!("Sending close message to {}", addr);
            let _ = sink.send(tungstenite::Message::Close(Some(CloseFrame{
                code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                reason: "Connection closed".into(),
            }))).await;
        }

        tracing::info!("Connection closed: {}", addr);

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

    pub async fn close(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        tracing::debug!("WebSocket server is shutting down.");
        // close all connections

        let mut connections = self.connections_outgoings.lock().await;
        for conn in connections.iter_mut() {
            conn.send(tungstenite::Message::Close(
                Some(CloseFrame {
                    code: tungstenite::protocol::frame::coding::CloseCode::Normal,
                    reason: "Server is shutting down".into(),
                })
            )).await?;
        }

        Ok(())
    }
}

