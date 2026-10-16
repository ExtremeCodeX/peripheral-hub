//! User-session daemon and typed diagnostic client. No hardware report CLI exists.

use peripheral_hub_daemon::*;
use peripheral_hub_device_registry::DriverRegistry;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_profiles::{Profile, ProfileId, ProfileRepository};
use peripheral_hub_transport_hid::{DeviceSource, HidDiscovery, simulated::SimulatedSource};
use std::path::PathBuf;
use std::time::Duration;

fn help() {
    println!(
        "Peripheral Hub daemon\n\
      serve [--simulate] [--endpoint PATH] [--profiles PATH] [--manifests DIR]\n\
      enumerate                         Non-destructive physical HID inventory\n\
      health|list|events [--endpoint PATH]\n\
      get|connect|disconnect|capabilities|state ID [--endpoint PATH]\n\
      lighting ID R G B [--endpoint PATH]  Typed request; hardware needs a reviewed driver\n\
      simulate-present|simulate-absent [--endpoint PATH]\n\
      profiles [--endpoint PATH]        List stored profile summaries\n\
      profile-get ID [--endpoint PATH]\n\
      profile-save ID NAME [R G B] [--endpoint PATH]\n\
      profile-delete ID [--endpoint PATH]\n\
      profile-apply ID INTERFACE_ID [--endpoint PATH]  Apply a stored profile to a connected device\n\
      profile-export ID [PATH] [--endpoint PATH]  Write versioned profile JSON\n\
      profile-import PATH [--endpoint PATH]       Import versioned profile JSON\n\
      --help\n\
      Profiles persist in a local SQLite file; default is under the user data directory\n\
      (override with --profiles PATH on serve, or PERIPHERAL_HUB_DATA).\n\
      --manifests DIR loads reviewed declarative TOML drivers for this run; invalid\n\
      manifests abort startup and no manifest is fetched or loaded at runtime.\n\
      Simulation is explicit software-only; no physical driver is bundled."
    );
}

/// Per-user database path; never inside the repository or a public location.
fn default_profiles_path() -> PathBuf {
    let base = std::env::var_os("PERIPHERAL_HUB_DATA")
        .map(PathBuf::from)
        .or_else(|| {
            #[cfg(windows)]
            {
                std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
            }
            #[cfg(not(windows))]
            {
                std::env::var_os("XDG_DATA_HOME")
                    .map(PathBuf::from)
                    .or_else(|| {
                        std::env::var_os("HOME")
                            .map(|home| PathBuf::from(home).join(".local/share"))
                    })
            }
        })
        .unwrap_or_else(std::env::temp_dir);
    base.join("peripheral-hub").join("profiles.sqlite")
}

