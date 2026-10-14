//! Actual local socket/pipe integration, explicit simulated driver and safe shutdown.
use peripheral_hub_daemon::*;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_transport_hid::simulated::SimulatedSource;
use std::time::Duration;
use tokio::sync::oneshot;

fn private_directory() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    directory
}

fn endpoint(directory: &tempfile::TempDir, suffix: &str) -> Endpoint {
    #[cfg(unix)]
    {
        Endpoint::new(directory.path().join(format!("{suffix}.sock")))
    }
    #[cfg(windows)]
    {
        let _ = directory;
        let path = Endpoint::default_for_user()
            .unwrap()
            .path()
            .to_string_lossy()
            .into_owned();
        Endpoint::new(path.replacen(
            "peripheral-hub-v1-",
            &format!("peripheral-hub-test-{}-{suffix}-", std::process::id()),
            1,
        ))
    }
}

async fn start(
    endpoint: &Endpoint,
) -> (
    oneshot::Sender<()>,
    tokio::task::JoinHandle<Result<(), IpcError>>,
) {
    let listener = Listener::bind(endpoint).await.unwrap();
    let manager = ManagerHandle::start(
        DeviceManager::new(simulated_registry(), Box::new(SimulatedSource::default())),
        Duration::from_secs(1),
    )
    .unwrap();
    let (stop, stopped) = oneshot::channel();
    let server = tokio::spawn(serve(
        listener,
        manager,
        ServerConfig {
            poll_interval: Duration::from_millis(25),
            frame_timeout: Duration::from_millis(250),
            max_clients: 4,
        },
        async {
            let _ = stopped.await;
        },
    ));
    (stop, server)
}

#[tokio::test]
async fn typed_client_driver_transport_roundtrip_and_reconnect_after_restart() {
    let directory = private_directory();
    let endpoint = endpoint(&directory, "roundtrip");
    let (stop, server) = start(&endpoint).await;
    let mut client = Client::new(endpoint.clone(), Duration::from_secs(2)).unwrap();
    let Reply::Health(health) = client
        .call(Operation::Health)
        .await
        .unwrap()
        .result
        .unwrap()
    else {
        panic!("health")
    };
    assert_eq!(health.mode, ExecutionMode::Simulated);
    let Reply::Devices(devices) = client
        .call(Operation::ListDevices)
        .await
        .unwrap()
        .result
        .unwrap()
    else {
        panic!("devices")
    };
    let json = serde_json::to_string(&devices).unwrap();
    assert!(!json.contains("serial_number"));
    assert!(!json.contains("simulation-only"));
    assert!(!json.contains("\"path\""));
    let id = devices[0].interface_id.clone();
    let old_id = id.clone();
    assert!(matches!(
        client
            .call(Operation::Connect {
                interface_id: id.clone()
            })
            .await
            .unwrap()
            .result,
        Ok(Reply::Device(DeviceSummary {
            status: ConnectionStatus::Connected,
            ..
        }))
    ));
    let lighting = LightingCommand {
        zone: "whole-device".into(),
        color: RgbColor {
            red: 0,
            green: 255,
            blue: 0,
        },
        brightness: None,
        effect: LightingEffect::Static,
    };
    assert!(matches!(
        client
            .call(Operation::SetLighting {
                interface_id: id.clone(),
                command: lighting.clone()
            })
            .await
            .unwrap()
            .result,
        Ok(Reply::Outcome(CommandOutcome {
            mode: ExecutionMode::Simulated
        }))
    ));
    let Reply::State(state) = client
        .call(Operation::ReadState {
            interface_id: id.clone(),
        })
        .await
        .unwrap()
        .result
        .unwrap()
    else {
        panic!("state")
    };
    assert_eq!(state.lighting, Some(lighting));
    client
        .call(Operation::SimulatePresence { present: false })
        .await
        .unwrap()
        .result
        .unwrap();
    assert!(matches!(
        client
            .call(Operation::GetDevice { interface_id: id })
            .await
            .unwrap()
            .result,
        Err(Fault::Device(DeviceError::NotConnected))
    ));
    let Reply::Events(events) = client
        .call(Operation::Events { after: 0 })
        .await
        .unwrap()
        .result
        .unwrap()
    else {
        panic!("events")
    };
    assert!(
        events
            .events
            .iter()
            .any(|event| event.status == ConnectionStatus::Disconnected)
    );
    stop.send(()).unwrap();
    server.await.unwrap().unwrap();
    assert!(client.call(Operation::Health).await.is_err());
    let (stop, server) = start(&endpoint).await;
    let Reply::Health(restarted) = client
        .call(Operation::Health)
        .await
        .unwrap()
        .result
        .unwrap()
    else {
        panic!("health")
    };
    assert_ne!(restarted.session_id, health.session_id);
    assert!(matches!(
        client
            .call(Operation::GetDevice {
                interface_id: old_id
            })
            .await
            .unwrap()
            .result,
        Err(Fault::Device(DeviceError::NotConnected))
    ));
    stop.send(()).unwrap();
    server.await.unwrap().unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn wrong_version_malformed_oversized_and_idle_clients_do_not_kill_daemon() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let directory = private_directory();
    let endpoint = endpoint(&directory, "rejection");
    let (stop, server) = start(&endpoint).await;
    let mut raw = tokio::net::UnixStream::connect(endpoint.path())
        .await
        .unwrap();
    write_frame(
        &mut raw,
        &Request {
            version: IPC_VERSION + 1,
            request_id: 7,
            operation: Operation::Health,
        },
    )
    .await
    .unwrap();
    let response: Response = read_frame(&mut raw).await.unwrap();
    assert_eq!(response.result.unwrap_err(), Fault::IncompatibleVersion);
    drop(raw);
    for payload in [None, Some(b"not-json".as_slice())] {
        let mut raw = tokio::net::UnixStream::connect(endpoint.path())
            .await
            .unwrap();
        if let Some(bytes) = payload {
            raw.write_u32(bytes.len() as u32).await.unwrap();
            raw.write_all(bytes).await.unwrap();
        } else {
            raw.write_u32(MAX_FRAME_BYTES as u32 + 1).await.unwrap();
        }
        let mut byte = [0];
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(1), raw.read(&mut byte))
                .await
                .unwrap()
                .unwrap(),
            0
        );
    }
    let _idle = tokio::net::UnixStream::connect(endpoint.path())
        .await
        .unwrap();
    let mut client = Client::new(endpoint.clone(), Duration::from_secs(2)).unwrap();
    assert!(client.call(Operation::Health).await.unwrap().result.is_ok());
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(!endpoint.path().exists());
}
