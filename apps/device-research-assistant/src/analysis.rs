//! Deterministic grouping, offset comparison and evidence-labeled hypotheses.
//!
//! The analysis never claims physical validation. It correlates imported
//! payloads and descriptive action labels, emits `inferred` or `unverified`
//! statements, and records the limitations of that reasoning.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::capture;
use crate::error::{DraError, DraResult};
use crate::model::{
    Direction, Event, MAX_PAYLOAD_BYTES, MAX_TOTAL_EVENTS, ResearchSession, Transfer,
    validate_token,
};
use crate::paths::{DraPaths, read_json, to_pretty_json, write_atomic};
use crate::session;

/// Version of the derived `analysis.json` document.
pub const ANALYSIS_SCHEMA_VERSION: u32 = 1;
/// Maximum groups retained in one analysis.
pub const MAX_GROUPS: usize = 512;
/// Maximum varying offsets retained in one analysis.
pub const MAX_VARIABLE_OFFSETS: usize = 512;
/// Maximum hypotheses retained in one analysis.
pub const MAX_HYPOTHESES: usize = 256;

/// Evidence label attached to a hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Human hardware evidence exists; never emitted by automatic analysis.
    ConfirmedOnHardware,
    /// A consistent correlation was found in imported data.
    Inferred,
    /// An observation exists without a one-to-one explanation.
    Unverified,
}

impl Confidence {
    /// Stable lowercase label used in reports.
    pub fn label(self) -> &'static str {
        match self {
            Self::ConfirmedOnHardware => "confirmed-on-hardware",
            Self::Inferred => "inferred",
            Self::Unverified => "unverified",
        }
    }
}

/// Category of an automatically generated hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisKind {
    /// A payload offset appears to encode a labeled action.
    ActionOffset,
    /// An offset varies without a one-to-one action mapping.
    VariableOffset,
    /// An identical outbound payload repeats; purpose is unknown.
    RepeatedPayload,
}

/// Grouping key for normalized outbound/inbound events.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupKey {
    /// Transfer direction.
    pub direction: Direction,
    /// Transfer category.
    pub transfer: Transfer,
    /// Interface number.
    pub interface: u8,
    /// Optional report id.
    #[serde(default)]
    pub report_id: Option<u8>,
    /// Payload length in bytes.
    pub payload_len: usize,
}

impl GroupKey {
    /// Compact label used in reports.
    pub fn label(&self) -> String {
        let report = self
            .report_id
            .map_or_else(|| "none".to_owned(), |id| id.to_string());
        format!(
            "{}/{}/if{}/report={report}/len={}",
            self.direction.label(),
            self.transfer.label(),
            self.interface,
            self.payload_len
        )
    }
}

/// Count of a byte value observed for one action label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionValue {
    /// Descriptive action imported with the event.
    pub action: String,
    /// Byte value observed at the offset.
    pub value: u8,
    /// Number of labeled events with that value.
    pub count: usize,
}

/// One payload offset that varies inside a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariableOffset {
    /// Group where the offset varies.
    pub group: GroupKey,
    /// Zero-based payload offset.
    pub offset: usize,
    /// Distinct byte values in ascending order.
    pub values: Vec<u8>,
    /// Labeled action/value observations, if any.
    pub action_values: Vec<ActionValue>,
}

/// Summary of one transfer group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupSummary {
    /// Group key.
    pub key: GroupKey,
    /// Number of events in the group.
    pub count: usize,
    /// Number of distinct full payloads.
    pub distinct_payloads: usize,
    /// Distinct action labels in sorted order.
    pub actions: Vec<String>,
}

/// One evidence-labeled statement produced by the analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hypothesis {
    /// Evidence label.
    pub confidence: Confidence,
    /// Statement category.
    pub kind: HypothesisKind,
    /// Bounded human-readable statement.
    pub statement: String,
    /// Related group, when the statement is group-specific.
    #[serde(default)]
    pub group: Option<GroupKey>,
    /// Related payload offset, when applicable.
    #[serde(default)]
    pub offset: Option<usize>,
}

