//! Closed stdin/stdout development bridge for offline E2E; no arbitrary ID/path.
#[path = "../plugin_host.rs"]
mod plugin_host;
#[path = "../starter_host.rs"]
mod starter_host;

fn main() -> Result<(), String> {
    use std::io::{BufRead, Write};
    if !cfg!(debug_assertions) {
        return Err("DEV_ONLY".into());
    }
    if std::env::args().skip(1).collect::<Vec<_>>() != ["--rpc"] {
        return Err("Use --rpc".into());
    }
    let mut host = starter_host::StarterHost::default();
    let mut reader = std::io::stdin().lock();
    loop {
        let mut line = String::new();
        let bytes = std::io::Read::take(&mut reader, 16386)
            .read_line(&mut line)
            .map_err(|_| "INPUT_FAILED")?;
        if bytes == 0 {
            break;
        }
        if !line.ends_with('\n') || bytes > 16385 {
            return Err("MESSAGE_LIMIT".into());
        }
        let result = serde_json::from_str::<starter_host::StarterAction>(&line)
            .map_err(|_| "INVALID_REQUEST".into())
            .and_then(|request| host.execute(request));
        let reply = match result {
            Ok(result) => serde_json::json!({"result":result}),
            Err(error) => serde_json::json!({"error":plugin_host::error_detail(error)}),
        };
        println!("{reply}");
        std::io::stdout().flush().map_err(|_| "OUTPUT_FAILED")?;
    }
    host.execute(starter_host::StarterAction::Stop {})?;
    Ok(())
}
