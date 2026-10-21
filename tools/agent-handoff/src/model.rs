//! Stable documents emitted and verified by the agent handoff tool.

use std::path::PathBuf;

use device_research_assistant::analysis::{Confidence, Hypothesis};
use serde::{Deserialize, Serialize};

/// Version of the generated `handoff.json` document.
pub const HANDOFF_SCHEMA_VERSION: u32 = 1;
/// Maximum accepted DRA report size.
pub const MAX_REPORT_BYTES: u64 = 32 * 1024 * 1024;
/// Maximum accepted declarative manifest size.
pub const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
/// Maximum bytes permitted in one generated Markdown file.
pub const MAX_GENERATED_MARKDOWN_BYTES: usize = 8 * 1024 * 1024;

/// Evidence gate result for a validated DRA report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Readiness {
    /// No hypotheses exist yet, so no handoff may be generated.
    Insufficient,
    /// Only unverified hypotheses exist; gather more controlled evidence.
    NeedsMoreEvidence,
    /// Inferred correlations are sufficient for a clearly labeled scaffold.
    ScaffoldReady,
    /// A human confirmed evidence on hardware; review it before implementation.
    HardwareReview,
}

impl Readiness {
    /// Stable lowercase label used in CLI output.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Insufficient => "insufficient",
            Self::NeedsMoreEvidence => "needs-more-evidence",
            Self::ScaffoldReady => "scaffold-ready",
            Self::HardwareReview => "hardware-review",
        }
    }

    /// Explain what the next contributor or agent must do.
    pub const fn guidance(self) -> &'static str {
        match self {
            Self::Insufficient => {
                "no hypothesis exists; capture a baseline and one labeled change before handoff"
            }
            Self::NeedsMoreEvidence => {
                "only unverified observations exist; refine the experiment before driver work"
            }
            Self::ScaffoldReady => {
                "inferred correlations may seed a scaffold; they are not hardware confirmation"
            }
            Self::HardwareReview => {
                "a human confirmed evidence on hardware; review provenance before implementation"
            }
        }
    }
}

/// Counts of hypotheses grouped by evidence label.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypothesisCounts {
    /// Confirmed-on-hardware hypotheses (only possible in human-authored reports).
    pub confirmed_on_hardware: usize,
    /// Inferred correlation hypotheses.
    pub inferred: usize,
    /// Unverified observation hypotheses.
    pub unverified: usize,
}

impl HypothesisCounts {
    /// Count every hypothesis in a report.
    pub fn from_hypotheses(hypotheses: &[Hypothesis]) -> Self {
        let mut counts = Self::default();
        for hypothesis in hypotheses {
            match hypothesis.confidence {
                Confidence::ConfirmedOnHardware => counts.confirmed_on_hardware += 1,
                Confidence::Inferred => counts.inferred += 1,
                Confidence::Unverified => counts.unverified += 1,
            }
        }
        counts
    }

    /// Total number of hypotheses.
    pub const fn total(self) -> usize {
        self.confirmed_on_hardware + self.inferred + self.unverified
    }
}

/// Report provenance recorded in the handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEvidence {
    /// Relative path of the copied sanitized report.
    pub report_file: String,
    /// SHA-256 of the exact DRA report bytes.
    pub report_sha256: String,
    /// DRA report schema version.
    pub report_schema_version: u32,
    /// DRA session schema version.
    pub session_schema_version: u32,
    /// Session creation time in Unix seconds.
    pub session_created_at_unix: u64,
    /// Analysis generation time in Unix seconds.
    pub analysis_generated_at_unix: u64,
    /// Report generation time in Unix seconds.
    pub report_generated_at_unix: u64,
    /// Version of the handoff document produced from this evidence.
    pub handoff_schema_version: u32,
}

/// Device identity as recorded by the session; not physical proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceEvidence {
    /// User-supplied model name.
    pub model: String,
    /// Firmware or revision only when the session recorded it.
    #[serde(default)]
    pub firmware: Option<String>,
}

/// Sanitized capture metadata copied from the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureEvidence {
    /// Safe relative capture file name.
    pub file: String,
    /// Declared origin token.
    pub source: String,
    /// Optional description recorded by the researcher.
    #[serde(default)]
    pub description: Option<String>,
    /// Number of normalized events.
    pub event_count: usize,
    /// Whether normalization changed the source order.
    pub reordered: bool,
    /// Import time in Unix seconds.
    pub imported_at_unix: u64,
}

