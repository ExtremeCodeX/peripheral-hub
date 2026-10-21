//! Command-line interface for the standalone Device Research Assistant.

use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;

use crate::analysis::{self, Analysis, Confidence};
use crate::capture;
use crate::error::{DraError, DraResult};
use crate::inventory;
use crate::paths::{DraPaths, default_data_root, now_unix};
use crate::report;
use crate::scaffold;
use crate::session::{self, SessionSpec};

/// Parse and execute one CLI invocation.
///
/// Arguments exclude the program name. The function performs only local file
/// work; `devices list` is the sole command that may enumerate hardware and it
/// never opens a device or writes a report.
pub fn run(args: Vec<String>) -> DraResult<()> {
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
    let data_dir = take_value(&mut args, "--data-dir")?.map(PathBuf::from);
    let json_output = take_flag(&mut args, "--json");
    if args.is_empty() {
        help();
        return Ok(());
    }
    let paths = DraPaths::new(data_dir);
    let command = args.remove(0);
    match command.as_str() {
        "devices" => devices(&paths, &args, json_output),
        "session" => sessions(&paths, &args, json_output),
        "capture" => captures(&paths, &args, json_output),
        "analyze" => analyze(&paths, &args, json_output),
        "report" => report_command(&paths, &args, json_output),
        "scaffold" => scaffold_command(&paths, &args, json_output),
        other => Err(DraError::InvalidArgument(format!(
            "unknown command '{other}'; see --help"
        ))),
    }
}

fn devices(paths: &DraPaths, args: &[String], json_output: bool) -> DraResult<()> {
    let _ = paths;
    match args {
        [command] if command == "list" => {}
        [] => return Err(DraError::InvalidArgument("usage: dra devices list".into())),
        _ => return Err(DraError::InvalidArgument("usage: dra devices list".into())),
    }
    let entries = inventory::enumerate()?;
    if json_output {
        return print_json(&entries);
    }
    if entries.is_empty() {
        println!("No HID interfaces observed.");
        return Ok(());
    }
    println!(
        "{:<9}  {:<4}  {:<9}  {:<10}  DESCRIPTION",
        "VID:PID", "IF", "USAGE", "TRANSPORT"
    );
    for entry in entries {
        let usage = entry.usage.as_ref().map_or_else(
            || "—".to_owned(),
            |usage| format!("{:04x}:{:04x}", usage.page, usage.usage),
        );
        let description = [entry.manufacturer.as_deref(), entry.product.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{:04x}:{:04x}  {:<4}  {:<9}  {:<10}  {}",
            entry.vendor_id,
            entry.product_id,
            entry
                .interface_number
                .map_or_else(|| "—".to_owned(), |number| number.to_string()),
            usage,
            entry.transport,
            description
        );
    }
    Ok(())
}

