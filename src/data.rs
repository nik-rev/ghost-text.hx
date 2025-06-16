//! Data structures for the Ghost Text protocol

use serde::{Deserialize, Serialize};

/// We send this response to the browser extension to establish a connection
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivateEditing {
    /// The protocol version
    pub protocol_version: u32,
    /// The port for the listening WebSocket
    ///
    /// This ideally is the same configured HTTP port (default 4001) but
    /// it does not have to be.
    pub web_socket_port: u32,
}

/// Represents a selected region of text
#[derive(Serialize, Deserialize, Debug)]
pub struct Selection {
    /// 0-indexed start of the selection
    pub start: usize,
    /// 0-indexed end of the selection
    pub end: usize,
}

/// User makes a change in the browser
#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserChange {
    /// The title of the document
    pub title: String,
    /// The host of the document's url
    pub url: String,
    /// Not used
    pub syntax: String,
    /// Value of the text content
    pub text: String,
    /// User's selections in the browser
    pub selections: Vec<Selection>,
}

/// User makes a change in the editor
#[derive(Serialize, Deserialize)]
pub struct EditorChange {
    /// The temporary file content
    pub text: String,
    /// User's selections in the browser
    pub selections: Vec<Selection>,
}
