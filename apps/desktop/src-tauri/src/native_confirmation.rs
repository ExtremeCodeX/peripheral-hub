//! Native host-owned confirmation. No HTML, IPC approval endpoint or token.
use crate::plugin_host::PhysicalConfirmation;
use std::sync::Mutex;
use tauri::Manager;
static PERMISSIONS_LOCK: Mutex<()> = Mutex::new(());

fn permissions_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|path| path.join("connection-permissions.json"))
        .map_err(|_| "PERMISSION_STORAGE_FAILED".into())
}

pub fn forget(app: &tauri::AppHandle) -> Result<(), String> {
    let _guard = PERMISSIONS_LOCK
        .lock()
        .map_err(|_| "PERMISSION_STORAGE_FAILED")?;
    crate::remembered_permissions::clear(&permissions_path(app)?)
}
#[cfg(target_os = "linux")]
#[path = "native_dialog.rs"]
mod native_dialog;

#[cfg(any(target_os = "linux", test))]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
#[cfg(any(target_os = "linux", test))]
use std::time::{Duration, Instant};

#[cfg(any(target_os = "linux", test))]
fn deliver_decision(
    sender: &mpsc::SyncSender<Result<(), String>>,
    pending: &AtomicBool,
    result: Result<(), String>,
) {
    if pending.swap(false, Ordering::SeqCst) {
        let _ = sender.try_send(result);
    }
}

#[cfg(any(target_os = "linux", test))]
fn wait_for_decision(
    receiver: mpsc::Receiver<Result<(), String>>,
    pending: &AtomicBool,
    timeout: Duration,
) -> Result<(), String> {
    let result = receiver
        .recv_timeout(timeout)
        .map_err(|_| "NATIVE_CONFIRMATION_UNAVAILABLE".to_string());
    pending.store(false, Ordering::SeqCst);
    result?
}

#[cfg(target_os = "linux")]
pub fn confirm(
    window: tauri::WebviewWindow,
    operation: PhysicalConfirmation,
) -> Result<(), String> {
    let scope = crate::remembered_permissions::scope(&operation);
    let path = permissions_path(window.app_handle())?;
    if let Some(ref scope) = scope {
        let _guard = PERMISSIONS_LOCK
            .lock()
            .map_err(|_| "PERMISSION_STORAGE_FAILED")?;
        if crate::remembered_permissions::contains(&path, scope) {
            return Ok(());
        }
    }
    let allow_remember = scope.is_some() && operation.method.starts_with("process.start.");
    let remembered = Arc::new(AtomicBool::new(false));
    let remember_choice = remembered.clone();
    // A fresh channel per operation prevents late responses from approving a
    // later request. The host mutex remains held on the blocking worker thread.
    let (sender, receiver) = mpsc::sync_channel(1);
    let pending = Arc::new(AtomicBool::new(true));
    let active = pending.clone();
    let deadline = Instant::now() + Duration::from_secs(120);
    let parent = window.clone();
    window
        .run_on_main_thread(move || {
            if !active.load(Ordering::SeqCst) {
                return; // queued dialog expired before the GTK thread could run it
            }
            if Instant::now() >= deadline {
                deliver_decision(
                    &sender,
                    &active,
                    Err("NATIVE_CONFIRMATION_UNAVAILABLE".into()),
                );
                return;
            }
            let Ok(parent) = parent.gtk_window() else {
                deliver_decision(
                    &sender,
                    &active,
                    Err("NATIVE_CONFIRMATION_UNAVAILABLE".into()),
                );
                return;
            };
            let result = native_dialog::show(
                &parent,
                &operation.description(),
                deadline,
                active.clone(),
                allow_remember,
            )
            .map(|remember| {
                remember_choice.store(remember, Ordering::SeqCst);
            });
            deliver_decision(&sender, &active, result);
        })
        .map_err(|_| "NATIVE_CONFIRMATION_UNAVAILABLE".to_string())?;

    wait_for_decision(receiver, &pending, Duration::from_secs(125))?;
    if remembered.load(Ordering::SeqCst) {
        if let Some(scope) = scope {
            let _guard = PERMISSIONS_LOCK
                .lock()
                .map_err(|_| "PERMISSION_STORAGE_FAILED")?;
            crate::remembered_permissions::save(&path, scope)?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn confirm(
    _window: tauri::WebviewWindow,
    _operation: PhysicalConfirmation,
) -> Result<(), String> {
    // No fallback to renderer authorization on unsupported platforms.
    Err("NATIVE_CONFIRMATION_UNAVAILABLE".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_decision_accept_and_cancel_use_only_their_own_channel() {
        for decision in [Ok(()), Err("NATIVE_CONFIRMATION_DENIED".into())] {
            let pending = AtomicBool::new(true);
            let (sender, receiver) = mpsc::sync_channel(1);
            deliver_decision(&sender, &pending, decision.clone());
            assert_eq!(
                wait_for_decision(receiver, &pending, Duration::from_millis(20)),
                decision
            );
            assert!(!pending.load(Ordering::SeqCst));
        }
    }

    #[test]
    fn native_decision_timeout_rejects_late_approval_and_cannot_approve_next_request() {
        let old_pending = AtomicBool::new(true);
        let (old_sender, old_receiver) = mpsc::sync_channel(1);
        assert_eq!(
            wait_for_decision(old_receiver, &old_pending, Duration::from_millis(10)),
            Err("NATIVE_CONFIRMATION_UNAVAILABLE".into())
        );
        assert!(!old_pending.load(Ordering::SeqCst));
        let pending = AtomicBool::new(true);
        let (_sender, receiver) = mpsc::sync_channel(1);
        deliver_decision(&old_sender, &old_pending, Ok(()));
        assert_eq!(
            wait_for_decision(receiver, &pending, Duration::from_millis(10)),
            Err("NATIVE_CONFIRMATION_UNAVAILABLE".into())
        );
    }

    #[test]
    fn native_decision_destroyed_or_unavailable_provider_fails_closed() {
        let pending = AtomicBool::new(true);
        let (sender, receiver) = mpsc::sync_channel(1);
        drop(sender);
        assert_eq!(
            wait_for_decision(receiver, &pending, Duration::from_millis(20)),
            Err("NATIVE_CONFIRMATION_UNAVAILABLE".into())
        );
        assert!(!pending.load(Ordering::SeqCst));
    }

    #[test]
    fn native_decision_cancellation_cannot_be_replaced_with_later_accept() {
        let pending = AtomicBool::new(true);
        let (sender, receiver) = mpsc::sync_channel(1);
        deliver_decision(&sender, &pending, Err("NATIVE_CONFIRMATION_DENIED".into()));
        deliver_decision(&sender, &pending, Ok(()));
        assert_eq!(
            wait_for_decision(receiver, &pending, Duration::from_millis(20)),
            Err("NATIVE_CONFIRMATION_DENIED".into())
        );
    }
}
