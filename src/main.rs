//! A helix ghost-text plugin

use futures_util::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{io::AsyncWriteExt as _, net::TcpListener};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message,
        handshake::{client::Request, server::Response},
    },
};

/// We send this response to the browser extension to establish a connection
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ActivateEditing {
    /// The protocol version
    protocol_version: u32,
    /// The port for the listening WebSocket
    ///
    /// This ideally is the same configured HTTP port (default 4001) but
    /// it does not have to be.
    web_socket_port: u32,
}

/// Represents a selected region of text
#[derive(Serialize, Deserialize)]
struct Selection {
    /// 0-indexed start of the selection
    start: u32,
    /// 0-indexed end of the selection
    end: u32,
}

/// User makes a change in the browser
#[derive(Serialize, Deserialize)]
struct BrowserChange {
    /// The title of the document
    title: String,
    /// The host of the document's url
    url: String,
    /// Not used
    syntax: String,
    /// Value of the text content
    text: String,
    /// User's selections in the browser
    selections: Vec<Selection>,
}

/// User makes a change in the editor
#[derive(Serialize, Deserialize)]
struct EditorChange {
    /// The temporary file content
    text: String,
    /// User's selections in the browser
    selections: Vec<Selection>,
}

const PORT: u16 = 4001;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = format!("127.0.0.1:{PORT}");
    let listener = TcpListener::bind(&addr).await?;

    log::info!("listening on ws://{addr}");

    while let Ok((mut stream, addr)) = listener.accept().await {
        tokio::spawn(async move {
            let mut peek_buf = [0_u8; 1024];
            let n = match stream.peek(&mut peek_buf).await {
                Ok(n) => n,
                Err(e) => {
                    log::error!("failed to peek: {e}");
                    return;
                }
            };

            let header = String::from_utf8_lossy(&peek_buf[..n]);

            if header.contains("Upgrade: websocket") {
                let mut ws_stream =
                    match accept_hdr_async(stream, |req: &Request, res: Response| {
                        log::info!("WebSocket request from: {:?}", req.uri());
                        Ok(res)
                    })
                    .await
                    {
                        Ok(ws) => ws,
                        Err(e) => {
                            log::error!("WebSocket upgrade failed: {e}");
                            return;
                        }
                    };

                log::info!("WebSocket connection established with {addr}");

                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            log::info!("Received text: {text}");
                            let _ = ws_stream.send(Message::Text(text)).await;
                        }
                        Ok(Message::Close(_)) => {
                            log::info!("Client disconnected: {addr}");
                            break;
                        }
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Failed to receive WebSocket message: {err}");
                            break;
                        }
                    }
                }
            } else if header.starts_with("GET / HTTP/1.1") {
                let body = json!({
                    "ProtocolVersion": 1,
                    "WebSocketPort": PORT
                })
                .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );

                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    log::error!("Failed to establish handshake with GhostText: {e}");
                }
            } else {
                log::error!("Unknown request type:\n{header}");
            }
        });
    }

    Ok(())
}
