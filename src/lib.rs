//! Client: GhostText browser extension
//! Server: This plugin

use serde::{Deserialize, Serialize};

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

const PORT: u32 = 4001;
const PROTOCOL_VERSION: u32 = 1;

struct Client;

impl Client {
    pub fn activate_editing() -> ActivateEditing {
        ActivateEditing {
            protocol_version: PROTOCOL_VERSION,
            web_socket_port: PORT,
        }
    }
}