fn sessions(paths: &DraPaths, args: &[String], json_output: bool) -> DraResult<()> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(DraError::InvalidArgument(
            "usage: dra session create|list|show ...".into(),
        ));
    };
    match subcommand.as_str() {
        "list" if rest.is_empty() => {
            let sessions = session::list(paths)?;
            if json_output {
                return print_json(&sessions);
            }
            if sessions.is_empty() {
                println!(
                    "No research sessions in {}.",
                    paths.sessions_dir().display()
                );
                return Ok(());
            }
            println!("{:<28}  {:<12}  {:<8}  DEVICE", "ID", "CREATED", "CAPTURES");
            for session in sessions {
                println!(
                    "{:<28}  {:<12}  {:<8}  {}",
                    session.id,
                    session.created_at_unix,
                    session.captures.len(),
                    session.device.model
                );
            }
            Ok(())
        }
        "show" => {
            let [id] = rest else {
                return Err(DraError::InvalidArgument(
                    "usage: dra session show ID".into(),
                ));
            };
            let session = session::load(paths, id)?;
            if json_output {
                return print_json(&session);
            }
            println!("Session: {}", session.id);
            println!("Name: {}", session.name);
            println!("Created (Unix): {}", session.created_at_unix);
            println!("Device: {}", session.device.model);
            if let Some(firmware) = &session.device.firmware {
                println!("Firmware: {firmware}");
            }
            if let Some(software) = &session.software {
                println!("Software: {}", software.name);
                if let Some(sha) = &software.sha256 {
                    println!("SHA-256: {sha}");
                }
            }
            println!("Goal: {}", session.goal);
            println!("Consent recorded: yes");
            if session.captures.is_empty() {
                println!("Captures: none");
            } else {
                println!("Captures:");
                for capture in session.captures {
                    println!(
                        "  {}  {} events  source={}  imported={}",
                        capture.file, capture.event_count, capture.source, capture.imported_at_unix
                    );
                }
            }
            Ok(())
        }
        "create" => {
            let mut args = rest.to_vec();
            let name = take_required(&mut args, "--name")?;
            let device = take_required(&mut args, "--device")?;
            let goal = take_required(&mut args, "--goal")?;
            let id = take_value(&mut args, "--id")?;
            let firmware = take_value(&mut args, "--firmware")?;
            let software = take_value(&mut args, "--software")?;
            let sha256 = take_value(&mut args, "--sha256")?;
            let consent = take_flag(&mut args, "--consent");
            if !consent {
                return Err(DraError::ConsentRequired);
            }
            reject_unknown(&args, "dra session create --help")?;
            let session = session::create(
                paths,
                SessionSpec {
                    id,
                    name,
                    device,
                    firmware,
                    software,
                    sha256,
                    goal,
                    consent,
                },
                now_unix(),
            )?;
            if json_output {
                return print_json(&session);
            }
            println!(
                "Created session '{}' at {}",
                session.id,
                paths.session_dir(&session.id).display()
            );
            Ok(())
        }
        _ => Err(DraError::InvalidArgument(
            "usage: dra session create|list|show ...".into(),
        )),
    }
}

fn captures(paths: &DraPaths, args: &[String], json_output: bool) -> DraResult<()> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(DraError::InvalidArgument(
            "usage: dra capture import --session ID --input FILE".into(),
        ));
    };
    match subcommand.as_str() {
        "import" => {
            let mut args = rest.to_vec();
            let session_id = take_required(&mut args, "--session")?;
            let input = take_required(&mut args, "--input")?;
            reject_unknown(&args, "dra capture import --session ID --input FILE")?;
            let (session, meta) =
                capture::import(paths, &session_id, std::path::Path::new(&input), now_unix())?;
            if json_output {
                return print_json(&json!({
                    "session": &session,
                    "capture": &meta,
                }));
            }
            println!(
                "Imported {} into session '{}' ({} events).",
                meta.file, session.id, meta.event_count
            );
            Ok(())
        }
        _ => Err(DraError::InvalidArgument(
            "usage: dra capture import --session ID --input FILE".into(),
        )),
    }
}

fn analyze(paths: &DraPaths, args: &[String], json_output: bool) -> DraResult<()> {
    let mut args = args.to_vec();
    let session_id = take_required(&mut args, "--session")?;
    reject_unknown(&args, "dra analyze --session ID")?;
    let analysis = analysis::analyze(paths, &session_id, now_unix())?;
    if json_output {
        return print_json(&analysis);
    }
    print_analysis(&analysis);
    Ok(())
}

fn report_command(paths: &DraPaths, args: &[String], json_output: bool) -> DraResult<()> {
    let mut args = args.to_vec();
    let session_id = take_required(&mut args, "--session")?;
    let output = take_value(&mut args, "--output")?.map(PathBuf::from);
    reject_unknown(&args, "dra report --session ID [--output DIR]")?;
    let (document, output) = report::generate(paths, &session_id, output.as_deref(), now_unix())?;
    if json_output {
        return print_json(&json!({
            "markdown": output.markdown,
            "json": output.json,
            "hypotheses": document.analysis.hypotheses.len(),
        }));
    }
    println!("Wrote {}", output.markdown.display());
    println!("Wrote {}", output.json.display());
    println!(
        "Hypotheses: {} (inferred/unverified only; no hardware confirmation)",
        document.analysis.hypotheses.len()
    );
    Ok(())
}

