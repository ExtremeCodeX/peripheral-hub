//! Sanitized Markdown and JSON research reports derived from an analysis.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::analysis::{self, Analysis, Confidence};
use crate::error::{DraError, DraResult};
use crate::model::ResearchSession;
use crate::paths::{DraPaths, to_pretty_json, write_atomic};
use crate::session;

/// Version of the exported `report.json` document.
pub const REPORT_SCHEMA_VERSION: u32 = 1;

/// Machine-readable report written next to the Markdown export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDocument {
    /// Version of this document.
    pub report_schema_version: u32,
    /// Local generation time in Unix seconds.
    pub generated_at_unix: u64,
    /// Sanitized session metadata (no payloads).
    pub session: ResearchSession,
    /// Analysis used to produce the report.
    pub analysis: Analysis,
}

impl ReportDocument {
    /// Validate consistency between the session and its analysis.
    pub fn validate(&self) -> DraResult<()> {
        if self.report_schema_version != REPORT_SCHEMA_VERSION {
            return Err(DraError::UnsupportedVersion {
                kind: "report",
                found: self.report_schema_version,
                expected: REPORT_SCHEMA_VERSION,
            });
        }
        self.session.validate()?;
        self.analysis.validate()?;
        if self.session.id != self.analysis.session_id {
            return Err(DraError::InvalidArgument(
                "report session and analysis ids differ".into(),
            ));
        }
        if self.session.captures.len() != self.analysis.capture_count {
            return Err(DraError::InvalidArgument(
                "report capture count disagrees with the analysis".into(),
            ));
        }
        Ok(())
    }

    /// Render a sanitized Markdown report.
    pub fn to_markdown(&self) -> String {
        render_markdown(self)
    }
}

/// Paths written by `report`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportOutput {
    /// Markdown report path.
    pub markdown: PathBuf,
    /// JSON report path.
    pub json: PathBuf,
}

/// Recompute the analysis and export Markdown plus JSON.
///
/// The default destination is the session directory. An explicit output
/// directory is created if needed; derived files may be regenerated.
pub fn generate(
    paths: &DraPaths,
    session_id: &str,
    output: Option<&Path>,
    now: u64,
) -> DraResult<(ReportDocument, ReportOutput)> {
    let session = session::load(paths, session_id)?;
    let analysis = analysis::analyze(paths, session_id, now)?;
    let document = ReportDocument {
        report_schema_version: REPORT_SCHEMA_VERSION,
        generated_at_unix: now,
        session,
        analysis,
    };
    document.validate()?;
    let directory = output
        .map(Path::to_path_buf)
        .unwrap_or(paths.checked_session_dir(session_id)?);
    std::fs::create_dir_all(&directory)?;
    let markdown = directory.join("report.md");
    let json = directory.join("report.json");
    write_atomic(&markdown, document.to_markdown().as_bytes())?;
    write_atomic(&json, to_pretty_json(&document)?.as_bytes())?;
    Ok((document, ReportOutput { markdown, json }))
}