/// Derived analysis persisted as `analysis.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Analysis {
    /// Version of this document.
    pub analysis_schema_version: u32,
    /// Session the analysis belongs to.
    pub session_id: String,
    /// Local generation time in Unix seconds.
    pub generated_at_unix: u64,
    /// Number of captures read.
    pub capture_count: usize,
    /// Number of events read across all captures.
    pub event_count: usize,
    /// Whether normalization reordered at least one capture.
    pub reordered: bool,
    /// Group summaries in deterministic order.
    pub groups: Vec<GroupSummary>,
    /// Varying offsets in deterministic order.
    pub variable_offsets: Vec<VariableOffset>,
    /// Evidence-labeled hypotheses.
    pub hypotheses: Vec<Hypothesis>,
    /// Limits and interpretation notes.
    pub limitations: Vec<String>,
}

impl Analysis {
    /// Validate the derived document before loading or writing it.
    pub fn validate(&self) -> DraResult<()> {
        if self.analysis_schema_version != ANALYSIS_SCHEMA_VERSION {
            return Err(DraError::UnsupportedVersion {
                kind: "analysis",
                found: self.analysis_schema_version,
                expected: ANALYSIS_SCHEMA_VERSION,
            });
        }
        validate_token(&self.session_id, "session id", crate::model::MAX_SESSION_ID)?;
        if self.capture_count == 0 || self.capture_count > crate::model::MAX_CAPTURES {
            return Err(DraError::InvalidArgument(
                "analysis capture count is out of range".into(),
            ));
        }
        if self.event_count == 0 || self.event_count > MAX_TOTAL_EVENTS {
            return Err(DraError::InvalidArgument(
                "analysis event count is out of range".into(),
            ));
        }
        if self.groups.len() > MAX_GROUPS
            || self.variable_offsets.len() > MAX_VARIABLE_OFFSETS
            || self.hypotheses.len() > MAX_HYPOTHESES
        {
            return Err(DraError::InvalidArgument(
                "analysis exceeds a retained-item limit".into(),
            ));
        }
        for summary in &self.groups {
            if summary.key.payload_len > MAX_PAYLOAD_BYTES || summary.count == 0 {
                return Err(DraError::InvalidArgument(
                    "analysis group is out of range".into(),
                ));
            }
        }
        for offset in &self.variable_offsets {
            if offset.offset >= offset.group.payload_len {
                return Err(DraError::InvalidArgument(
                    "analysis offset is outside its group payload".into(),
                ));
            }
        }
        if self.hypotheses.iter().any(|hypothesis| {
            hypothesis.statement.is_empty() || hypothesis.statement.chars().count() > 1024
        }) {
            return Err(DraError::InvalidArgument(
                "analysis hypothesis statement is out of range".into(),
            ));
        }
        Ok(())
    }
}

/// Path of the derived analysis document.
pub fn analysis_file(paths: &DraPaths, id: &str) -> DraResult<std::path::PathBuf> {
    Ok(paths.checked_session_dir(id)?.join("analysis.json"))
}

/// Load a previously written analysis.
pub fn load(paths: &DraPaths, id: &str) -> DraResult<Analysis> {
    let analysis: Analysis = read_json(&analysis_file(paths, id)?, 4 * 1024 * 1024)?;
    analysis.validate()?;
    if analysis.session_id != id {
        return Err(DraError::InvalidArgument(
            "analysis session id does not match the requested session".into(),
        ));
    }
    Ok(analysis)
}

/// Analyze every capture in a session and atomically persist `analysis.json`.
pub fn analyze(paths: &DraPaths, session_id: &str, now: u64) -> DraResult<Analysis> {
    let session = session::load(paths, session_id)?;
    let captures = capture::load_all(paths, &session)?;
    if captures.is_empty() {
        return Err(DraError::NoCapture(session.id.clone()));
    }
    let analysis = build_analysis(&session, &captures, now)?;
    analysis.validate()?;
    write_atomic(
        &analysis_file(paths, session_id)?,
        to_pretty_json(&analysis)?.as_bytes(),
    )?;
    Ok(analysis)
}

