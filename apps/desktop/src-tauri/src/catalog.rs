use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tauri::Manager;

static SYNC_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Independent metadata worker. No access to DevHost, plugin identities or HID.
pub fn sync(app: tauri::AppHandle, refresh: bool) -> Result<serde_json::Value, String> {
    let _guard = SYNC_LOCK.try_lock().map_err(|_| "BUSY")?;
    let url = option_env!("PERIPHERAL_HUB_CATALOG_URL").unwrap_or("");
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|_| "CACHE_ERROR")?
        .join("catalog-v1.json");
    let script = worker_script()?;
    let input = serde_json::to_vec(&serde_json::json!({
        "url": url, "cache": cache, "refresh": refresh,
        "schema": serde_json::from_str::<serde_json::Value>(include_str!("../../../../schemas/community-publication-v2.schema.json")).map_err(|_| "WORKER_ERROR")?
    })).map_err(|_| "WORKER_ERROR")?;
    run_worker(&script, &input, Duration::from_secs(15))
}

fn worker_script() -> Result<String, String> {
    let validator =
        serde_json::to_string(include_str!("../../../../tools/validate-publication.py"))
            .map_err(|_| "WORKER_ERROR")?;
    Ok(format!("import types\nvalidator=types.ModuleType('publication')\nvalidator.__file__='publication.py'\nexec({validator},validator.__dict__)\n{}\nworker_main(validator)", include_str!("catalog_worker.py")))
}

fn worker_command(script: &str) -> Command {
    // Match the local plugin host: no PATH lookup or inherited Python/proxy/
    // certificate environment. Cache and URL are explicit, validated inputs.
    let mut command = Command::new("/usr/bin/python3");
    command
        .env_clear()
        .current_dir("/")
        .args(["-I", "-S", "-B", "-c", script]);
    command
}

fn run_worker(script: &str, input: &[u8], timeout: Duration) -> Result<serde_json::Value, String> {
    let mut child = worker_command(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "PYTHON_UNAVAILABLE")?;
    let mut stdin = child.stdin.take().ok_or("WORKER_ERROR")?;
    let input = input.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let stdout = child.stdout.take().ok_or("WORKER_ERROR")?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .take(2 * 1024 * 1024 + 65537)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = writer.join();
                let _ = reader.join();
                return Err("TIMEOUT".into());
            }
        }
    }
    writer
        .join()
        .map_err(|_| "WORKER_ERROR")?
        .map_err(|_| "WORKER_ERROR")?;
    let bytes = reader
        .join()
        .map_err(|_| "WORKER_ERROR")?
        .map_err(|_| "WORKER_ERROR")?;
    if bytes.len() > 2 * 1024 * 1024 + 65536 {
        return Err("TOO_LARGE".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "WORKER_ERROR".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn worker_uses_absolute_interpreter_and_clean_environment() {
        let command = super::worker_command("pass");
        assert_eq!(command.get_program(), "/usr/bin/python3");
        assert_eq!(command.get_current_dir(), Some(std::path::Path::new("/")));
        assert_eq!(command.get_envs().count(), 0);
        let result = super::run_worker(
            "import json,os,sys; print(json.dumps({'executable':sys.executable,'env':sorted(os.environ),'cwd':os.getcwd(),'isolated':sys.flags.isolated}))",
            b"",
            std::time::Duration::from_secs(2),
        ).unwrap();
        assert_eq!(result["executable"], "/usr/bin/python3");
        assert_eq!(result["cwd"], "/");
        assert_eq!(result["isolated"], 1);
        // CPython may inject LC_CTYPE for locale coercion, but nothing else.
        assert!(result["env"]
            .as_array()
            .unwrap()
            .iter()
            .all(|k| k == "LC_CTYPE"));
    }

    #[test]
    fn supervisor_terminates_slow_and_excessive_workers() {
        use super::run_worker;
        use std::time::{Duration, Instant};
        let start = Instant::now();
        assert_eq!(
            run_worker(
                "import time; time.sleep(10)",
                &vec![b'x'; 65536],
                Duration::from_millis(100)
            ),
            Err("TIMEOUT".into())
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        assert_eq!(
            run_worker(
                "print('x' * (3 * 1024 * 1024))",
                b"",
                Duration::from_secs(2)
            ),
            Err("TOO_LARGE".into())
        );
    }
    #[test]
    fn embedded_bootstrap_runs_without_network_or_hardware() {
        use std::io::Write;
        use std::process::Stdio;
        let script = super::worker_script().unwrap();
        let mut child = super::worker_command(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(br#"{"url":"","cache":"unused","refresh":true,"schema":{}}"#)
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["status"], "NOT_CONFIGURED");
        assert!(value["index"].is_null());
    }

    #[test]
    fn embedded_worker_validates_real_schema_and_reads_cache_offline() {
        let path = std::path::PathBuf::from(format!(
            "/tmp/opencode/catalog-rust-{}.json",
            std::process::id()
        ));
        let index: serde_json::Value =
            serde_json::from_str(include_str!("../../../../catalog/v1/index.json")).unwrap();
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../schemas/community-publication-v2.schema.json"
        ))
        .unwrap();
        let url = "https://example.org/fixture/v1/index.json";
        let saved = serde_json::json!({"url":url,"etag":"\"fixture\"","checkedAt":1,"index":index});
        std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
        let input = serde_json::to_vec(
            &serde_json::json!({"url":url,"cache":path,"refresh":false,"schema":schema}),
        )
        .unwrap();
        let result = super::run_worker(
            &super::worker_script().unwrap(),
            &input,
            std::time::Duration::from_secs(2),
        );
        std::fs::remove_file(&path).unwrap();
        let result = result.unwrap();
        assert_eq!(result["status"], "CACHED");
        assert_eq!(result["index"], index);
        assert!(result["cacheWarning"].is_null());
    }
}
