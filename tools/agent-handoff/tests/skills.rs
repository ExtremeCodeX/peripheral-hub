//! Compatibility checks between the executable tool and the agent procedures.

use std::path::{Path, PathBuf};

const SKILLS: [&str; 6] = [
    "add-device",
    "reverse-engineer",
    "capture-protocol",
    "develop-driver",
    "validate-device",
    "review-contribution",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn skill(name: &str) -> String {
    let path = root().join(".agents/skills").join(name).join("SKILL.md");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn requested_skills_have_frontmatter_headings_and_real_commands() {
    for name in SKILLS {
        let text = skill(name);
        let mut parts = text.splitn(3, "---\n");
        assert_eq!(parts.next(), Some(""), "{name}: frontmatter start");
        let frontmatter = parts.next().expect("frontmatter");
        assert!(
            frontmatter.contains(&format!("name: {name}\n")),
            "{name}: frontmatter name"
        );
        assert!(frontmatter.contains("description: "), "{name}: description");
        assert!(
            frontmatter.contains("compatibility: opencode"),
            "{name}: compatibility"
        );
        for heading in [
            "## Inputs",
            "## Tools",
            "## Procedure",
            "## Artifacts",
            "## Failure handling",
            "## Stop conditions",
        ] {
            assert!(text.contains(heading), "{name}: missing {heading}");
        }
        assert!(
            text.contains("agent-contribution-workflow.md")
                || text.contains("agent-integration.md"),
            "{name}: missing shared handoff reference"
        );
        assert!(text.contains("dra-agent"), "{name}: missing dra-agent gate");
        assert!(
            text.contains("cargo run -p peripheral-hub-agent-handoff --bin dra-agent --"),
            "{name}: missing executable handoff command"
        );
        for line in text.lines() {
            let Some(index) = line.find("cargo run -p ") else {
                continue;
            };
            let package = line[index + "cargo run -p ".len()..]
                .split_whitespace()
                .next()
                .expect("package");
            assert!(
                matches!(
                    package,
                    "device-research-assistant"
                        | "peripheral-hub-agent-handoff"
                        | "peripheral-hub-daemon"
                ),
                "{name}: unknown package {package}"
            );
        }
        assert!(
            !text.contains("dra capture import --format pcap"),
            "{name}: invented pcap import"
        );
    }
}

#[test]
fn shared_agent_documents_and_schema_exist() {
    let root = root();
    for relative in [
        "docs/architecture/agent-integration.md",
        "docs/reverse-engineering/agent-contribution-workflow.md",
        "docs/architecture/decisions/0012-agent-dra-handoff.md",
        "schemas/research-handoff.schema.json",
    ] {
        assert!(root.join(relative).is_file(), "missing {relative}");
    }
    let schema: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("schemas/research-handoff.schema.json")).expect("schema"),
    )
    .expect("schema JSON");
    assert_eq!(schema["properties"]["handoff_schema_version"]["const"], 1);
    assert_eq!(schema["properties"]["source"]["$ref"], "#/$defs/source");
    assert_eq!(
        schema["$defs"]["safety"]["properties"]["loadable_manifest_generated"]["const"],
        false
    );
    let readme = std::fs::read_to_string(root.join(".agents/skills/README.md")).expect("readme");
    assert!(readme.contains("dra-agent"));
    assert!(readme.contains("agent-contribution-workflow.md"));
}
