use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message,
        handshake::{client::Request, server::Response},
    },
};

const PORT: u16 = 4001;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = format!("127.0.0.1:{PORT}");
    let listener = TcpListener::bind(&addr).await?;

    log::info!("listening on ws://{addr}");

    while let Ok((mut stream, peer)) = listener.accept().await {
        tokio::spawn(async move {
            let mut peek_buf = [0u8; 1024];
            let n = match stream.peek(&mut peek_buf).await {
                Ok(n) => n,
                Err(e) => {
                    log::error!("failed to peek: {e}");
                    return;
                }
            };

            let header = String::from_utf8_lossy(&peek_buf[..n]);

            if header.contains("Upgrade: websocket") {
                let ws_stream = match accept_hdr_async(stream, |req: &Request, res: Response| {
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

                let mut ws_stream = ws_stream;
                let addr = peer;

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
                    log::error!("Failed to establish handshake with GhostText: {}", e);
                }
            } else {
                log::error!("Unknown request type:\n{}", header);
            }
        });
    }

    Ok(())
}
