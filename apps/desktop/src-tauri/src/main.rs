//! Restricted Tauri bridge for the Peripheral Hub desktop app.
//!
//! This is the only route from the React UI to the daemon. The frontend cannot
//! access USB/HID or the local socket directly; it submits a typed `Operation`
//! and receives the daemon's versioned `Response`. A fresh authorized local IPC
//! connection is used per call, matching the daemon's reconnect model.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use peripheral_hub_ipc::{Client, Endpoint, IpcError, Operation, Response};
use std::time::Duration;

/// Client deadline for one desktop-initiated operation.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

/// Map a local transport/codec error to a stable, non-sensitive UI message.
fn bridge_message(error: &IpcError) -> String {
    match error {
        IpcError::Timeout => {
            "The daemon did not answer before the deadline. The operation may have an unknown outcome and was not retried.".to_string()
        }
        IpcError::UnexpectedResponse => {
            "The daemon sent an unexpected response; versions may be incompatible.".to_string()
        }
        IpcError::InvalidMessage | IpcError::InvalidLength => {
            "The local IPC message was rejected by the codec.".to_string()
        }
        IpcError::Io(_) => {
            "No local daemon is reachable. Start it with `peripheral-hub-daemon serve`.".to_string()
        }
    }
}

/// The single allowed frontend command: one typed, bounded daemon call.
#[tauri::command]
async fn daemon_call(operation: Operation) -> Result<Response, String> {
    let endpoint =
        Endpoint::default_for_user().map_err(|_| "The local endpoint is unavailable.".to_string())?;
    let mut client = Client::new(endpoint, CLIENT_TIMEOUT)
        .map_err(|_| "The IPC client rejected the configured deadline.".to_string())?;
    client
        .call(operation)
        .await
        .map_err(|error| bridge_message(&error))
}

/// Build and run the desktop window.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![daemon_call])
        .run(tauri::generate_context!())
        .expect("failed to run the Peripheral Hub desktop application");
}

fn main() {
    run();
}
