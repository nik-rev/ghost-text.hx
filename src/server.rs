//! The Ghost Text server

use std::io::Write as _;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use crate::data::{BrowserChange, EditorChange, Selection};
use abi_stable::std_types::{RSliceMut, RString};
use futures_util::{SinkExt as _, StreamExt as _};

use serde_json::json;
use steel::rvals::Custom;
use steel::steel_vm::ffi::{FFIValue, HostRuntimeFunction};
use tokio::net::TcpStream;
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
    /// Kill the WebSocket connection
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    /// When we edit text in the browser, update contents of the helix buffer
    change_tx: Arc<Mutex<Option<UnboundedSender<EditorChange>>>>,
}

static WRITE_FOCUSED_BUFFER: OnceLock<HostRuntimeFunction> = OnceLock::new();

/// FFI Function to update contents of the Helix buffer
///
/// Receives a `String` corresponding to contents of the new file
pub fn register_helix_buffer(host: HostRuntimeFunction) {
    WRITE_FOCUSED_BUFFER
        .set(host)
        .ok()
        .expect("cell is not initialized");
}

impl Custom for Server {}

impl Server {
    /// WebSocket port on which we communicate with the GhostText server
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

    /// Start the server
    pub fn start(&self) {
        let this = self.clone();

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (change_tx, change_rx) = mpsc::unbounded_channel();

        *self.change_tx.lock().expect("mutex not acquired") = Some(change_tx);

        let thread = thread::spawn(move || {
            let rt = Runtime::new().expect("failed to create Tokio runtime");
            rt.block_on(async move {
                if let Err(e) = this.run(shutdown_rx, change_rx).await {
                    log::error!("Server error: {e}");
                }
            });
        });

        *self.shutdown_tx.lock().expect("mutex not acquired") = Some(shutdown_tx);
        *self.thread_handle.lock().expect("mutex not acquired") = Some(thread);
    }

    /// Stop the Ghost Text server
    pub fn stop(&self) {
        let sender = self.shutdown_tx.lock().expect("mutex not acquired").take();
        if let Some(tx) = sender {
            let _ = tx.send(());
        }

        let join_handle = self
            .thread_handle
            .lock()
            .expect("mutex not acquired")
            .take();

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
                        Ok((stream, addr)) = listener.accept() => {
                            self.clone().accept_connection(stream, addr);
                        }
                    }
                }

                #[expect(unreachable_code, reason = "for type inference")]
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            } => {},

            _ = &mut shutdown => {
                log::info!("Shutdown signal received");
            }
        }

        Ok(())
    }

    /// Accept websocket connection
    fn accept_connection(self, mut stream: TcpStream, addr: SocketAddr) {
        tokio::spawn(async move {
            let mut buf = [0_u8; 1024];
            let bytes_to_read = match stream.peek(&mut buf).await {
                Ok(n) => n,
                Err(err) => {
                    log::error!("buffer peek failed: {err}");
                    return;
                }
            };

            let header = String::from_utf8_lossy(&buf[..bytes_to_read]);

            if header.contains("Upgrade: websocket") {
                let web_socket_stream =
                    match accept_hdr_async(stream, |request: &Request, response| {
                        log::info!("WebSocket request from: {:?}", request.uri());
                        Ok(response)
                    })
                    .await
                    {
                        Ok(ws) => ws,
                        Err(err) => {
                            log::error!("WebSocket upgrade failed: {err}");
                            return;
                        }
                    };

                log::info!("WebSocket connection from {addr}");

                let (mut web_socket_write, mut web_socket_read) = web_socket_stream.split();
                let (tx, mut rx) = mpsc::unbounded_channel();

                {
                    let mut lock = self.sender.lock().await;
                    *lock = Some(tx);
                }

                let forward = tokio::spawn(async move {
                    while let Some(message) = rx.recv().await {
                        if let Err(err) = web_socket_write.send(message).await {
                            log::error!("send error: {err}");
                            break;
                        }
                    }
                });

                while let Some(message) = web_socket_read.next().await {
                    match message {
                        Ok(Message::Text(browser_change)) => {
                            match serde_json::de::from_str::<BrowserChange>(&browser_change) {
                                Ok(browser_change) => {
                                    WRITE_FOCUSED_BUFFER
                                        .get()
                                        .expect("cell to be initialized")
                                        .call(RSliceMut::from_mut_slice(&mut [FFIValue::StringV(
                                            RString::from(browser_change.text),
                                        )]))
                                        .unwrap();
                                }
                                Err(err) => {
                                    log::error!(
                                        "failed to deserialize change from the browser: {err}"
                                    );
                                }
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("failed to read message: {err}");
                            break;
                        }
                    }
                }

                forward.abort();
                let mut lock = self.sender.lock().await;
                *lock = None;
            } else if header.starts_with("GET / HTTP/1.1") {
                // Handshake.

                let body = json!({
                    "ProtocolVersion": 1,
                    "WebSocketPort": Self::PORT
                })
                .to_string();

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );

                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
    }

    /// Initializes logging to `out.log` in the project root.
    pub fn init_logging() {
        static INIT: OnceLock<()> = OnceLock::new();
        static FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

        INIT.get_or_init(|| {
            let file = std::fs::File::options()
                .create(true)
                .append(true)
                .open("out.log")
                .expect("failed to open log file");

            FILE.set(Mutex::new(file))
                .expect("cell not to be initialized");

            let logger = env_logger::Builder::new()
                .format(|buf, record| {
                    let ts = buf.timestamp();
                    let mut file = FILE
                        .get()
                        .expect("cell to be initialized")
                        .lock()
                        .expect("cell not to be already held by the current thread");
                    writeln!(file, "[{}] {}: {}", ts, record.level(), record.args())
                })
                .filter(None, log::LevelFilter::Info)
                .build();

            log::set_boxed_logger(Box::new(logger)).expect("failed to set logger");
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
                        unreachable!(
                            "\
1. `range->span` returns [start, end]
2. Steel has no concept of tuple
3. We cannot pass tuple through FFI"
                        )
                    };

                    Selection { start, end }
                })
                .collect(),
        };

        if let Some(tx) = self
            .change_tx
            .lock()
            .expect("mutex to be available")
            .as_ref()
        {
            let _ = tx.send(change);
        }
    }
}
