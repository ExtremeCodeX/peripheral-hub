use std::sync::mpsc::{self, SyncSender};
use std::time::{Duration, Instant};

use hidapi::{HidDevice, HidError};
use peripheral_hub_device_sdk::*;

/// Reviewed report layout and allowed operations. Default policies are not guessed.
#[derive(Clone)]
pub struct HidPolicy {
    /// Payload/deadline bounds, excluding the report ID.
    pub limits: TransportLimits,
    /// Whether interrupt input includes a report-ID byte (zero means unnumbered).
    pub numbered_input: bool,
    /// Reviewed output report IDs; empty denies all output writes.
    pub output_ids: Vec<u8>,
    /// Reviewed feature report IDs; empty denies all feature operations.
    pub feature_ids: Vec<u8>,
    /// Minimum time between report operations (1 ms..1 second).
    pub min_interval: Duration,
}

impl HidPolicy {
    /// Validate a reviewed policy before opening or issuing any report operation.
    pub fn validate(&self) -> DeviceResult<()> {
        self.limits.validate()?;
        if self.min_interval < Duration::from_millis(1)
            || self.min_interval > Duration::from_secs(1)
            || self.output_ids.len() > 256
            || self.feature_ids.len() > 256
            || (self.limits.output_bytes > 0 && self.output_ids.is_empty())
            || (self.limits.feature_bytes > 0 && self.feature_ids.is_empty())
        {
            return Err(DeviceError::InvalidArgument(
                "invalid reviewed HID policy".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn map_error(error: HidError) -> DeviceError {
    match error {
        HidError::IoError { error } => match error.kind() {
            std::io::ErrorKind::PermissionDenied => DeviceError::PermissionDenied,
            std::io::ErrorKind::NotFound
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::NotConnected => DeviceError::NotConnected,
            std::io::ErrorKind::TimedOut => DeviceError::Timeout,
            _ => DeviceError::TransportError("HID OS operation failed".into()),
        },
        _ => DeviceError::TransportError("hidapi operation failed".into()),
    }
}

enum Operation {
    Read(ReportKind, u8),
    Write(ReportKind, HidReport),
}
enum Outcome {
    Read(HidReport),
    Written(usize),
}
struct Work {
    operation: Operation,
    deadline: Instant,
    reply: mpsc::Sender<DeviceResult<Outcome>>,
}

trait Backend: Send + 'static {
    fn operate(
        &mut self,
        operation: Operation,
        policy: &HidPolicy,
        timeout: Duration,
    ) -> DeviceResult<Outcome>;
}

struct OsBackend(HidDevice);
impl Backend for OsBackend {
    fn operate(
        &mut self,
        operation: Operation,
        policy: &HidPolicy,
        timeout: Duration,
    ) -> DeviceResult<Outcome> {
        match operation {
            Operation::Read(kind, id) => {
                let maximum = if kind == ReportKind::Input {
                    policy.limits.input_bytes
                } else {
                    policy.limits.feature_bytes
                };
                let prefix = if kind == ReportKind::Feature || policy.numbered_input {
                    1
                } else {
                    0
                };
                let mut bytes = vec![0; maximum + prefix];
                let count = if kind == ReportKind::Input {
                    let millis = timeout.as_millis().clamp(1, i32::MAX as u128) as i32;
                    self.0.read_timeout(&mut bytes, millis).map_err(map_error)?
                } else {
                    bytes[0] = id;
                    self.0.get_feature_report(&mut bytes).map_err(map_error)?
                };
                normalize_read(&bytes, count, id, prefix).map(Outcome::Read)
            }
            Operation::Write(kind, report) => {
                let mut bytes = Vec::with_capacity(report.payload.len() + 1);
                bytes.push(report.report_id);
                bytes.extend_from_slice(&report.payload);
                if kind == ReportKind::Feature {
                    // hidapi's feature API returns success/error, not a byte count.
                    self.0.send_feature_report(&bytes).map_err(map_error)?;
                    Ok(Outcome::Written(report.payload.len()))
                } else {
                    let count = self.0.write(&bytes).map_err(map_error)?;
                    if count != bytes.len() {
                        return Err(DeviceError::TransportError("short HID output write".into()));
                    }
                    Ok(Outcome::Written(count - 1))
                }
            }
        }
    }
}

fn normalize_read(bytes: &[u8], count: usize, id: u8, prefix: usize) -> DeviceResult<HidReport> {
    if count == 0 {
        return Err(DeviceError::Timeout);
    }
    if count > bytes.len() || count < prefix || prefix > 1 {
        return Err(DeviceError::ProtocolError(
            "HID read size is invalid".into(),
        ));
    }
    let observed_id = if prefix == 1 { bytes[0] } else { 0 };
    if observed_id != id {
        return Err(DeviceError::ProtocolError(
            "unexpected HID report ID".into(),
        ));
    }
    Ok(HidReport {
        report_id: observed_id,
        payload: bytes[prefix..count].to_vec(),
    })
}

/// One OS handle owned by one bounded worker. Timeouts permanently retire it.
/// An in-flight kernel operation cannot always be cancelled by hidapi.
pub struct HidTransport {
    sender: Option<SyncSender<Work>>,
    policy: HidPolicy,
    next_operation: Instant,
}

impl HidTransport {
    pub(crate) fn new(device: HidDevice, policy: HidPolicy) -> DeviceResult<Self> {
        Self::with_backend(OsBackend(device), policy)
    }

    fn with_backend(backend: impl Backend, policy: HidPolicy) -> DeviceResult<Self> {
        policy.validate()?;
        let (sender, receiver) = mpsc::sync_channel::<Work>(1);
        let worker_policy = policy.clone();
        std::thread::Builder::new()
            .name("peripheral-hid".into())
            .spawn(move || {
                let mut backend = backend;
                while let Ok(work) = receiver.recv() {
                    let result = if let Some(remaining) =
                        work.deadline.checked_duration_since(Instant::now())
                    {
                        backend.operate(work.operation, &worker_policy, remaining)
                    } else {
                        Err(DeviceError::Timeout)
                    };
                    let _ = work.reply.send(result);
                }
            })
            .map_err(|_| DeviceError::TransportError("HID worker unavailable".into()))?;
        Ok(Self {
            sender: Some(sender),
            policy,
            next_operation: Instant::now(),
        })
    }

    fn perform(&mut self, operation: Operation, timeout: Duration) -> DeviceResult<Outcome> {
        let sender = self.sender.as_ref().ok_or(DeviceError::NotConnected)?;
        if timeout.is_zero() || timeout > self.policy.limits.max_timeout {
            return Err(DeviceError::InvalidArgument(
                "HID timeout out of range".into(),
            ));
        }
        if Instant::now() < self.next_operation {
            return Err(DeviceError::InvalidArgument(
                "HID rate limit exceeded".into(),
            ));
        }
        self.next_operation = Instant::now() + self.policy.min_interval;
        let deadline = Instant::now() + timeout;
        let (reply, receiver) = mpsc::channel();
        sender
            .try_send(Work {
                operation,
                deadline,
                reply,
            })
            .map_err(|_| DeviceError::NotConnected)?;
        let mut result = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap_or(Err(DeviceError::Timeout));
        if Instant::now() > deadline {
            result = Err(DeviceError::Timeout);
        }
        if matches!(
            result,
            Err(DeviceError::Timeout | DeviceError::NotConnected | DeviceError::TransportError(_))
        ) {
            self.sender = None;
        }
        result
    }
}

impl Transport for HidTransport {
    fn limits(&self) -> TransportLimits {
        self.policy.limits
    }
    fn read_report(
        &mut self,
        kind: ReportKind,
        id: u8,
        timeout: Duration,
    ) -> DeviceResult<HidReport> {
        if kind == ReportKind::Output {
            return Err(DeviceError::InvalidArgument(
                "cannot read output reports".into(),
            ));
        }
        let maximum = match kind {
            ReportKind::Input => self.policy.limits.input_bytes,
            ReportKind::Feature => self.policy.limits.feature_bytes,
            ReportKind::Output => 0,
        };
        if maximum == 0 || (kind == ReportKind::Feature && !self.policy.feature_ids.contains(&id)) {
            return Err(DeviceError::NotSupported);
        }
        if kind == ReportKind::Input && !self.policy.numbered_input && id != 0 {
            return Err(DeviceError::InvalidArgument(
                "unnumbered input requires ID zero".into(),
            ));
        }
        match self.perform(Operation::Read(kind, id), timeout)? {
            Outcome::Read(report) => Ok(report),
            _ => Err(DeviceError::ProtocolError(
                "wrong HID worker response".into(),
            )),
        }
    }
    fn write_report(
        &mut self,
        kind: ReportKind,
        report: &HidReport,
        timeout: Duration,
    ) -> DeviceResult<usize> {
        let (maximum, allowed) = match kind {
            ReportKind::Output => (self.policy.limits.output_bytes, &self.policy.output_ids),
            ReportKind::Feature => (self.policy.limits.feature_bytes, &self.policy.feature_ids),
            ReportKind::Input => {
                return Err(DeviceError::InvalidArgument(
                    "cannot write input reports".into(),
                ));
            }
        };
        if maximum == 0 || !allowed.contains(&report.report_id) {
            return Err(DeviceError::NotSupported);
        }
        if report.payload.is_empty() || report.payload.len() > maximum {
            return Err(DeviceError::InvalidArgument(
                "HID payload exceeds reviewed bounds".into(),
            ));
        }
        match self.perform(Operation::Write(kind, report.clone()), timeout)? {
            Outcome::Written(count) => Ok(count),
            _ => Err(DeviceError::ProtocolError(
                "wrong HID worker response".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_read_normalization_keeps_unnumbered_zero_payload_and_checks_feature_prefix() {
        assert_eq!(
            normalize_read(&[0, 42], 2, 0, 0).unwrap().payload,
            vec![0, 42]
        );
        assert_eq!(
            normalize_read(&[3, 0, 42], 3, 3, 1).unwrap().payload,
            vec![0, 42]
        );
        assert_eq!(normalize_read(&[0, 42], 2, 0, 1).unwrap().payload, vec![42]);
        assert_eq!(normalize_read(&[0], 0, 0, 0), Err(DeviceError::Timeout));
        assert!(normalize_read(&[3, 42], 2, 2, 1).is_err());
        assert!(normalize_read(&[3, 42], 3, 3, 1).is_err());
        assert_eq!(
            map_error(HidError::IoError {
                error: std::io::ErrorKind::PermissionDenied.into()
            }),
            DeviceError::PermissionDenied
        );
    }
    struct SlowBackend;
    impl Backend for SlowBackend {
        fn operate(&mut self, _: Operation, _: &HidPolicy, _: Duration) -> DeviceResult<Outcome> {
            std::thread::sleep(Duration::from_millis(60));
            Ok(Outcome::Written(1))
        }
    }
    fn policy() -> HidPolicy {
        HidPolicy {
            limits: TransportLimits {
                input_bytes: 8,
                output_bytes: 8,
                feature_bytes: 0,
                max_timeout: Duration::from_millis(100),
            },
            numbered_input: false,
            output_ids: vec![0],
            feature_ids: vec![],
            min_interval: Duration::from_millis(1),
        }
    }
    #[test]
    fn deadline_retires_handle_and_never_returns_late_success() {
        let mut transport = HidTransport::with_backend(SlowBackend, policy()).unwrap();
        let report = HidReport {
            report_id: 0,
            payload: vec![1],
        };
        assert_eq!(
            transport.write_report(ReportKind::Output, &report, Duration::from_millis(5)),
            Err(DeviceError::Timeout)
        );
        assert_eq!(
            transport.write_report(ReportKind::Output, &report, Duration::from_millis(5)),
            Err(DeviceError::NotConnected)
        );
    }
    #[test]
    fn forbidden_reports_are_rejected_before_worker_io() {
        let mut transport = HidTransport::with_backend(SlowBackend, policy()).unwrap();
        assert_eq!(
            transport.write_report(
                ReportKind::Output,
                &HidReport {
                    report_id: 1,
                    payload: vec![1]
                },
                Duration::from_millis(10)
            ),
            Err(DeviceError::NotSupported)
        );
        assert_eq!(
            transport.read_report(ReportKind::Feature, 0, Duration::from_millis(10)),
            Err(DeviceError::NotSupported)
        );
        assert!(
            transport
                .read_report(ReportKind::Input, 1, Duration::from_millis(10))
                .is_err()
        );
    }
}
