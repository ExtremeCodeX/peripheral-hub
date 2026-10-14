//! User-session daemon and typed diagnostic client. No hardware report CLI exists.

use peripheral_hub_daemon::*;
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_transport_hid::{DeviceSource, HidDiscovery, simulated::SimulatedSource};
use std::time::Duration;

fn help() {
    println!(
        "Peripheral Hub daemon\n\
      serve [--simulate] [--endpoint PATH]\n\
      enumerate                         Non-destructive physical HID inventory\n\
      health|list|events [--endpoint PATH]\n\
      get|connect|disconnect|capabilities|state ID [--endpoint PATH]\n\
      lighting ID R G B [--endpoint PATH]  Typed request; hardware needs a reviewed driver\n\
      simulate-present|simulate-absent [--endpoint PATH]\n\
      --help\n\
      Simulation is explicit software-only; no physical driver is bundled."
    );
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(std::io::stderr)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        help();
        return Ok(());
    }
    let endpoint = if let Some(index) = args.iter().position(|arg| arg == "--endpoint") {
        if index + 1 >= args.len() {
            return Err("--endpoint needs a local path".into());
        }
        let path = args.remove(index + 1);
        args.remove(index);
        Some(Endpoint::new(path))
    } else {
        None
    };
    if args.is_empty() {
        return Err("missing command; see --help".into());
    }
    let command = args.remove(0);
    if command == "enumerate" {
        if !args.is_empty() || endpoint.is_some() {
            return Err("enumerate accepts no extra arguments".into());
        }
        let mut manager =
            DeviceManager::try_new(DriverRegistry::new(), Box::new(HidDiscovery::new()?))?;
        manager.refresh()?;
        println!(
            "{}",
            serde_json::to_string_pretty(&manager.handle(Operation::ListDevices)?)?
        );
        return Ok(());
    }
    let endpoint = match endpoint {
        Some(endpoint) => endpoint,
        None => Endpoint::default_for_user()?,
    };
    if command == "serve" {
        let simulated = args == ["--simulate"];
        if !args.is_empty() && !simulated {
            return Err("unknown serve arguments; see --help".into());
        }
        // Claim the endpoint before initializing a backend or spawning a manager.
        let listener = Listener::bind(&endpoint).await?;
        let (registry, source): (DriverRegistry, Box<dyn DeviceSource>) = if simulated {
            (simulated_registry(), Box::new(SimulatedSource::new()?))
        } else {
            let source: Box<dyn DeviceSource> = match HidDiscovery::new() {
                Ok(source) => Box::new(source),
                Err(error) => Box::new(UnavailableSource::new(error)),
            };
            (DriverRegistry::new(), source)
        };
        let manager = ManagerHandle::start(
            DeviceManager::try_new(registry, source)?,
            Duration::from_secs(2),
        )?;
        tracing::info!(event = "daemon_started", simulated);
        #[cfg(unix)]
        let mut termination =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        serve(listener, manager, ServerConfig::default(), async move {
            #[cfg(unix)]
            tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = termination.recv() => {} }
            #[cfg(not(unix))]
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
        return Ok(());
    }
    let operation = match (command.as_str(), args.as_slice()) {
        ("health", []) => Operation::Health,
        ("list", []) => Operation::ListDevices,
        ("events", []) => Operation::Events { after: 0 },
        ("simulate-present", []) => Operation::SimulatePresence { present: true },
        ("simulate-absent", []) => Operation::SimulatePresence { present: false },
        (operation @ ("get" | "connect" | "disconnect" | "capabilities" | "state"), [id]) => {
            let interface_id = DeviceInterfaceId::new(id.clone())?;
            match operation {
                "get" => Operation::GetDevice { interface_id },
                "connect" => Operation::Connect { interface_id },
                "disconnect" => Operation::Disconnect { interface_id },
                "capabilities" => Operation::GetCapabilities { interface_id },
                _ => Operation::ReadState { interface_id },
            }
        }
        ("lighting", [id, red, green, blue]) => Operation::SetLighting {
            interface_id: DeviceInterfaceId::new(id.clone())?,
            command: LightingCommand {
                zone: "whole-device".into(),
                color: RgbColor {
                    red: red.parse()?,
                    green: green.parse()?,
                    blue: blue.parse()?,
                },
                brightness: None,
                effect: LightingEffect::Static,
            },
        },
        _ => return Err("unknown command or invalid arguments; see --help".into()),
    };
    let response = Client::new(endpoint, Duration::from_secs(5))?
        .call(operation)
        .await?;
    let failed = response.result.is_err();
    println!("{}", serde_json::to_string_pretty(&response)?);
    if failed {
        return Err("daemon returned a structured failure (see JSON)".into());
    }
    Ok(())
}
