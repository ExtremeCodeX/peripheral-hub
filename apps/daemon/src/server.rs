use crate::DeviceManager;
use peripheral_hub_device_sdk::DeviceError;
use peripheral_hub_ipc::*;
use std::future::Future;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, oneshot};
use tokio::task::JoinSet;

enum WorkKind {
    Operation(Operation),
    Refresh,
    Shutdown,
}
struct Work {
    kind: WorkKind,
    deadline: Instant,
    reply: oneshot::Sender<Result<Reply, Fault>>,
}

/// Bounded serialized manager actor; blocking HID work never runs on async I/O tasks.
#[derive(Clone)]
pub struct ManagerHandle {
    sender: mpsc::SyncSender<Work>,
    timeout: Duration,
}
impl ManagerHandle {
    /// Start one serialized manager worker with a 32-item queue and bounded deadlines.
    pub fn start(mut manager: DeviceManager, timeout: Duration) -> std::io::Result<Self> {
        if timeout.is_zero() || timeout > Duration::from_secs(30) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid manager timeout",
            ));
        }
        let (sender, receiver) = mpsc::sync_channel::<Work>(32);
        std::thread::Builder::new()
            .name("peripheral-manager".into())
            .spawn(move || {
                let _ = manager.refresh();
                while let Ok(work) = receiver.recv() {
                    if work.deadline <= Instant::now() {
                        let _ = work.reply.send(Err(Fault::Timeout));
                        continue;
                    }
                    let stop = matches!(work.kind, WorkKind::Shutdown);
                    let result = match work.kind {
                        WorkKind::Operation(operation) => manager.handle(operation),
                        WorkKind::Refresh => {
                            manager.refresh().map(|_| Reply::Ack).map_err(Fault::from)
                        }
                        WorkKind::Shutdown => {
                            manager.shutdown().map(|_| Reply::Ack).map_err(Fault::from)
                        }
                    };
                    let _ = work.reply.send(result);
                    if stop {
                        return;
                    }
                }
                let _ = manager.shutdown();
            })?;
        Ok(Self { sender, timeout })
    }
    async fn submit(&self, kind: WorkKind) -> Result<Reply, Fault> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .try_send(Work {
                kind,
                deadline: Instant::now() + self.timeout,
                reply,
            })
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => Fault::Busy,
                mpsc::TrySendError::Disconnected(_) => Fault::Internal,
            })?;
        tokio::time::timeout(self.timeout, receiver)
            .await
            .map_err(|_| Fault::Timeout)?
            .map_err(|_| Fault::Internal)?
    }
    /// Submit a typed operation without replaying a timed-out mutation.
    pub async fn request(&self, operation: Operation) -> Result<Reply, Fault> {
        self.submit(WorkKind::Operation(operation)).await
    }
    /// Close managed handles and stop the worker.
    pub async fn shutdown(&self) -> Result<(), Fault> {
        self.submit(WorkKind::Shutdown).await.map(|_| ())
    }
    fn poll(&self) {
        let (reply, _) = oneshot::channel();
        let _ = self.sender.try_send(Work {
            kind: WorkKind::Refresh,
            deadline: Instant::now() + self.timeout,
            reply,
        });
    }
}

/// Server resource/deadline policy; no privileged or network settings exist.
#[derive(Clone)]
pub struct ServerConfig {
    /// Enumeration interval (10 ms..60 seconds).
    pub poll_interval: Duration,
    /// Incoming/outgoing frame deadline (1 ms..30 seconds).
    pub frame_timeout: Duration,
    /// Concurrent local clients (1..32).
    pub max_clients: usize,
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
            frame_timeout: Duration::from_secs(5),
            max_clients: 16,
        }
    }
}
impl ServerConfig {
    fn validate(&self) -> Result<(), IpcError> {
        if self.poll_interval < Duration::from_millis(10)
            || self.poll_interval > Duration::from_secs(60)
            || self.frame_timeout < Duration::from_millis(1)
            || self.frame_timeout > Duration::from_secs(30)
            || !(1..=32).contains(&self.max_clients)
        {
            return Err(IpcError::InvalidMessage);
        }
        Ok(())
    }
}

struct Budget {
    start: Instant,
    used: usize,
}
impl Budget {
    fn take(&mut self) -> bool {
        if self.start.elapsed() >= Duration::from_secs(1) {
            self.start = Instant::now();
            self.used = 0;
        }
        if self.used == 128 {
            return false;
        }
        self.used += 1;
        true
    }
}

async fn client(
    mut stream: Box<dyn LocalStream>,
    manager: ManagerHandle,
    config: ServerConfig,
    budget: Arc<Mutex<Budget>>,
) -> Result<(), IpcError> {
    // Cap work even for clients retaining an idle connection indefinitely.
    for _ in 0..32 {
        let request: Request = tokio::time::timeout(config.frame_timeout, read_frame(&mut stream))
            .await
            .map_err(|_| IpcError::Timeout)??;
        let result = if request.version != IPC_VERSION {
            Err(Fault::IncompatibleVersion)
        } else if !budget.lock().map_err(|_| IpcError::InvalidMessage)?.take() {
            Err(Fault::Busy)
        } else {
            manager.request(request.operation).await
        };
        let response = Response {
            version: IPC_VERSION,
            request_id: request.request_id,
            result,
        };
        tokio::time::timeout(config.frame_timeout, write_frame(&mut stream, &response))
            .await
            .map_err(|_| IpcError::Timeout)??;
    }
    Ok(())
}

/// Run polling and bounded local clients until an explicit shutdown future resolves.
/// Bind the protected Listener before invoking this function to guarantee readiness.
pub async fn serve(
    listener: Listener,
    manager: ManagerHandle,
    config: ServerConfig,
    shutdown: impl Future<Output = ()>,
) -> Result<(), IpcError> {
    config.validate()?;
    let permits = Arc::new(Semaphore::new(config.max_clients));
    let budget = Arc::new(Mutex::new(Budget {
        start: Instant::now(),
        used: 0,
    }));
    let mut interval = tokio::time::interval(config.poll_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut clients = JoinSet::new();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            _ = interval.tick() => manager.poll(),
            finished = clients.join_next(), if !clients.is_empty() => {
                if matches!(finished, Some(Err(_))) { tracing::warn!(event = "ipc_task_failed"); }
            },
            accepted = listener.accept() => match accepted {
                Ok(stream) => {
                    if let Ok(permit) = Arc::clone(&permits).try_acquire_owned() {
                        let manager = manager.clone(); let config = config.clone(); let budget = Arc::clone(&budget);
                        clients.spawn(async move { let _permit = permit; let _ = client(stream, manager, config, budget).await; });
                    }
                },
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => tracing::warn!(event = "ipc_peer_denied"),
                Err(error) => { clients.abort_all(); let _ = manager.shutdown().await; return Err(error.into()); },
            }
        }
    }
    clients.abort_all();
    while clients.join_next().await.is_some() {}
    manager.shutdown().await.map_err(|error| match error {
        Fault::Timeout => IpcError::Timeout,
        Fault::Device(DeviceError::Timeout) => IpcError::Timeout,
        _ => IpcError::InvalidMessage,
    })?;
    drop(listener);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_budget_is_bounded_and_recovers_after_window() {
        let mut budget = Budget {
            start: Instant::now(),
            used: 0,
        };
        for _ in 0..128 {
            assert!(budget.take());
        }
        assert!(!budget.take());
        budget.start = Instant::now() - Duration::from_secs(2);
        assert!(budget.take());
        assert_eq!(budget.used, 1);
    }
}