fn scaffold_command(paths: &DraPaths, args: &[String], json_output: bool) -> DraResult<()> {
    let mut args = args.to_vec();
    let session_id = take_required(&mut args, "--session")?;
    let output = take_value(&mut args, "--output")?.map(PathBuf::from);
    let force = take_flag(&mut args, "--force");
    reject_unknown(&args, "dra scaffold --session ID [--output DIR] [--force]")?;
    let (analysis, output) =
        scaffold::generate(paths, &session_id, output.as_deref(), force, now_unix())?;
    if json_output {
        return print_json(&json!({
            "directory": output.directory,
            "readme": output.readme,
            "manifest_example": output.manifest_example,
            "tests_readme": output.tests_readme,
            "hypotheses": output.hypotheses,
            "hypothesis_count": analysis.hypotheses.len(),
        }));
    }
    println!("Wrote scaffold to {}", output.directory.display());
    println!("  {}", output.readme.display());
    println!("  {}", output.manifest_example.display());
    println!("  {}", output.tests_readme.display());
    println!("  {}", output.hypotheses.display());
    println!(
        "The manifest template is fully commented and is not a driver; {} hypotheses remain unconfirmed.",
        analysis.hypotheses.len()
    );
    Ok(())
}

fn print_analysis(analysis: &Analysis) {
    println!("Session: {}", analysis.session_id);
    println!(
        "Captures: {}  Events: {}  Groups: {}  Varying offsets: {}",
        analysis.capture_count,
        analysis.event_count,
        analysis.groups.len(),
        analysis.variable_offsets.len()
    );
    let inferred = analysis
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.confidence == Confidence::Inferred)
        .count();
    let unverified = analysis
        .hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.confidence == Confidence::Unverified)
        .count();
    println!("Hypotheses: {inferred} inferred, {unverified} unverified");
    for hypothesis in &analysis.hypotheses {
        println!(
            "  [{}] {}",
            hypothesis.confidence.label(),
            hypothesis.statement
        );
    }
    println!("No hypothesis was confirmed on hardware.");
}

fn help() {
    println!(
        "Device Research Assistant (DRA)\n\
         \n\
         Usage:\n\
           dra devices list [--json]\n\
           dra session create --name NAME --device MODEL --goal TEXT --consent\n\
                              [--id ID] [--firmware FW] [--software NAME] [--sha256 HEX] [--json]\n\
           dra session list [--json]\n\
           dra session show ID [--json]\n\
           dra capture import --session ID --input FILE [--json]\n\
           dra analyze --session ID [--json]\n\
           dra report --session ID [--output DIR] [--json]\n\
           dra scaffold --session ID [--output DIR] [--force] [--json]\n\
           dra --data-dir PATH ...\n\
         \n\
         Data root: {}\n\
         Override with --data-dir or PERIPHERAL_HUB_DRA_DATA; sessions are local files.\n\
         \n\
         Boundaries:\n\
           - No vendor executable is run and no packet is sent or replayed.\n\
           - Capture import accepts the project's stable JSON format only; raw captures\n\
             and private paths stay in ignored local storage.\n\
           - Analysis emits inferred/unverified correlations. It never emits\n\
             confirmed-on-hardware automatically.\n\
           - The scaffold is a commented template, not a driver or a capability claim.\n\
           - devices list is a non-destructive HID enumeration; it opens no handle.",
        default_data_root().display()
    );
}

fn take_value(args: &mut Vec<String>, flag: &str) -> DraResult<Option<String>> {
    let Some(index) = args.iter().position(|argument| argument == flag) else {
        return Ok(None);
    };
    if index + 1 >= args.len() {
        return Err(DraError::InvalidArgument(format!("{flag} needs a value")));
    }
    let value = args.remove(index + 1);
    args.remove(index);
    Ok(Some(value))
}

fn take_required(args: &mut Vec<String>, flag: &str) -> DraResult<String> {
    take_value(args, flag)?.ok_or_else(|| DraError::InvalidArgument(format!("{flag} is required")))
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|argument| argument == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

fn reject_unknown(args: &[String], usage: &str) -> DraResult<()> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(DraError::InvalidArgument(format!(
            "unexpected argument '{}'; usage: {usage}",
            args[0]
        )))
    }
}

fn print_json<T: Serialize>(value: &T) -> DraResult<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
