//! The Ghost Text server

use std::io::Write as _;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use crate::data::{BrowserChange, EditorChange, Selection};
use futures_util::{SinkExt as _, StreamExt as _};
use serde::{Deserialize, Serialize};

use serde_json::json;
use steel::rvals::Custom;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;
use tokio::{
    io::AsyncWriteExt as _,
    net::TcpListener,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{Message, handshake::client::Request},
};

/// Server for the GhostText client
#[derive(Clone)]
pub struct Server {
    /// Sender to the WebSocket connection
    sender: Arc<tokio::sync::Mutex<Option<UnboundedSender<Message>>>>,
    /// Thread where the server lives
    thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Shutdown tx
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    /// Send changes
    change_tx: Arc<Mutex<Option<UnboundedSender<EditorChange>>>>,
}

impl Custom for Server {}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::missing_docs_in_private_items, missing_docs, reason = "todo")]
struct OutgoingPayload {
    text: String,
    selections: Vec<Selection>,
}

impl Server {
    /// Port for the Ghost Text protocol
    const PORT: u16 = 4001;

    /// Create a new server
    pub fn new() -> Self {
        Self {
            sender: Arc::new(tokio::sync::Mutex::new(None)),
            thread_handle: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            change_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Start it
    pub fn start(&self) {
        let this = self.clone();

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let (change_tx, change_rx) = mpsc::unbounded_channel();

        *self.change_tx.lock().expect("not to fail") = Some(change_tx);

        let thread = thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                if let Err(e) = this.run(shutdown_rx, change_rx).await {
                    log::error!("Server error: {e}");
                }
            });
        });

        *self.shutdown_tx.lock().expect("not to fail") = Some(shutdown_tx);
        *self.thread_handle.lock().expect("not to fail") = Some(thread);
    }

    /// Stop the Ghost Text server
    pub fn stop(&self) {
        let sender = self.shutdown_tx.lock().expect("not to fail").take();
        if let Some(tx) = sender {
            let _ = tx.send(());
        }

        let join_handle = self.thread_handle.lock().expect("not to fail").take();
        if let Some(handle) = join_handle {
            let _ = handle.join();
        }

        log::info!("GhostText server stopped");
    }

    /// Run the Ghost Text server
    pub async fn run(
        self,
        mut shutdown: oneshot::Receiver<()>,
        mut change_rx: UnboundedReceiver<EditorChange>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("127.0.0.1:{}", Self::PORT);
        let listener = TcpListener::bind(&addr).await?;
        log::info!("Listening on ws://{addr}");

        tokio::select! {
            _ = async {
                loop {
                    tokio::select! {
                        Some(change) = change_rx.recv() => {
                            let msg = Message::Text(serde_json::to_string(&change)?.into());
                            if let Some(tx) = self.sender.lock().await.as_ref() {
                                let _ = tx.send(msg);
                            }
                         }
                        Ok((mut stream, addr)) = listener.accept() => {
                            let this = self.clone();
                            tokio::spawn(async move {
                                let mut buf = [0_u8; 1024];
                                let n = match stream.peek(&mut buf).await {
                                    Ok(n) => n,
                                    Err(e) => {
                                        log::error!("peek failed: {e}");
                                        return;
                                    }
                                };

                                let header = String::from_utf8_lossy(&buf[..n]);

                                if header.contains("Upgrade: websocket") {
                                    let ws_stream = match accept_hdr_async(stream, |req: &Request, res| {
                                        log::info!("WebSocket request from: {:?}", req.uri());
                                        Ok(res)
                                    }).await {
                                        Ok(ws) => ws,
                                        Err(e) => {
                                            log::error!("WebSocket upgrade failed: {e}");
                                            return;
                                        }
                                    };

                                    log::info!("WebSocket connection from {addr}");

                                    let (mut write, mut read) = ws_stream.split();
                                    let (tx, mut rx) = mpsc::unbounded_channel();

                                    {
                                        let mut lock = this.sender.lock().await;
                                        *lock = Some(tx);
                                    }

                                    let forward = tokio::spawn(async move {
                                        while let Some(msg) = rx.recv().await {
                                            if let Err(e) = write.send(msg).await {
                                                log::error!("send error: {e}");
                                                break;
                                            }
                                        }
                                    });

                                    while let Some(msg) = read.next().await {
                                        match msg {
                                            Ok(Message::Text(txt)) => match serde_json::de::from_str::<BrowserChange>(&txt) {
                                                Ok(change) => log::info!("{change:?}"),
                                                Err(err) => {
                                                    log::error!("failed to deserialize browser change: {err}");
                                                }
                                            },
                                            Ok(Message::Close(_)) => break,
                                            Ok(_) => {},
                                            Err(e) => {
                                                log::error!("read error: {e}");
                                                break;
                                            }
                                        }
                                    }

                                    forward.abort();
                                    let mut lock = this.sender.lock().await;
                                    *lock = None;
                                } else if header.starts_with("GET / HTTP/1.1") {
                                    let body = json!({
                                        "ProtocolVersion": 1,
                                        "WebSocketPort": Self::PORT
                                    }).to_string();

                                    let response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                                        body.len()
                                    );

                                    let _ = stream.write_all(response.as_bytes()).await;
                                }
                            });
                        }
                    }
                }

                #[allow(unreachable_code, reason = "todo")]
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            } => {},

            _ = &mut shutdown => {
                log::info!("Shutdown signal received");
            }
        }

        Ok(())
    }

    /// Initializes logging to `out.log` in the project root.
    #[allow(clippy::missing_panics_doc, reason = "todo")]
    pub fn init_logging() {
        static INIT: OnceLock<()> = OnceLock::new();
        static FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        INIT.get_or_init(|| {
            let file = std::fs::File::options()
                .create(true)
                .append(true)
                .open("out.log")
                .expect("Failed to open out.log");

            FILE.set(Mutex::new(file)).expect("");

            let logger = env_logger::Builder::new()
                .format(|buf, record| {
                    let ts = buf.timestamp();
                    let mut file = FILE.get().expect("").lock().expect("");
                    writeln!(file, "[{}] {}: {}", ts, record.level(), record.args())
                })
                .filter(None, log::LevelFilter::Info)
                .build();

            log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger");
            log::set_max_level(log::LevelFilter::Info);
        });
    }

    /// Update the Ghost Text server
    pub fn update(self, text: String, selections: Vec<Vec<usize>>) {
        let change = EditorChange {
            text,
            selections: selections
                .into_iter()
                .map(|sel| {
                    let [start, end] = sel[..] else {
                        unreachable!()
                    };

                    Selection { start, end }
                })
                .collect(),
        };

        if let Some(tx) = self.change_tx.lock().expect("not to fail").as_ref() {
            let _ = tx.send(change);
        }
    }
}