/// Structural validation result for an optional declarative manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEvidence {
    /// SHA-256 of the exact manifest bytes.
    pub sha256: String,
    /// Validated model id.
    pub model_id: String,
    /// Validated display name.
    pub display_name: String,
    /// Number of match rules.
    pub match_rule_count: usize,
    /// Number of modeled operations.
    pub operation_count: usize,
    /// Declared SDK contract major version.
    pub sdk_major: u16,
    /// Declared SDK contract minor version.
    pub sdk_minor: u16,
    /// Human-readable validation boundary.
    pub validation_note: String,
}

/// Deterministic evidence assessment for agent and human review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAssessment {
    /// SHA-256 of the assessed report.
    pub report_sha256: String,
    /// Session id recorded in the report.
    pub session_id: String,
    /// Evidence gate result.
    pub readiness: Readiness,
    /// Hypotheses grouped by evidence label.
    pub counts: HypothesisCounts,
    /// Hypotheses copied verbatim from the sanitized report.
    pub hypotheses: Vec<Hypothesis>,
    /// Controlled experiments still required before implementation.
    pub required_experiments: Vec<String>,
    /// Interpretation limits that must travel with the evidence.
    pub warnings: Vec<String>,
}

/// Machine-readable handoff document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffDocument {
    /// Version of this document.
    pub handoff_schema_version: u32,
    /// Local generation time in Unix seconds.
    pub created_at_unix: u64,
    /// Exact report provenance.
    pub source: SourceEvidence,
    /// Device identity recorded by the session.
    pub device: DeviceEvidence,
    /// Sanitized capture metadata.
    pub captures: Vec<CaptureEvidence>,
    /// Evidence assessment and remaining work.
    pub assessment: EvidenceAssessment,
    /// Optional structurally validated manifest.
    #[serde(default)]
    pub manifest: Option<ManifestEvidence>,
    /// Explicit statements about what the tool did not generate or perform.
    pub safety: SafetyBoundaries,
}

/// Safety statements that always accompany a generated handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafetyBoundaries {
    /// No driver code was generated.
    pub driver_code_generated: bool,
    /// No loadable manifest was generated.
    pub loadable_manifest_generated: bool,
    /// No hardware was accessed by this tool.
    pub hardware_access_performed: bool,
    /// No vendor executable was run.
    pub vendor_execution_performed: bool,
    /// No packet was sent or replayed.
    pub packet_replay_performed: bool,
    /// No raw capture or private artifact was copied.
    pub raw_capture_included: bool,
    /// No full payload was copied; only sanitized offsets/statements are present.
    pub full_payloads_included: bool,
    /// Stable interpretation reminders.
    pub notes: Vec<String>,
}

impl SafetyBoundaries {
    /// Build the fixed safety record for every handoff.
    pub fn none_performed() -> Self {
        Self {
            driver_code_generated: false,
            loadable_manifest_generated: false,
            hardware_access_performed: false,
            vendor_execution_performed: false,
            packet_replay_performed: false,
            raw_capture_included: false,
            full_payloads_included: false,
            notes: vec![
                "Evidence labels come from the DRA; this tool revalidates consistency, not physical behavior."
                    .to_owned(),
                "The copied report is sanitized by the DRA and must still be reviewed before sharing."
                    .to_owned(),
                "Explicit human authorization is required before any physical experiment."
                    .to_owned(),
                "Unknown packets must never be replayed, and no capability may be claimed from this handoff."
                    .to_owned(),
            ],
        }
    }
}

/// Paths written by a successful `create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffOutput {
    /// Output directory.
    pub directory: PathBuf,
    /// Machine-readable handoff.
    pub handoff_json: PathBuf,
    /// Agent instructions and evidence summary.
    pub agent_markdown: PathBuf,
    /// Protocol notes stub.
    pub protocol_markdown: PathBuf,
    /// Validation matrix stub.
    pub validation_markdown: PathBuf,
    /// Test guidance stub.
    pub tests_readme: PathBuf,
    /// Fixture guidance stub.
    pub fixtures_readme: PathBuf,
    /// Commented manifest example; never loadable.
    pub manifest_example: PathBuf,
    /// Exact copy of the validated sanitized report.
    pub evidence_report_json: PathBuf,
    /// Regenerated sanitized Markdown report.
    pub evidence_report_markdown: PathBuf,
}

/// Result of `verify --handoff DIR`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    /// Verified handoff schema version.
    pub handoff_schema_version: u32,
    /// Verified session id.
    pub session_id: String,
    /// Verified report digest.
    pub report_sha256: String,
    /// Recomputed evidence readiness.
    pub readiness: Readiness,
    /// Recomputed hypothesis counts.
    pub counts: HypothesisCounts,
    /// Relative report path that matched its digest.
    pub report_file: String,
}
