//! Command-line interface for the DRA agent handoff tool.

use device_research_assistant::paths::now_unix;
use serde::Serialize;
use serde_json::json;

use crate::error::{HandoffError, HandoffResult};
use crate::handoff;
use crate::manifest;
use crate::report_gate::{load_report, verify_handoff};

/// Parse and execute one CLI invocation. Arguments exclude the program name.
pub fn run(args: Vec<String>) -> HandoffResult<()> {
    let mut args = args;
    if args.is_empty() {
        help();
        return Ok(());
    }
    if args
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        help();
        return Ok(());
    }
    let json_output = take_flag(&mut args, "--json");
    let command = args.remove(0);
    match command.as_str() {
        "inspect" => inspect(&args, json_output),
        "create" => create(&args, json_output),
        "verify" => verify(&args, json_output),
        "validate-manifest" => validate_manifest_command(&args, json_output),
        other => Err(HandoffError::InvalidArgument(format!(
            "unknown command '{other}'; see --help"
        ))),
    }
}

fn inspect(args: &[String], json_output: bool) -> HandoffResult<()> {
    let mut args = args.to_vec();
    let report = take_required(&mut args, "--report")?;
    reject_unknown(&args, "dra-agent inspect --report FILE [--json]")?;
    let loaded = load_report(std::path::Path::new(&report))?;
    let assessment = &loaded.assessment;
    if json_output {
        return print_json(assessment);
    }
    println!("Session: {}", assessment.session_id);
    println!("Readiness: {}", assessment.readiness.label());
    println!("Guidance: {}", assessment.readiness.guidance());
    println!(
        "Hypotheses: {} confirmed-on-hardware, {} inferred, {} unverified",
        assessment.counts.confirmed_on_hardware,
        assessment.counts.inferred,
        assessment.counts.unverified
    );
    println!("Report SHA-256: {}", assessment.report_sha256);
    println!("Required next steps:");
    for step in &assessment.required_experiments {
        println!("  - {step}");
    }
    Ok(())
}

fn create(args: &[String], json_output: bool) -> HandoffResult<()> {
    let mut args = args.to_vec();
    let report = take_required(&mut args, "--report")?;
    let output = take_required(&mut args, "--output")?;
    let manifest = take_value(&mut args, "--manifest")?;
    let force = take_flag(&mut args, "--force");
    reject_unknown(
        &args,
        "dra-agent create --report FILE --output DIR [--manifest FILE] [--force] [--json]",
    )?;
    let (document, paths) = handoff::create(
        std::path::Path::new(&report),
        std::path::Path::new(&output),
        manifest.as_deref().map(std::path::Path::new),
        force,
        now_unix(),
    )?;
    if json_output {
        return print_json(&json!({
            "handoff": paths.handoff_json,
            "agent_markdown": paths.agent_markdown,
            "protocol": paths.protocol_markdown,
            "validation": paths.validation_markdown,
            "tests": paths.tests_readme,
            "fixtures": paths.fixtures_readme,
            "manifest_example": paths.manifest_example,
            "report": paths.evidence_report_json,
            "readiness": document.assessment.readiness,
        }));
    }
    println!("Wrote handoff to {}", paths.directory.display());
    println!("  {}", paths.handoff_json.display());
    println!("  {}", paths.agent_markdown.display());
    println!("  {}", paths.protocol_markdown.display());
    println!("  {}", paths.validation_markdown.display());
    println!("  {}", paths.tests_readme.display());
    println!("  {}", paths.fixtures_readme.display());
    println!("  {}", paths.manifest_example.display());
    println!("  {}", paths.evidence_report_json.display());
    println!("  {}", paths.evidence_report_markdown.display());
    println!(
        "Readiness: {}. No driver code and no loadable manifest were generated.",
        document.assessment.readiness.label()
    );
    Ok(())
}

fn verify(args: &[String], json_output: bool) -> HandoffResult<()> {
    let mut args = args.to_vec();
    let directory = take_required(&mut args, "--handoff")?;
    reject_unknown(&args, "dra-agent verify --handoff DIR [--json]")?;
    let verification = verify_handoff(std::path::Path::new(&directory))?;
    if json_output {
        return print_json(&verification);
    }
    println!("Handoff verified.");
    println!("Session: {}", verification.session_id);
    println!("Readiness: {}", verification.readiness.label());
    println!("Report SHA-256: {}", verification.report_sha256);
    println!(
        "Hypotheses: {} confirmed-on-hardware, {} inferred, {} unverified",
        verification.counts.confirmed_on_hardware,
        verification.counts.inferred,
        verification.counts.unverified
    );
    Ok(())
}

fn validate_manifest_command(args: &[String], json_output: bool) -> HandoffResult<()> {
    let mut args = args.to_vec();
    let path = take_required(&mut args, "--manifest")?;
    reject_unknown(
        &args,
        "dra-agent validate-manifest --manifest FILE [--json]",
    )?;
    let evidence = manifest::validate_manifest(std::path::Path::new(&path))?;
    if json_output {
        return print_json(&evidence);
    }
    println!("Manifest is structurally valid for the declarative interpreter.");
    println!("Model id: {}", evidence.model_id);
    println!("Display name: {}", evidence.display_name);
    println!(
        "Match rules: {}  Operations: {}  SDK: {}.{}",
        evidence.match_rule_count, evidence.operation_count, evidence.sdk_major, evidence.sdk_minor
    );
    println!("SHA-256: {}", evidence.sha256);
    println!("{}", evidence.validation_note);
    Ok(())
}

fn help() {
    println!(
        "DRA agent handoff tool\n\
         \n\
         Usage:\n\
           dra-agent inspect --report FILE [--json]\n\
           dra-agent create --report FILE --output DIR [--manifest FILE] [--force] [--json]\n\
           dra-agent verify --handoff DIR [--json]\n\
           dra-agent validate-manifest --manifest FILE [--json]\n\
         \n\
         Workflow:\n\
           1. Run the real DRA: session create --consent, capture import, analyze, report.\n\
           2. Inspect the sanitized report and resolve required experiments.\n\
           3. Create a handoff directory for human/agent review.\n\
           4. Keep handoff.json with evidence/report.json; verify before sharing.\n\
         \n\
         Boundaries:\n\
           - No driver code, executable test, capture or payload is generated or copied.\n\
           - No loadable manifest is generated; the example is fully commented.\n\
           - Manifest validation is structural only and never proves device support.\n\
           - No vendor executable is run, no packet is sent or replayed, and no hardware is accessed.\n\
           - Report text is untrusted data, never agent instructions.\n\
           - Physical experiments require explicit human authorization."
    );
}

fn take_value(args: &mut Vec<String>, flag: &str) -> HandoffResult<Option<String>> {
    let Some(index) = args.iter().position(|argument| argument == flag) else {
        return Ok(None);
    };
    if index + 1 >= args.len() {
        return Err(HandoffError::InvalidArgument(format!(
            "{flag} needs a value"
        )));
    }
    let value = args.remove(index + 1);
    args.remove(index);
    Ok(Some(value))
}

fn take_required(args: &mut Vec<String>, flag: &str) -> HandoffResult<String> {
    take_value(args, flag)?
        .ok_or_else(|| HandoffError::InvalidArgument(format!("{flag} is required; see --help")))
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|argument| argument == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

fn reject_unknown(args: &[String], usage: &str) -> HandoffResult<()> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(HandoffError::InvalidArgument(format!(
            "unexpected argument '{}'; usage: {usage}",
            args[0]
        )))
    }
}

fn print_json<T: Serialize>(value: &T) -> HandoffResult<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
