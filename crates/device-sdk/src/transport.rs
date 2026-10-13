use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{DeviceError, DeviceResult, error::invalid};

/// Distinct HID report operations, not unrestricted USB access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    /// Device-to-host input report.
    Input,
    /// Host-to-device output report.
    Output,
    /// Explicit bidirectional feature report.
    Feature,
}

/// Normalized HID report: report ID is separate from payload (including ID zero).
#[derive(Clone, PartialEq, Eq)]
pub struct HidReport {
    /// Observed/selected report ID, not prepended to the payload.
    pub report_id: u8,
    /// Report data excluding the report ID; do not log by default.
    pub payload: Vec<u8>,
}

impl std::fmt::Debug for HidReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HidReport")
            .field("report_id", &self.report_id)
            .field("payload_bytes", &self.payload.len())
            .finish_non_exhaustive()
    }
}

/// Local operation bounds; zero bytes disables that report kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportLimits {
    /// Maximum input payload bytes, excluding the report ID (0..65535).
    pub input_bytes: usize,
    /// Maximum output payload bytes, excluding the report ID (0..65535).
    pub output_bytes: usize,
    /// Maximum feature payload bytes, excluding the report ID (0..65535).
    pub feature_bytes: usize,
    /// Maximum nonzero operation timeout, at most 30 seconds.
    pub max_timeout: Duration,
}

impl TransportLimits {
    /// Validate limits without touching a device.
    pub fn validate(self) -> DeviceResult<()> {
        if [self.input_bytes, self.output_bytes, self.feature_bytes]
            .into_iter()
            .any(|n| n > 65535)
            || self.max_timeout.is_zero()
            || self.max_timeout > Duration::from_secs(30)
        {
            return Err(invalid("transport limits exceed SDK bounds"));
        }
        Ok(())
    }

    fn max_bytes(self, kind: ReportKind) -> usize {
        match kind {
            ReportKind::Input => self.input_bytes,
            ReportKind::Output => self.output_bytes,
            ReportKind::Feature => self.feature_bytes,
        }
    }

    fn validate_request(self, kind: ReportKind, timeout: Duration) -> DeviceResult<()> {
        self.validate()?;
        if self.max_bytes(kind) == 0 {
            return Err(DeviceError::NotSupported);
        }
        if timeout.is_zero() || timeout > self.max_timeout {
            return Err(invalid("timeout is outside transport bounds"));
        }
        Ok(())
    }
}

/// Backend interface for bounded HID report I/O, no backend supplied in Phase 1.
///
/// Writes return payload byte counts, excluding the report ID. A backend must
/// normalize OS-specific conventions, honor deadlines, and not allocate beyond
/// declared limits. Reads require the requested report ID; mismatch is an error.
pub trait Transport: Send {
    /// Backend-declared report limits.
    fn limits(&self) -> TransportLimits;
    /// Read input/feature data; output reports cannot be read.
    fn read_report(
        &mut self,
        kind: ReportKind,
        report_id: u8,
        timeout: Duration,
    ) -> DeviceResult<HidReport>;
    /// Write output/feature data; input reports cannot be written.
    fn write_report(
        &mut self,
        kind: ReportKind,
        report: &HidReport,
        timeout: Duration,
    ) -> DeviceResult<usize>;
}

/// Enforce validated limits around a backend and reject partial writes/responses.
/// This wrapper performs no I/O unless the supplied backend does.
pub struct CheckedTransport<T> {
    inner: T,
    limits: TransportLimits,
}

impl<T: Transport> CheckedTransport<T> {
    /// Snapshot and validate backend limits.
    pub fn new(inner: T) -> DeviceResult<Self> {
        let limits = inner.limits();
        limits.validate()?;
        Ok(Self { inner, limits })
    }
}

impl<T: Transport> Transport for CheckedTransport<T> {
    fn limits(&self) -> TransportLimits {
        self.limits
    }

    fn read_report(
        &mut self,
        kind: ReportKind,
        report_id: u8,
        timeout: Duration,
    ) -> DeviceResult<HidReport> {
        if kind == ReportKind::Output {
            return Err(invalid("output reports cannot be read"));
        }
        self.limits.validate_request(kind, timeout)?;
        let report = self.inner.read_report(kind, report_id, timeout)?;
        if report.report_id != report_id || report.payload.len() > self.limits.max_bytes(kind) {
            return Err(DeviceError::ProtocolError(
                "report ID or payload length violated the request".into(),
            ));
        }
        Ok(report)
    }

    fn write_report(
        &mut self,
        kind: ReportKind,
        report: &HidReport,
        timeout: Duration,
    ) -> DeviceResult<usize> {
        if kind == ReportKind::Input {
            return Err(invalid("input reports cannot be written"));
        }
        self.limits.validate_request(kind, timeout)?;
        if report.payload.is_empty() || report.payload.len() > self.limits.max_bytes(kind) {
            return Err(invalid("report payload length is outside transport bounds"));
        }
        let written = self.inner.write_report(kind, report, timeout)?;
        if written != report.payload.len() {
            return Err(DeviceError::TransportError(
                "incomplete or invalid report write count".into(),
            ));
        }
        Ok(written)
    }
}