fn build_analysis(
    session: &ResearchSession,
    captures: &[(crate::model::CaptureMeta, crate::model::NormalizedCapture)],
    now: u64,
) -> DraResult<Analysis> {
    let mut merged: Vec<(usize, Event)> = Vec::new();
    let mut reordered = false;
    for (capture_index, (meta, normalized)) in captures.iter().enumerate() {
        reordered |= meta.reordered || normalized.reordered;
        for event in &normalized.events {
            merged.push((capture_index, event.clone()));
        }
    }
    if merged.len() > MAX_TOTAL_EVENTS {
        return Err(DraError::LimitExceeded {
            kind: "total analyzed events".to_owned(),
            limit: MAX_TOTAL_EVENTS,
        });
    }
    merged.sort_by_key(|(capture_index, event)| (event.timestamp_us, *capture_index, event.index));

    let mut grouped: BTreeMap<GroupKey, Vec<usize>> = BTreeMap::new();
    for (position, (_, event)) in merged.iter().enumerate() {
        grouped
            .entry(GroupKey {
                direction: event.direction,
                transfer: event.transfer,
                interface: event.interface,
                report_id: event.report_id,
                payload_len: event.payload.len(),
            })
            .or_default()
            .push(position);
    }

    let mut groups = Vec::new();
    let mut variable_offsets = Vec::new();
    let mut hypotheses = Vec::new();
    let mut seen_statements = BTreeSet::new();
    let mut truncated = false;
    let group_count = grouped.len();

    for (key, positions) in grouped.iter().take(MAX_GROUPS) {
        let events: Vec<&Event> = positions
            .iter()
            .map(|position| &merged[*position].1)
            .collect();
        let distinct_payloads: BTreeSet<&[u8]> = events
            .iter()
            .map(|event| event.payload.as_slice())
            .collect();
        let actions: BTreeSet<&str> = events
            .iter()
            .filter_map(|event| event.action.as_deref())
            .collect();
        groups.push(GroupSummary {
            key: key.clone(),
            count: events.len(),
            distinct_payloads: distinct_payloads.len(),
            actions: actions.iter().map(|action| (*action).to_owned()).collect(),
        });

        let mut group_variable_offsets = 0usize;
        if key.direction == Direction::Out {
            for offset in 0..key.payload_len {
                let values: BTreeSet<u8> =
                    events.iter().map(|event| event.payload[offset]).collect();
                if values.len() <= 1 {
                    continue;
                }
                group_variable_offsets += 1;
                let mut action_counts: BTreeMap<(&str, u8), usize> = BTreeMap::new();
                for event in &events {
                    if let Some(action) = event.action.as_deref() {
                        *action_counts
                            .entry((action, event.payload[offset]))
                            .or_default() += 1;
                    }
                }
                let action_values: Vec<ActionValue> = action_counts
                    .into_iter()
                    .map(|((action, value), count)| ActionValue {
                        action: action.to_owned(),
                        value,
                        count,
                    })
                    .collect();
                if variable_offsets.len() < MAX_VARIABLE_OFFSETS {
                    variable_offsets.push(VariableOffset {
                        group: key.clone(),
                        offset,
                        values: values.iter().copied().collect(),
                        action_values: action_values.clone(),
                    });
                } else {
                    truncated = true;
                }

                if !action_values.is_empty() {
                    push_offset_hypothesis(
                        &mut hypotheses,
                        &mut seen_statements,
                        key,
                        offset,
                        &action_values,
                    );
                }
            }
        }

        if group_variable_offsets > 0 && events.iter().all(|event| event.action.is_none()) {
            let statement = format!(
                "Group {} has {group_variable_offsets} varying offset(s) but no labeled actions; their meaning is unverified.",
                key.label()
            );
            push_hypothesis(
                &mut hypotheses,
                &mut seen_statements,
                Hypothesis {
                    confidence: Confidence::Unverified,
                    kind: HypothesisKind::VariableOffset,
                    statement,
                    group: Some(key.clone()),
                    offset: None,
                },
            );
        }
        if distinct_payloads.len() == 1 && events.len() >= 2 {
            let statement = format!(
                "Group {} repeats one outbound payload {} times; it may be polling or a heartbeat, but its purpose is unverified.",
                key.label(),
                events.len()
            );
            push_hypothesis(
                &mut hypotheses,
                &mut seen_statements,
                Hypothesis {
                    confidence: Confidence::Unverified,
                    kind: HypothesisKind::RepeatedPayload,
                    statement,
                    group: Some(key.clone()),
                    offset: None,
                },
            );
        }
    }
    if group_count > MAX_GROUPS {
        truncated = true;
    }

    let mut limitations = vec![
        "Hypotheses are correlations from imported text; none is confirmed on hardware.".to_owned(),
        "confirmed-on-hardware is never inferred automatically; record human evidence separately."
            .to_owned(),
        "No packet was replayed or sent, and no vendor executable was run.".to_owned(),
        "Framing, checksum, sequencing, timing and multi-packet state are not modeled.".to_owned(),
    ];
    if reordered {
        limitations
            .push("At least one capture was reordered by timestamp during normalization.".into());
    }
    if truncated {
        limitations.push(format!(
            "Analysis retained at most {MAX_GROUPS} groups, {MAX_VARIABLE_OFFSETS} varying offsets and {MAX_HYPOTHESES} hypotheses."
        ));
    }
    if variable_offsets.is_empty() {
        limitations.push(
            "No varying outbound offsets were found; controlled, labeled captures are still required."
                .into(),
        );
    }

    Ok(Analysis {
        analysis_schema_version: ANALYSIS_SCHEMA_VERSION,
        session_id: session.id.clone(),
        generated_at_unix: now,
        capture_count: captures.len(),
        event_count: merged.len(),
        reordered,
        groups,
        variable_offsets,
        hypotheses,
        limitations,
    })
}

