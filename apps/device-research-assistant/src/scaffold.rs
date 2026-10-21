//! Evidence-honest driver scaffold generation.
//!
//! The scaffold intentionally emits a commented manifest example and no
//! executable driver code. Unconfirmed capabilities must never become a
//! loadable manifest or a claimed protocol implementation.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::analysis::{self, Analysis};
use crate::error::{DraError, DraResult};
use crate::paths::{DraPaths, to_pretty_json, write_atomic};
use crate::session;

/// Version of the scaffold metadata document.
pub const SCAFFOLD_SCHEMA_VERSION: u32 = 1;

/// Files written by `scaffold`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldOutput {
    /// Scaffold directory.
    pub directory: PathBuf,
    /// Human-readable notes.
    pub readme: PathBuf,
    /// Commented manifest template; never loadable as supplied.
    pub manifest_example: PathBuf,
    /// Test guidance.
    pub tests_readme: PathBuf,
    /// Sanitized hypotheses copied from the analysis.
    pub hypotheses: PathBuf,
}

/// Generate a scaffold from an existing session and its captures.
///
/// A scaffold is produced only when the analysis contains at least one
/// hypothesis. The manifest template is fully commented so it cannot be loaded
/// accidentally, and no capture payload is copied.
pub fn generate(
    paths: &DraPaths,
    session_id: &str,
    output: Option<&Path>,
    force: bool,
    now: u64,
) -> DraResult<(Analysis, ScaffoldOutput)> {
    let session = session::load(paths, session_id)?;
    let analysis = analysis::analyze(paths, session_id, now)?;
    if analysis.hypotheses.is_empty() {
        return Err(DraError::InvalidArgument(
            "analysis produced no hypotheses; scaffold requires correlated evidence".into(),
        ));
    }
    let directory = output
        .map(Path::to_path_buf)
        .unwrap_or(paths.checked_session_dir(session_id)?.join("scaffold"));
    if directory.exists() {
        let has_entries = std::fs::read_dir(&directory)?.next().is_some();
        if has_entries && !force {
            return Err(DraError::OutputNotEmpty(directory.display().to_string()));
        }
    }
    std::fs::create_dir_all(&directory)?;

    let readme = directory.join("README.md");
    let manifest_example = directory.join("manifest.toml.example");
    let tests_readme = directory.join("tests").join("README.md");
    let hypotheses = directory.join("hypotheses.json");

    write_atomic(&readme, notes(&session, &analysis).as_bytes())?;
    write_atomic(&manifest_example, MANIFEST_TEMPLATE.as_bytes())?;
    write_atomic(&tests_readme, TESTS_TEMPLATE.as_bytes())?;
    let hypotheses_document = json!({
        "scaffold_schema_version": SCAFFOLD_SCHEMA_VERSION,
        "session_id": &session.id,
        "evidence_note": "inferred/unverified only; no hardware confirmation is implied",
        "hypotheses": &analysis.hypotheses,
    });
    write_atomic(
        &hypotheses,
        to_pretty_json(&hypotheses_document)?.as_bytes(),
    )?;

    Ok((
        analysis,
        ScaffoldOutput {
            directory,
            readme,
            manifest_example,
            tests_readme,
            hypotheses,
        },
    ))
}

fn notes(session: &crate::model::ResearchSession, analysis: &Analysis) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "# DRA scaffold: {}\n\n",
        session.name.replace(['\n', '\r'], " ")
    ));
    output.push_str(
        "> This is a scaffold, not a driver. It declares no capability and contains no protocol \
         implementation. Every packet meaning below is unverified or inferred from imported \
         text, not confirmed on hardware. Do not load the commented template as a manifest.\n\n",
    );
    output.push_str(&format!("- Session: `{}`\n", session.id));
    output.push_str(&format!("- Device: {}\n", session.device.model));
    output.push_str(&format!("- Goal: {}\n", session.goal));
    output.push_str(&format!(
        "- Analysis: {} captures, {} events, {} hypotheses\n\n",
        analysis.capture_count,
        analysis.event_count,
        analysis.hypotheses.len()
    ));
    output.push_str("## Hypotheses\n\n");
    for hypothesis in &analysis.hypotheses {
        output.push_str(&format!(
            "- **{}** — {}\n",
            hypothesis.confidence.label(),
            hypothesis.statement
        ));
    }
    output.push_str("\n## Required next steps\n\n");
    output.push_str(
        "1. Review the captures and remove unrelated traffic or private data.\n\
         2. Confirm each hypothesis with controlled, authorized hardware experiments.\n\
         3. Record the device and firmware where the evidence was obtained.\n\
         4. Only then write a reviewed declarative or compiled driver with bounded operations.\n\
         5. Add tests before making any compatibility claim.\n",
    );
    output
}

const MANIFEST_TEMPLATE: &str = "\
# Peripheral Hub driver manifest scaffold — NOT a loadable manifest.
#
# Every line is commented out deliberately. The template contains no capability
# and no packet. Uncommenting fields requires reviewed, hardware-confirmed
# evidence and explicit authorization; an inferred offset is not enough.
#
# # [driver]
# # model_id = \"vendor:product\"
# # name = \"replace with the reviewed model name\"
# # sdk_version = \"1.0\"
#
# # [match]
# # vendor_id = 0x0000
# # product_id = 0x0000
# # interface_number = 0
#
# # [capabilities]
# # No capability may be declared from this scaffold.
#
# # [[operations]]
# # No write operation may be declared from this scaffold.
";

const TESTS_TEMPLATE: &str = "\
# Scaffold tests

This directory is intentionally empty of executable tests. Add tests only after
the protocol hypothesis is confirmed and a reviewed driver implementation exists.

Recommended sequence:

1. Create a synthetic fixture from a sanitized capture, not from a raw private file.
2. Assert rejected versions, unknown fields, oversized payloads and invalid bounds.
3. Assert that unsupported operations return an explicit failure, never success.
4. Keep hardware tests explicit, opt-in and separately authorized.
";
