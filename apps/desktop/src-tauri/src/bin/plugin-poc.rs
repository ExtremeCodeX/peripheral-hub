#[path = "../passive_usb.rs"]
mod passive_usb;
#[path = "../plugin_host.rs"]
mod plugin_host;
use plugin_host::{DevAction, DevHost};

fn main() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    // This branch never constructs the host or starts a Python process.
    if args.as_slice() == ["--passive-usb"] {
        println!(
            "{}",
            serde_json::to_string(&passive_usb::enumerate()).map_err(|_| "SERIALIZATION_FAILED")?
        );
        return Ok(());
    }
    // Diagnostic local harness: passive discovery/metadata only for hardware.
    // stdin authorization flags cannot replace a native host confirmation, so
    // physical open/read/write fail closed here. Simulators are unaffected.
    if args.as_slice() == ["--local-r1-hardware-rpc"] {
        use std::io::{BufRead, Write};
        let mut host = DevHost::default();
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        loop {
            let mut line = String::new();
            let bytes = std::io::Read::take(&mut reader, 262146)
                .read_line(&mut line)
                .map_err(|_| "INPUT_FAILED")?;
            if bytes == 0 {
                break;
            }
            if !line.ends_with('\n') || bytes > 262145 {
                return Err("MESSAGE_LIMIT".into());
            }
            let result = serde_json::from_str::<DevAction>(&line)
                .map_err(|_| "INVALID_REQUEST".into())
                .and_then(|action| host.execute(action));
            let reply = match result {
                Ok(result) => serde_json::json!({"result": result}),
                Err(error) => serde_json::json!({"error": plugin_host::error_detail(error)}),
            };
            println!("{reply}");
            std::io::stdout().flush().map_err(|_| "OUTPUT_FAILED")?;
        }
        host.execute(DevAction::Stop {})?;
        return Ok(());
    }
    let plugin_id =
        match args.as_slice() {
            [flag] if flag == "--dev-fake" => "dev.fake",
            [flag] if flag == "--local-r1-simulated" => "local.r1.simulated",
            _ => return Err(
                "Use --passive-usb, --dev-fake, --local-r1-simulated or --local-r1-hardware-rpc; no arbitrary paths/args"
                    .into(),
            ),
        };
    let mut host = DevHost::default();
    println!(
        "{}",
        host.execute(DevAction::Start {
            plugin_id: plugin_id.into(),
            development: true
        })?
    );
    println!("{}", host.execute(DevAction::Capabilities {})?);
    let opened = host.execute(DevAction::Open {})?;
    let session = opened["session"].as_str().unwrap().to_owned();
    println!("{opened}");
    let state = host.execute(DevAction::Read {
        session: session.clone(),
    })?;
    println!("{state}");
    println!(
        "{}",
        host.execute(DevAction::Set {
            session: session.clone(),
            baseline_token: state
                .get("baselineTokens")
                .map(|t| &t["polling_rate"])
                .unwrap_or(&state["baselineToken"])
                .as_str()
                .unwrap()
                .into(),
            key: "polling_rate".into(),
            value: serde_json::json!(1000)
        })?
    );
    println!("{}", host.execute(DevAction::Close { session })?);
    println!("{}", host.execute(DevAction::Stop {})?);
    Ok(())
}
