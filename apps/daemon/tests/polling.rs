//! Polling and queued-request deadlines independent of physical peripherals.
use peripheral_hub_daemon::*;
use peripheral_hub_device_sdk::*;
use peripheral_hub_ipc::*;
use peripheral_hub_transport_hid::{DeviceSource, simulated::SimulatedSource};
#[cfg(unix)]
use std::sync::atomic::AtomicBool;
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

#[cfg(unix)]
struct SharedSource {
    inner: SimulatedSource,
    present: Arc<AtomicBool>,
}
#[cfg(unix)]
impl DeviceSource for SharedSource {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Simulated
    }
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        self.inner
            .set_simulated_presence(self.present.load(Ordering::Relaxed))?;
        self.inner.scan()
    }
    fn open(&mut self, descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        self.inner.open(descriptor)
    }
}

#[cfg(unix)]
#[tokio::test]
async fn daemon_polling_detects_external_presence_changes_without_ipc_mutation() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let endpoint = Endpoint::new(directory.path().join("poll.sock"));
    let listener = Listener::bind(&endpoint).await.unwrap();
    let present = Arc::new(AtomicBool::new(true));
    let source = SharedSource {
        inner: SimulatedSource::default(),
        present: Arc::clone(&present),
    };
    let manager = ManagerHandle::start(
        DeviceManager::new(simulated_registry(), Box::new(source)),
        Duration::from_secs(1),
    )
    .unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(serve(
        listener,
        manager.clone(),
        ServerConfig {
            poll_interval: Duration::from_millis(10),
            ..ServerConfig::default()
        },
        async {
            let _ = stopped.await;
        },
    ));
    let Reply::Devices(devices) = manager.request(Operation::ListDevices).await.unwrap() else {
        panic!("devices")
    };
    let original = devices[0].interface_id.clone();
    present.store(false, Ordering::Relaxed);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if matches!(manager.request(Operation::ListDevices).await.unwrap(), Reply::Devices(devices) if devices.is_empty()) { break; }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }).await.unwrap();
    present.store(true, Ordering::Relaxed);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Reply::Devices(devices) = manager.request(Operation::ListDevices).await.unwrap()
                && !devices.is_empty()
            {
                assert_ne!(devices[0].interface_id, original);
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    stop.send(()).unwrap();
    task.await.unwrap().unwrap();
}

struct GatedSource {
    gate: Option<Arc<Barrier>>,
    calls: Arc<AtomicUsize>,
    inner: SimulatedSource,
}
impl DeviceSource for GatedSource {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Simulated
    }
    fn scan(&mut self) -> DeviceResult<Vec<DeviceDescriptor>> {
        if let Some(gate) = self.gate.take() {
            gate.wait();
        }
        self.inner.scan()
    }
    fn open(&mut self, descriptor: &DeviceDescriptor) -> DeviceResult<Box<dyn Transport>> {
        self.inner.open(descriptor)
    }
    fn set_simulated_presence(&mut self, present: bool) -> DeviceResult<()> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.inner.set_simulated_presence(present)
    }
}

#[tokio::test]
async fn expired_queued_mutations_are_not_executed_later() {
    let gate = Arc::new(Barrier::new(2));
    let calls = Arc::new(AtomicUsize::new(0));
    let source = GatedSource {
        gate: Some(Arc::clone(&gate)),
        calls: Arc::clone(&calls),
        inner: SimulatedSource::default(),
    };
    let manager = ManagerHandle::start(
        DeviceManager::new(simulated_registry(), Box::new(source)),
        Duration::from_millis(50),
    )
    .unwrap();
    assert_eq!(
        manager
            .request(Operation::SimulatePresence { present: false })
            .await
            .unwrap_err(),
        Fault::Timeout
    );
    gate.wait();
    manager.request(Operation::Health).await.unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    manager.shutdown().await.unwrap();
}
