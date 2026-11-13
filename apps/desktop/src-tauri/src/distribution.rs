use serde::Deserialize;
use serde_json::{json, Value};

pub fn registry() -> Value {
    serde_json::from_str(include_str!("../../../../catalog/installable.json"))
        .expect("reviewed distribution registry")
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    List {},
    Install { plugin_id: String },
}

pub fn execute(request: Request) -> Result<Value, String> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _lock = LOCK.try_lock().map_err(|_| "BUSY")?;
    if !cfg!(target_os = "linux") {
        return Err("LINUX_REQUIRED".into());
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|p| std::path::PathBuf::from(p).join(".local/share"))
        })
        .ok_or("INSTALLATION_UNAVAILABLE")?;
    let (action, id) = match request {
        Request::List {} => ("list", String::new()),
        Request::Install { plugin_id } => ("install", plugin_id),
    };
    let input = serde_json::to_vec(
        &json!({"action": action, "id": id, "base": base, "registry": registry()}),
    )
    .map_err(|_| "INVALID_PACKAGE")?;
    let script = format!("{}\nworker_main()", include_str!("distribution_worker.py"));
    let result = crate::catalog::run_worker(&script, &input, std::time::Duration::from_secs(35))?;
    if let Some(error) = result["error"].as_str() {
        return Err(error.into());
    }
    Ok(result)
}