/// Build a profile from diagnostic arguments; optional RGB sets whole-device static lighting.
fn profile_from(
    id: &str,
    name: &str,
    rgb: Option<(&str, &str, &str)>,
) -> Result<Profile, Box<dyn std::error::Error>> {
    let lighting = match rgb {
        Some((red, green, blue)) => Some(LightingCommand {
            zone: "whole-device".into(),
            color: RgbColor {
                red: red.parse()?,
                green: green.parse()?,
                blue: blue.parse()?,
            },
            brightness: None,
            effect: LightingEffect::Static,
        }),
        None => None,
    };
    Ok(Profile {
        id: ProfileId::new(id)?,
        name: name.into(),
        device_model: None,
        lighting,
    })
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
    let profiles_path = if let Some(index) = args.iter().position(|arg| arg == "--profiles") {
        if index + 1 >= args.len() {
            return Err("--profiles needs a local path".into());
        }
        let path = args.remove(index + 1);
        args.remove(index);
        Some(PathBuf::from(path))
    } else {
        None
    };
    let manifests_dir = if let Some(index) = args.iter().position(|arg| arg == "--manifests") {
        if index + 1 >= args.len() {
            return Err("--manifests needs a local directory path".into());
        }
        let path = args.remove(index + 1);
        args.remove(index);
        Some(PathBuf::from(path))
    } else {
        None
    };
    if args.is_empty() {
        return Err("missing command; see --help".into());
    }
    let command = args.remove(0);
    if command != "serve" && manifests_dir.is_some() {
        return Err("--manifests is only valid with serve".into());
    }
    if command == "enumerate" {
        if !args.is_empty() || endpoint.is_some() || profiles_path.is_some() {
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
        let (mut registry, source): (DriverRegistry, Box<dyn DeviceSource>) = if simulated {
            (simulated_registry(), Box::new(SimulatedSource::new()?))
        } else {
            let source: Box<dyn DeviceSource> = match HidDiscovery::new() {
                Ok(source) => Box::new(source),
                Err(error) => Box::new(UnavailableSource::new(error)),
            };
            (DriverRegistry::new(), source)
        };
        if let Some(dir) = manifests_dir {
            let mode = if simulated {
                ExecutionMode::Simulated
            } else {
                ExecutionMode::Physical
            };
            register_manifest_dir(&mut registry, &dir, mode)?;
        }
        let profiles_path = profiles_path.unwrap_or_else(default_profiles_path);
        if let Some(parent) = profiles_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let profiles = ProfileRepository::open_file(&profiles_path)?;
        tracing::info!(event = "profiles_opened");
        let manager = ManagerHandle::start(
            DeviceManager::with_profiles(registry, source, profiles)?,
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
    if command == "profile-export" {
        let (id, path) = match args.as_slice() {
            [id] => (id.clone(), None),
            [id, path] => (id.clone(), Some(PathBuf::from(path))),
            _ => return Err("profile-export needs ID [PATH]".into()),
        };
        let profile_id = ProfileId::new(id).map_err(|_| "invalid profile id")?;
        let response = Client::new(endpoint, Duration::from_secs(5))?
            .call(Operation::GetProfile { profile_id })
            .await?;
        let profile = match response.result {
            Ok(Reply::Profile(profile)) => profile,
            Ok(_) => return Err("unexpected daemon reply".into()),
            Err(fault) => return Err(format!("profile export failed: {fault}").into()),
        };
        let json = profile.to_json()?;
        match path {
            Some(path) => {
                std::fs::write(&path, json.as_bytes())?;
                eprintln!("wrote {}", path.display());
            }
            None => println!("{json}"),
        }
        return Ok(());
    }
    if command == "profile-import" {
        let [path] = args.as_slice() else {
            return Err("profile-import needs PATH".into());
        };
        let profile = import_profile(std::path::Path::new(path))?;
        let response = Client::new(endpoint, Duration::from_secs(5))?
            .call(Operation::SaveProfile { profile })
            .await?;
        let failed = response.result.is_err();
        println!("{}", serde_json::to_string_pretty(&response)?);
        if failed {
            return Err("daemon returned a structured failure (see JSON)".into());
        }
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
        ("profiles", []) => Operation::ListProfiles,
        ("profile-get", [id]) => Operation::GetProfile {
            profile_id: ProfileId::new(id.clone()).map_err(|_| "invalid profile id")?,
        },
        ("profile-delete", [id]) => Operation::DeleteProfile {
            profile_id: ProfileId::new(id.clone()).map_err(|_| "invalid profile id")?,
        },
        ("profile-apply", [id, interface_id]) => Operation::ApplyProfile {
            interface_id: DeviceInterfaceId::new(interface_id.clone())?,
            profile_id: ProfileId::new(id.clone()).map_err(|_| "invalid profile id")?,
        },
        ("profile-save", [id, name]) => Operation::SaveProfile {
            profile: profile_from(id, name, None)?,
        },
        ("profile-save", [id, name, red, green, blue]) => Operation::SaveProfile {
            profile: profile_from(id, name, Some((red, green, blue)))?,
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
