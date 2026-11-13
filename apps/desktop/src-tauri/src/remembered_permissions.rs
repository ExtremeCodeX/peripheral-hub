//! Host-owned opt-in grants for execution and connection/read only, never writes.
use crate::plugin_host::PhysicalConfirmation;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;

pub fn scope(operation: &PhysicalConfirmation) -> Option<String> {
    if !matches!(
        operation.method.as_str(),
        "process.start.dev" | "process.start.public" | "device.open" | "device.get_state"
    ) {
        return None;
    }
    let hash = operation.params["sha256"].as_str()?;
    let mode = operation.params["mode"].as_str()?;
    if hash.len() != 64
        || !hash.bytes().all(|c| c.is_ascii_hexdigit())
        || !matches!(mode, "hardware" | "fake")
    {
        return None;
    }
    // JSON encodes component boundaries; a grant cannot collide across identities.
    let identity = serde_json::to_vec(&(&operation.plugin_id, hash, mode)).ok()?;
    Some(format!("{:x}", Sha256::digest(identity)))
}

fn load(path: &Path) -> HashSet<String> {
    if !std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() <= 65536)
    {
        return HashSet::new();
    }
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn contains(path: &Path, scope: &str) -> bool {
    load(path).contains(scope)
}

pub fn save(path: &Path, scope: String) -> Result<(), String> {
    let mut grants = load(path);
    grants.insert(scope);
    let parent = path.parent().ok_or("PERMISSION_STORAGE_FAILED")?;
    std::fs::create_dir_all(parent).map_err(|_| "PERMISSION_STORAGE_FAILED")?;
    let bytes = serde_json::to_vec(&grants).map_err(|_| "PERMISSION_STORAGE_FAILED")?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|_| "PERMISSION_STORAGE_FAILED")?;
    let result = (|| {
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "PERMISSION_STORAGE_FAILED")?;
        std::fs::rename(&temporary, path).map_err(|_| "PERMISSION_STORAGE_FAILED")
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(str::to_string)
}

pub fn clear(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("PERMISSION_STORAGE_FAILED".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn operation() -> PhysicalConfirmation {
        PhysicalConfirmation {
            generation: 1,
            operation: 0,
            plugin_id: "dev.mouse".into(),
            method: "process.start.dev".into(),
            params: json!({"sha256":"a".repeat(64), "mode":"hardware"}),
        }
    }
    #[test]
    fn grants_bind_snapshot_identity_and_mode_but_not_connection_generation() {
        let initial = operation();
        let key = scope(&initial).unwrap();
        for method in ["process.start.dev", "device.open", "device.get_state"] {
            let mut next = initial.clone();
            next.method = method.into();
            next.generation = 2;
            assert_eq!(scope(&next), Some(key.clone()));
        }
        let mut changed = initial.clone();
        changed.params["sha256"] = json!("b".repeat(64));
        assert_ne!(scope(&changed), Some(key.clone()));
        changed = initial.clone();
        changed.plugin_id = "dev.other".into();
        assert_ne!(scope(&changed), Some(key.clone()));
        changed = initial.clone();
        changed.params["mode"] = json!("fake");
        assert_ne!(scope(&changed), Some(key));
        for method in ["device.set_setting", "unknown"] {
            changed.method = method.into();
            assert_eq!(scope(&changed), None);
        }
        changed.method = "device.open".into();
        changed.params = json!({});
        assert_eq!(scope(&changed), None);
    }
    #[test]
    fn persistence_revocation_and_corrupt_file_fail_closed() {
        let path = std::env::temp_dir().join(format!("hub-grants-{}.json", std::process::id()));
        clear(&path).unwrap();
        assert!(!contains(&path, "test"));
        save(&path, "test".into()).unwrap();
        assert!(contains(&path, "test"));
        assert!(!contains(&path, "other"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        clear(&path).unwrap();
        assert!(!contains(&path, "test"));
        std::fs::write(&path, b"invalid").unwrap();
        assert!(!contains(&path, "test"));
        clear(&path).unwrap();
    }
}