fn render_markdown(document: &ReportDocument) -> String {
    let session = &document.session;
    let analysis = &document.analysis;
    let mut output = String::new();
    output.push_str(&format!(
        "# Research report: {}\n\n",
        escape_inline(&session.name)
    ));
    output.push_str(&format!("- Session: `{}`\n", escape_inline(&session.id)));
    output.push_str(&format!(
        "- Created (Unix seconds): {}\n",
        session.created_at_unix
    ));
    let firmware = session
        .device
        .firmware
        .as_deref()
        .map_or_else(|| "not recorded".to_owned(), escape_inline);
    output.push_str(&format!(
        "- Device: {} (firmware: {firmware})\n",
        escape_inline(&session.device.model)
    ));
    match &session.software {
        Some(software) => {
            let sha = software
                .sha256
                .as_deref()
                .map_or_else(|| "not recorded".to_owned(), escape_inline);
            output.push_str(&format!(
                "- Software: {} (SHA-256: {sha})\n",
                escape_inline(&software.name)
            ));
        }
        None => output.push_str("- Software: not recorded\n"),
    }
    output.push_str(&format!("- Goal: {}\n", escape_inline(&session.goal)));
    output.push_str(&format!(
        "- Generated (Unix seconds): {}\n\n",
        document.generated_at_unix
    ));
    output.push_str(
        "> Evidence labels: `inferred` and `unverified` are correlations from imported data. \
         `confirmed-on-hardware` is never inferred automatically. No packet was replayed or sent, \
         and no vendor executable was run.\n\n",
    );

    output.push_str("## Captures\n\n");
    output.push_str("| # | Source | Events | Description |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for (index, capture) in session.captures.iter().enumerate() {
        let description = capture
            .description
            .as_deref()
            .map_or_else(|| "—".to_owned(), escape_cell);
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            index + 1,
            escape_cell(&capture.source),
            capture.event_count,
            description
        ));
    }
    output.push('\n');

    output.push_str("## Transfer groups\n\n");
    output.push_str("| Group | Events | Distinct payloads | Actions |\n");
    output.push_str("| --- | --- | --- | --- |\n");
    for group in &analysis.groups {
        let actions = if group.actions.is_empty() {
            "—".to_owned()
        } else {
            group
                .actions
                .iter()
                .map(|action| escape_cell(action))
                .collect::<Vec<_>>()
                .join(", ")
        };
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            escape_cell(&group.key.label()),
            group.count,
            group.distinct_payloads,
            actions
        ));
    }
    output.push('\n');

    output.push_str("## Varying offsets\n\n");
    if analysis.variable_offsets.is_empty() {
        output.push_str("No varying outbound offsets were recorded.\n\n");
    } else {
        output.push_str("| Group | Offset | Values | Labeled actions |\n");
        output.push_str("| --- | --- | --- | --- |\n");
        for offset in &analysis.variable_offsets {
            let values = offset
                .values
                .iter()
                .map(|value| format!("0x{value:02x}"))
                .collect::<Vec<_>>()
                .join(", ");
            let actions = if offset.action_values.is_empty() {
                "—".to_owned()
            } else {
                offset
                    .action_values
                    .iter()
                    .map(|observation| {
                        format!(
                            "{} → 0x{:02x} (×{})",
                            escape_cell(&observation.action),
                            observation.value,
                            observation.count
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                escape_cell(&offset.group.label()),
                offset.offset,
                values,
                actions
            ));
        }
        output.push('\n');
    }

    output.push_str("## Hypotheses\n\n");
    if analysis.hypotheses.is_empty() {
        output.push_str(
            "No hypotheses were produced. Labeled, controlled captures that differ in one action at a time are still required.\n\n",
        );
    } else {
        for hypothesis in &analysis.hypotheses {
            let label = match hypothesis.confidence {
                Confidence::ConfirmedOnHardware => "confirmed-on-hardware",
                Confidence::Inferred => "inferred",
                Confidence::Unverified => "unverified",
            };
            output.push_str(&format!(
                "- **{label}** — {}\n",
                escape_inline(&hypothesis.statement)
            ));
        }
        output.push('\n');
    }

    output.push_str("## Limitations\n\n");
    for limitation in &analysis.limitations {
        output.push_str(&format!("- {}\n", escape_inline(limitation)));
    }
    output.push('\n');
    output.push_str(
        "This report is a sanitized research aid. It does not prove hardware compatibility, \
         and any driver support still requires reviewed evidence and explicit authorization.\n",
    );
    output
}

fn escape_cell(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '|' => output.push_str("\\|"),
            '\n' | '\r' => output.push(' '),
            _ => output.push(character),
        }
    }
    output
}

fn escape_inline(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '#' | '|' => {
                output.push('\\');
                output.push(character);
            }
            '\n' | '\r' => output.push(' '),
            _ => output.push(character),
        }
    }
    output
}