fn push_offset_hypothesis(
    hypotheses: &mut Vec<Hypothesis>,
    seen: &mut BTreeSet<String>,
    key: &GroupKey,
    offset: usize,
    action_values: &[ActionValue],
) {
    let mut action_map: BTreeMap<&str, BTreeSet<u8>> = BTreeMap::new();
    let mut value_map: BTreeMap<u8, BTreeSet<&str>> = BTreeMap::new();
    for observation in action_values {
        action_map
            .entry(observation.action.as_str())
            .or_default()
            .insert(observation.value);
        value_map
            .entry(observation.value)
            .or_default()
            .insert(observation.action.as_str());
    }
    let consistent = action_map.len() >= 2
        && value_map.len() >= 2
        && action_map.values().all(|values| values.len() == 1)
        && value_map.values().all(|actions| actions.len() == 1);
    let (confidence, statement) = if consistent {
        let pairs: Vec<String> = action_map
            .iter()
            .take(4)
            .map(|(action, values)| {
                let value = values.iter().next().copied().unwrap_or(0);
                format!("action {} maps to 0x{value:02x}", quote(action))
            })
            .collect();
        (
            Confidence::Inferred,
            format!(
                "Offset {offset} may encode action identity in group {}: {}.",
                key.label(),
                pairs.join("; ")
            ),
        )
    } else {
        (
            Confidence::Unverified,
            format!(
                "Offset {offset} varies with labeled actions in group {}, but the mapping is not one-to-one; further controlled captures are required.",
                key.label()
            ),
        )
    };
    push_hypothesis(
        hypotheses,
        seen,
        Hypothesis {
            confidence,
            kind: HypothesisKind::ActionOffset,
            statement,
            group: Some(key.clone()),
            offset: Some(offset),
        },
    );
}

fn push_hypothesis(
    hypotheses: &mut Vec<Hypothesis>,
    seen: &mut BTreeSet<String>,
    hypothesis: Hypothesis,
) {
    if hypotheses.len() >= MAX_HYPOTHESES || !seen.insert(hypothesis.statement.clone()) {
        return;
    }
    hypotheses.push(hypothesis);
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"action\"".to_owned())
}
