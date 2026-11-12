//! Local process host. Development snapshots require native consent; no sandbox.
//! R1 belongs to its independently installed companion, never this core.
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const SOURCE: &str = include_str!("fake_plugin.py");
#[path = "local_runtime.rs"]
mod local_runtime;

// Registry metadata, not device/protocol logic. Additional local process
// languages can implement the same mode/evidence contract without host codecs.
fn allowed_plugin(id: &str, development: bool, debug: bool) -> Option<bool> {
    match id {
        "local.r1.hardware" | "local.r1.simulated" => None,
        "dev.fake" | "dev.starter" if debug && development => Some(false),
        id if debug && development && local_runtime::valid_dev_id(id) => Some(false),
        _ => None,
    }
}

#[cfg(test)]
#[test]
fn reduced_candidate_r1_is_rejected_before_loading_sources() {
    for id in ["local.r1.hardware", "local.r1.simulated"] {
        for development in [false, true] {
            for debug in [false, true] {
                assert_eq!(allowed_plugin(id, development, debug), None);
            }
        }
        assert_eq!(local_plugin(id, true).unwrap_err(), "UNAUTHORIZED_PLUGIN");
    }
}

fn local_plugin(id: &str, development: bool) -> Result<(String, bool), String> {
    // Resolve authorization before source construction or process startup. IPC
    // cannot enable debug support in release by sending development:true.
    let hardware =
        allowed_plugin(id, development, cfg!(debug_assertions)).ok_or("UNAUTHORIZED_PLUGIN")?;
    Ok((local_source(id)?, hardware))
}

// Reviewed local identities only. Never accept source, paths or arguments from IPC.
fn local_source(id: &str) -> Result<String, String> {
    match id {
        "dev.fake" => Ok(SOURCE.into()),
        #[cfg(debug_assertions)]
        "dev.starter" => local_runtime::starter_source(),
        #[cfg(debug_assertions)]
        id if local_runtime::valid_dev_id(id) => local_runtime::plugin_source(id),
        "local.r1.simulated" => local_runtime::source(false, SOURCE),
        "local.r1.hardware" => local_runtime::source(true, SOURCE),
        _ => Err("UNAUTHORIZED_PLUGIN".into()),
    }
}
const MAX_LINE: usize = 262144;
const MAX_BATCH: usize = 8;
const TIMEOUT: Duration = Duration::from_secs(3);

#[cfg(test)]
#[path = "plugin_host_tests.rs"]
mod tests;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum DevAction {
    Start {
        plugin_id: String,
        development: bool,
    },
    Capabilities {},
    Discover {},
    Open {},
    OpenCandidate {
        candidate_id: String,
        authorization: bool,
    },
    ReadActive {
        session: String,
        authorization: bool,
    },
    Apply {
        session: String,
        baseline_token: String,
        key: String,
        value: Value,
        authorization: bool,
    },
    Read {
        session: String,
    },
    Set {
        session: String,
        baseline_token: String,
        key: String,
        value: Value,
    },
    Close {
        session: String,
    },
    Stop {},
}

// Host-only challenge. No Deserialize/IPC command or reusable approval token.
// A confirmation is consumed synchronously while the caller exclusively owns
// DevHost, so neither the operation nor its process/session can change.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalConfirmation {
    pub generation: u64,
    pub operation: u64,
    pub plugin_id: String,
    pub method: String,
    pub params: Value,
}

impl PhysicalConfirmation {
    #[allow(dead_code)] // The non-GUI plugin-poc binary deliberately has no dialog.
    pub fn description(&self) -> String {
        let operation = match self.method.as_str() {
            "process.start.dev" => format!(
                "Executar código Python local de desenvolvimento, modificável e NÃO isolado. Ele terá as permissões do seu usuário, inclusive arquivos, rede e hardware. Não autorize código desconhecido.\nSHA-256 do snapshot: {}",
                self.params["sha256"].as_str().unwrap_or("")
            ),
            "device.open" => format!(
                "Abrir e identificar o candidato {}. Isso permite acesso HID ao dispositivo.",
                self.params["candidateId"].as_str().unwrap_or("")
            ),
            "device.get_state" => "Ler a configuração atual. A leitura ativa também envia comandos preparadores ao periférico.".into(),
            "device.set_setting" => format!(
                "Alterar o grupo {} para {}. Inclui leitura ativa, backup e escrita de configuração no periférico.",
                self.params["key"].as_str().unwrap_or(""), self.params["value"]
            ),
            _ => "Operação física não reconhecida; não autorize.".into(),
        };
        format!(
            "Integração: {}\nGeração: {} · Operação: {}\n\n{}\n\nAutorize somente se você solicitou esta operação. A autorização vale apenas para esta operação; não é permissão permanente.",
            self.plugin_id, self.generation, self.operation, operation
        )
    }
}

#[derive(Default)]
pub struct DevHost {
    process: Option<Process>,
    session: Option<String>,
    baseline: Option<String>,
    group_baselines: Value,
    capabilities: Value,
    hardware: bool,
    candidates: HashSet<String>,
    generation: u64,
    operation: u64,
}

impl DevHost {
    #[allow(dead_code)] // Non-GUI entry is used by harnesses, not every binary.
    pub fn execute(&mut self, action: DevAction) -> Result<Value, String> {
        // Non-GUI callers have no native confirmation provider. Never let CLI
        // input or a renderer boolean stand in for host-side confirmation.
        self.execute_with_confirmation(action, |_| Err("NATIVE_CONFIRMATION_REQUIRED".into()))
    }

    pub fn execute_with_confirmation(
        &mut self,
        action: DevAction,
        confirm: impl FnOnce(&PhysicalConfirmation) -> Result<(), String>,
    ) -> Result<Value, String> {
        // Legacy flags encode intent only. The host confirmation below is the
        // actual boundary for physical open/read/write, in debug AND release.
        let (action, physical_intent) = match action {
            DevAction::ReadActive {
                session,
                authorization,
            } => (DevAction::Read { session }, authorization),
            DevAction::Apply {
                session,
                baseline_token,
                key,
                value,
                authorization,
            } => (
                DevAction::Set {
                    session,
                    baseline_token,
                    key,
                    value,
                },
                authorization,
            ),
            action => (action, false),
        };
        match action {
            DevAction::Start {
                plugin_id,
                development,
            } => {
                let (source, hardware) = local_plugin(&plugin_id, development)?;
                if self.process.is_some() {
                    return Err("ALREADY_STARTED".into());
                }
                let generation = self.generation.checked_add(1).ok_or("HOST_FAILED")?;
                if plugin_id != "dev.fake" {
                    use sha2::{Digest, Sha256};
                    confirm(&PhysicalConfirmation {
                        generation,
                        operation: 0,
                        plugin_id: plugin_id.clone(),
                        method: "process.start.dev".into(),
                        params: json!({"sha256":format!("{:x}", Sha256::digest(source.as_bytes()))}),
                    })?;
                }
                let mut process = Process::spawn(&source, TIMEOUT)?;
                process.hardware = hardware;
                process.identity = plugin_id;
                let hello = process.handshake()?;
                let capabilities = process.call("plugin.get_capabilities", json!({}))?;
                if !valid_mode_result("plugin.get_capabilities", &capabilities, process.hardware) {
                    return Err("INVALID_CAPABILITIES".into());
                }
                self.capabilities = capabilities;
                self.hardware = process.hardware;
                self.candidates.clear();
                self.process = Some(process);
                self.generation = generation;
                Ok(hello)
            }
            DevAction::Stop {} => {
                self.session = None;
                self.baseline = None;
                self.group_baselines = Value::Null;
                self.candidates.clear();
                if let Some(mut process) = self.process.take() {
                    process.shutdown()?;
                }
                Ok(json!({"mode":if self.hardware {"hardware"} else {"fake"},"stopped":true}))
            }
            action => {
                let (method, params) = match action {
                    DevAction::Capabilities {} => ("plugin.get_capabilities", json!({})),
                    DevAction::Discover {} => {
                        if !self.hardware {
                            return Err("UNSUPPORTED_OPERATION".into());
                        }
                        if self.session.is_some() {
                            return Err("ALREADY_OPEN".into());
                        }
                        self.candidates.clear();
                        ("device.discover", json!({}))
                    }
                    DevAction::Open {} => {
                        if self.hardware {
                            return Err("AUTHORIZATION_REQUIRED".into());
                        }
                        if self.session.is_some() {
                            return Err("ALREADY_OPEN".into());
                        }
                        ("device.open", json!({"candidateId":"fake-device"}))
                    }
                    DevAction::OpenCandidate {
                        candidate_id,
                        authorization,
                    } => {
                        if !self.hardware || !authorization {
                            return Err("AUTHORIZATION_REQUIRED".into());
                        }
                        if self.session.is_some() {
                            return Err("ALREADY_OPEN".into());
                        }
                        if !self.candidates.contains(&candidate_id) {
                            return Err("UNKNOWN_CANDIDATE".into());
                        }
                        (
                            "device.open",
                            json!({"candidateId":candidate_id,"authorization":true}),
                        )
                    }
                    DevAction::Read { session } => {
                        if self.hardware && !physical_intent {
                            return Err("AUTHORIZATION_REQUIRED".into());
                        }
                        self.check_session(&session)?;
                        (
                            "device.get_state",
                            if self.hardware {
                                json!({"session":session,"authorization":true})
                            } else {
                                json!({"session":session})
                            },
                        )
                    }
                    DevAction::Set {
                        session,
                        baseline_token,
                        key,
                        value,
                    } => {
                        if self.hardware && !physical_intent {
                            return Err("AUTHORIZATION_REQUIRED".into());
                        }
                        self.check_session(&session)?;
                        let expected = self
                            .group_baselines
                            .get(&key)
                            .and_then(Value::as_str)
                            .or(self.baseline.as_deref());
                        if expected != Some(&baseline_token) {
                            return Err("CONFLICT".into());
                        }
                        if !allows(&self.capabilities, &key, &json!(value), "write") {
                            return Err("INVALID_SETTING".into());
                        }
                        let mut params = json!({"session":session,"baselineToken":baseline_token,"key":key,"value":value});
                        if self.hardware {
                            params["authorization"] = json!(true);
                        }
                        ("device.set_setting", params)
                    }
                    DevAction::Close { session } => {
                        self.check_session(&session)?;
                        ("device.close", json!({"session":session}))
                    }
                    _ => unreachable!(),
                };
                if self.process.is_none() {
                    return Err("NOT_STARTED".into());
                }
                if self.hardware
                    && matches!(
                        method,
                        "device.open" | "device.get_state" | "device.set_setting"
                    )
                {
                    self.operation = self.operation.checked_add(1).ok_or("HOST_FAILED")?;
                    confirm(&PhysicalConfirmation {
                        generation: self.generation,
                        operation: self.operation,
                        plugin_id: self.process.as_ref().unwrap().identity.clone(),
                        method: method.into(),
                        params: params.clone(),
                    })?;
                    // &mut self (and the IPC host mutex) is held throughout the
                    // native decision. No grant is cached, returned to IPC or
                    // usable by another operation/generation. All validation
                    // above therefore still applies to these exact params.
                    if method == "device.open" {
                        self.candidates.clear();
                    }
                }
                let result = self
                    .process
                    .as_mut()
                    .ok_or("NOT_STARTED")?
                    .call(method, params);
                // Any failure destroys this generation; never retry/replay or accept late replies.
                let value = match result {
                    Ok(value) => value,
                    Err(error) => {
                        self.process = None;
                        self.session = None;
                        self.baseline = None;
                        self.group_baselines = Value::Null;
                        return Err(error);
                    }
                };
                if !valid_mode_result(method, &value, self.hardware) {
                    self.process = None;
                    self.session = None;
                    self.baseline = None;
                    self.group_baselines = Value::Null;
                    return Err(operation_error(
                        self.hardware,
                        method,
                        "INVALID_RESULT_SCHEMA",
                    ));
                }
                if method == "plugin.get_capabilities" {
                    self.capabilities = value.clone();
                }
                if method == "device.discover" {
                    self.candidates = value["candidates"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|c| c["candidateId"].as_str().unwrap().to_owned())
                        .collect();
                }
                if method == "device.open" {
                    self.session = value["session"].as_str().map(str::to_owned);
                }
                if method == "device.get_state" || method == "device.set_setting" {
                    if value["session"].as_str() != self.session.as_deref() {
                        self.process = None;
                        self.session = None;
                        self.baseline = None;
                        self.group_baselines = Value::Null;
                        return Err(operation_error(
                            self.hardware,
                            method,
                            "INVALID_SESSION_RESPONSE",
                        ));
                    }
                    if !value["values"]
                        .as_object()
                        .unwrap()
                        .iter()
                        .all(|(key, v)| allows(&self.capabilities, key, v, "read"))
                        || self.capabilities["settings"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|s| {
                                s["access"].as_array().unwrap().contains(&json!("read"))
                                    && value["values"].get(s["key"].as_str().unwrap()).is_none()
                            })
                    {
                        self.process = None;
                        self.session = None;
                        self.baseline = None;
                        self.group_baselines = Value::Null;
                        return Err(operation_error(
                            self.hardware,
                            method,
                            "INVALID_STATE_CAPABILITY",
                        ));
                    }
                    self.baseline = value["baselineToken"].as_str().map(str::to_owned);
                    self.group_baselines =
                        value.get("baselineTokens").cloned().unwrap_or(Value::Null);
                }
                if method == "device.close" {
                    self.session = None;
                    self.baseline = None;
                    self.group_baselines = Value::Null;
                }
                Ok(value)
            }
        }
    }
    fn check_session(&self, session: &str) -> Result<(), String> {
        if self.session.as_deref() != Some(session) {
            Err("INVALID_SESSION".into())
        } else {
            Ok(())
        }
    }
}

impl Drop for DevHost {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.shutdown();
        }
    }
}

fn keys(value: &Value, expected: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.len() == expected.len() && expected.iter().all(|k| o.contains_key(*k)))
}
fn token(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|s| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}
// Deliberately bounded schema subset; no refs, patterns, executable validators,
// additional properties or implicit coercion. Object properties are all required.
fn valid_schema(s: &Value, depth: usize) -> bool {
    if depth > 8 {
        return false;
    }
    match s["type"].as_str() {
        Some("boolean") => keys(s, &["type"]),
        Some("integer" | "number" | "string") if s.get("enum").is_some() => {
            keys(s, &["type", "enum"])
                && s["enum"].as_array().is_some_and(|a| {
                    !a.is_empty()
                        && a.len() <= 512
                        && a.iter().all(|v| match s["type"].as_str() {
                            Some("integer") => v.as_i64().is_some() || v.as_u64().is_some(),
                            Some("number") => v.as_f64().is_some_and(f64::is_finite),
                            _ => v.as_str().is_some_and(|t| !t.is_empty() && t.len() <= 64),
                        })
                })
        }
        Some("integer" | "number") => {
            keys(s, &["type", "minimum", "maximum", "multipleOf"])
                && ["minimum", "maximum", "multipleOf"].iter().all(|k| {
                    s[k].as_f64()
                        .is_some_and(|n| n.is_finite() && n.abs() <= 1e9)
                })
                && s["minimum"].as_f64() <= s["maximum"].as_f64()
                && s["multipleOf"].as_f64().unwrap() > 0.0
        }
        Some("array") => {
            keys(s, &["type", "items", "minItems", "maxItems"])
                && s["minItems"].as_u64().is_some_and(|n| n <= 32)
                && s["maxItems"]
                    .as_u64()
                    .is_some_and(|n| n <= 32 && n >= s["minItems"].as_u64().unwrap())
                && valid_schema(&s["items"], depth + 1)
        }
        Some("object") => {
            keys(s, &["type", "properties"])
                && s["properties"].as_object().is_some_and(|p| {
                    !p.is_empty()
                        && p.len() <= 32
                        && p.iter().all(|(k, v)| {
                            !k.is_empty() && k.len() <= 64 && valid_schema(v, depth + 1)
                        })
                })
        }
        _ => false,
    }
}
fn matches_schema(s: &Value, v: &Value, depth: usize) -> bool {
    if depth > 8 {
        return false;
    }
    if let Some(a) = s["enum"].as_array() {
        return (s["type"] != "integer" || v.as_i64().is_some() || v.as_u64().is_some())
            && a.contains(v);
    }
    match s["type"].as_str() {
        Some("boolean") => v.is_boolean(),
        Some("integer" | "number") => {
            (s["type"] != "integer" || v.as_i64().is_some() || v.as_u64().is_some())
                && v.as_f64().is_some_and(|n| {
                    let step = s["multipleOf"].as_f64().unwrap();
                    n.is_finite()
                        && n >= s["minimum"].as_f64().unwrap()
                        && n <= s["maximum"].as_f64().unwrap()
                        && (n / step - (n / step).round()).abs() < 1e-9
                })
        }
        Some("array") => v.as_array().is_some_and(|a| {
            a.len() >= s["minItems"].as_u64().unwrap() as usize
                && a.len() <= s["maxItems"].as_u64().unwrap() as usize
                && a.iter().all(|v| matches_schema(&s["items"], v, depth + 1))
        }),
        Some("object") => v.as_object().is_some_and(|o| {
            let p = s["properties"].as_object().unwrap();
            o.len() == p.len()
                && p.iter()
                    .all(|(k, s)| o.get(k).is_some_and(|v| matches_schema(s, v, depth + 1)))
        }),
        _ => false,
    }
}
fn allows(capabilities: &Value, key: &str, value: &Value, access: &str) -> bool {
    capabilities["settings"].as_array().is_some_and(|settings| {
        settings.iter().any(|setting| {
            setting["key"] == key
                && setting["access"]
                    .as_array()
                    .is_some_and(|a| a.contains(&json!(access)))
                && matches_schema(
                    if access == "read" {
                        setting.get("readSchema").unwrap_or(&setting["schema"])
                    } else {
                        &setting["schema"]
                    },
                    value,
                    0,
                )
        })
    })
}
fn valid_result(method: &str, value: &Value) -> bool {
    if value["mode"] != "fake" {
        return false;
    }
    match method {
        "plugin.get_capabilities" => {
            let mut seen = HashSet::new();
            keys(value, &["mode", "hardwareAccess", "settings"])
                && value["hardwareAccess"] == false
                && value["settings"].as_array().is_some_and(|settings| {
                    !settings.is_empty()
                        && settings.len() <= 32
                        && settings.iter().all(|setting| {
                            (keys(setting, &["key", "access", "schema", "effects"])
                                || keys(
                                    setting,
                                    &["key", "access", "schema", "readSchema", "effects"],
                                ))
                                && setting["key"].as_str().is_some_and(|k| {
                                    !k.is_empty() && k.len() <= 64 && seen.insert(k)
                                })
                                && setting["effects"] == "simulation_only"
                                && setting["access"].as_array().is_some_and(|a| {
                                    !a.is_empty()
                                        && a.len() <= 2
                                        && a.iter().all(|v| v == "read" || v == "write")
                                })
                                && valid_schema(&setting["schema"], 0)
                                && setting.get("readSchema").is_none_or(|s| valid_schema(s, 0))
                        })
                })
        }
        "device.open" => {
            keys(
                value,
                &["mode", "hardwareAccess", "session", "identification"],
            ) && value["hardwareAccess"] == false
                && value["identification"] == "simulated"
                && token(&value["session"])
        }
        "device.get_state" | "device.set_setting" => {
            (keys(
                value,
                &[
                    "mode",
                    "hardwareAccess",
                    "session",
                    "values",
                    "baselineToken",
                    "outcome",
                ],
            ) || keys(
                value,
                &[
                    "mode",
                    "hardwareAccess",
                    "session",
                    "values",
                    "baselineToken",
                    "outcome",
                    "baselineTokens",
                    "readTimes",
                ],
            )) && value["hardwareAccess"] == false
                && value["outcome"] == "simulated"
                && token(&value["session"])
                && token(&value["baselineToken"])
                && value["values"]
                    .as_object()
                    .is_some_and(|v| !v.is_empty() && v.len() <= 32)
                && value.get("baselineTokens").is_none_or(|tokens| {
                    let values = value["values"].as_object().unwrap();
                    tokens.as_object().is_some_and(|t| {
                        t.len() == values.len()
                            && values.keys().all(|k| t.get(k).is_some_and(token))
                    }) && value["readTimes"].as_object().is_some_and(|t| {
                        t.len() == values.len()
                            && values.keys().all(|k| {
                                t.get(k)
                                    .and_then(Value::as_f64)
                                    .is_some_and(|n| n.is_finite() && n >= 0.0)
                            })
                    })
                })
        }
        "device.close" => value == &json!({"mode":"fake","closed":true}),
        _ => false,
    }
}

// Mode is pinned by the reviewed registry, not accepted from peer assertions.
// Reuse the bounded generic schema checks after validating hardware evidence.
fn valid_mode_result(method: &str, value: &Value, hardware: bool) -> bool {
    if !hardware {
        return valid_result(method, value);
    }
    if value["mode"] != "hardware" {
        return false;
    }
    if method == "device.close" {
        return value == &json!({"mode":"hardware","closed":true});
    }
    if value["hardwareAccess"] != true {
        return false;
    }
    if method == "device.discover" {
        let mut seen = HashSet::new();
        return keys(value, &["mode", "hardwareAccess", "candidates"])
            && value["candidates"].as_array().is_some_and(|items| {
                items.len() <= 32
                    && items.iter().all(|c| {
                        keys(
                            c,
                            &[
                                "candidateId",
                                "transport",
                                "vid",
                                "pid",
                                "revision",
                                "interface",
                                "hidraw",
                                "identity",
                            ],
                        ) && token(&c["candidateId"])
                            && seen.insert(c["candidateId"].clone())
                            && c["identity"] == "unconfirmed"
                            && ["transport", "vid", "pid", "revision", "interface", "hidraw"]
                                .iter()
                                .all(|k| {
                                    c[k].as_str().is_some_and(|s| {
                                        s.len() <= 64
                                            && s.bytes()
                                                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                                    })
                                })
                    })
            });
    }
    let mut normalized = value.clone();
    normalized["mode"] = json!("fake");
    normalized["hardwareAccess"] = json!(false);
    match method {
        "plugin.get_capabilities" => {
            if !value["settings"].as_array().is_some_and(|settings| {
                settings.iter().all(|s| {
                    s["effects"] == "active_read_and_configuration_write"
                        || (s["effects"] == "active_read" && s["access"] == json!(["read"]))
                })
            }) {
                return false;
            }
            for setting in normalized["settings"].as_array_mut().unwrap() {
                setting["effects"] = json!("simulation_only");
            }
        }
        "device.open" => {
            if ![
                json!("input_signature_confirmed"),
                json!("usb_transport_matched"),
            ]
            .contains(&value["identification"])
            {
                return false;
            }
            normalized["identification"] = json!("simulated");
        }
        "device.get_state" | "device.set_setting" => {
            let allowed = if method == "device.get_state" {
                vec![json!("read_from_device")]
            } else {
                vec![json!("unchanged"), json!("confirmed_by_readback")]
            };
            if !allowed.contains(&value["outcome"]) || value.get("baselineTokens").is_none() {
                return false;
            }
            normalized["outcome"] = json!("simulated");
        }
        _ => return false,
    }
    valid_result(method, &normalized)
}

pub fn error_detail(error: String) -> Value {
    // This string can contain only our validated, allowlisted remote envelope.
    if let Ok(value) = serde_json::from_str::<Value>(&error) {
        if valid_error_data(&value) {
            return value;
        }
    }
    let kind = if error.len() <= 64 && error.bytes().all(|b| b.is_ascii_uppercase() || b == b'_') {
        error
    } else {
        "HOST_FAILED".into()
    };
    let outcome = if kind == "TIMEOUT" || kind == "HOST_TASK_FAILED" {
        "unknown"
    } else {
        "not_sent"
    };
    json!({"kind":kind,"configurationOutcome":outcome,"stage":"host","operationId":null,"retryable":false})
}

fn operation_error(hardware: bool, method: &str, kind: &str) -> String {
    if !hardware {
        return kind.into();
    }
    let kind = if ["TIMEOUT", "PLUGIN_EXITED"].contains(&kind) {
        kind
    } else {
        "PROTOCOL_VIOLATION"
    };
    json!({"kind":kind,"configurationOutcome":if method == "device.set_setting" {"unknown"} else {"not_sent"},
        "stage":"host","operationId":null,"retryable":false}).to_string()
}

fn valid_error_data(data: &Value) -> bool {
    keys(
        data,
        &[
            "kind",
            "configurationOutcome",
            "stage",
            "operationId",
            "retryable",
        ],
    ) && data["kind"].as_str().is_some_and(|s| {
        [
            "AUTHORIZATION_REQUIRED",
            "INVALID_SESSION",
            "UNKNOWN_CANDIDATE",
            "ALREADY_OPEN",
            "CONFLICT",
            "INVALID_SETTING",
            "UNSUPPORTED_OPERATION",
            "PERMISSION_DENIED",
            "DEVICE_BUSY",
            "DEVICE_DISCONNECTED",
            "IDENTITY_UNCONFIRMED",
            "READ_PREPARATION_FAILED",
            "DEVICE_IO_FAILED",
            "INVALID_SNAPSHOT",
            "BACKUP_FAILED",
            "VERIFICATION_FAILED",
            "DISCOVERY_FAILED",
            "INTERNAL_ERROR",
            "TIMEOUT",
            "PLUGIN_EXITED",
            "PROTOCOL_VIOLATION",
        ]
        .contains(&s)
    }) && [json!("not_sent"), json!("unconfirmed"), json!("unknown")]
        .contains(&data["configurationOutcome"])
        && data["stage"].as_str().is_some_and(|s| {
            [
                "idle",
                "validating",
                "identifying",
                "preparing_read",
                "reading",
                "writing",
                "backing_up",
                "host",
            ]
            .contains(&s)
        })
        && (data["operationId"].is_null() || token(&data["operationId"]))
        && data["retryable"] == false
}

struct Process {
    child: Child,
    reaped: bool,
    writer: Option<mpsc::SyncSender<Vec<u8>>>,
    receiver: mpsc::Receiver<Result<Vec<u8>, String>>,
    overflow: Arc<AtomicBool>,
    stderr_bytes: Arc<AtomicUsize>,
    threads: Vec<JoinHandle<()>>,
    next_id: u64,
    timeout: Duration,
    identity: String,
    hardware: bool,
}

impl Process {
    fn handshake(&mut self) -> Result<Value, String> {
        let hello = self.call("plugin.hello", json!({"apiVersions":["0.1"]}))?;
        if hello
            != json!({"apiVersion":"0.1","pluginId":self.identity,"mode":if self.hardware {"hardware"} else {"fake"},"hardwareAccess":self.hardware})
        {
            return Err("INVALID_HANDSHAKE".into());
        }
        Ok(hello)
    }
    // Sources are host-owned snapshots (embedded fixture or pinned local R1).
    // Interpreter/args are private and fixed. No manifest loader or frontend paths.
    fn spawn(source: &str, timeout: Duration) -> Result<Self, String> {
        let mut command = Command::new("/usr/bin/python3");
        #[cfg(unix)]
        command.process_group(0);
        command.env_clear();
        // Preserve only user-local paths required for durable data and the
        // original cross-application advisory lock. Never inherit Python env.
        for key in ["HOME", "XDG_RUNTIME_DIR"] {
            if let Ok(value) = std::env::var(key) {
                if std::path::Path::new(&value).is_absolute() {
                    command.env(key, value);
                }
            }
        }
        let mut child = command
            .args(["-I", "-S", "-B", "-u", "-c", source])
            .current_dir("/")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "SPAWN_FAILED")?;
        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let (writer, requests) = mpsc::sync_channel::<Vec<u8>>(1);
        let writing = thread::spawn(move || {
            while let Ok(frame) = requests.recv() {
                if stdin.write_all(&frame).and_then(|_| stdin.flush()).is_err() {
                    break;
                }
            }
        });
        let (sender, receiver) = mpsc::sync_channel(8);
        let overflow = Arc::new(AtomicBool::new(false));
        let flag = overflow.clone();
        let reading = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut frame = Vec::new();
                let result = (&mut reader)
                    .take((MAX_LINE + 2) as u64)
                    .read_until(b'\n', &mut frame);
                let item = match result {
                    Ok(0) => Err("PLUGIN_EXITED".into()),
                    Ok(_) if frame.last() == Some(&b'\n') && frame.len() <= MAX_LINE + 1 => {
                        frame.pop();
                        Ok(frame)
                    }
                    _ => Err("INVALID_STDOUT_FRAME".into()),
                };
                let failed = item.is_err();
                if sender.try_send(item).is_err() {
                    flag.store(true, Ordering::SeqCst);
                    break;
                }
                if failed {
                    break;
                }
            }
        });
        let stderr_bytes = Arc::new(AtomicUsize::new(0));
        let count = stderr_bytes.clone();
        let logging = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            // Drain without retaining/logging potentially sensitive untrusted content.
            while let Ok(n) = stderr.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                // Only this thread writes the counter; avoid wraparound.
                count.store(
                    count.load(Ordering::Relaxed).saturating_add(n),
                    Ordering::Relaxed,
                );
            }
        });
        Ok(Self {
            child,
            reaped: false,
            writer: Some(writer),
            receiver,
            overflow,
            stderr_bytes,
            threads: vec![writing, reading, logging],
            next_id: 1,
            timeout,
            identity: "dev.fake".into(),
            hardware: false,
        })
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let original_timeout = self.timeout;
        if self.hardware {
            self.timeout = Duration::from_secs(match method {
                "device.open" => 8,
                "device.get_state" => 20,
                "device.set_setting" => 15,
                _ => 3,
            });
        }
        let response =
            self.exchange(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        self.timeout = original_timeout;
        let response = response.map_err(|error| operation_error(self.hardware, method, &error))?;
        if response.get("error").is_some() {
            if self.hardware {
                let data = &response["error"]["data"];
                if !valid_error_data(data) {
                    return Err(operation_error(true, method, "INVALID_ERROR"));
                }
                return Err(data.to_string());
            }
            return Err("PLUGIN_RPC_ERROR".into());
        }
        Ok(response["result"].clone())
    }
    fn exchange(&mut self, request: Value) -> Result<Value, String> {
        let deadline = Instant::now() + self.timeout;
        let requests: Vec<&Value> = if let Some(batch) = request.as_array() {
            if batch.is_empty() || batch.len() > MAX_BATCH {
                return Err("BATCH_LIMIT".into());
            }
            batch.iter().collect()
        } else {
            vec![&request]
        };
        let expected: HashSet<u64> = requests
            .iter()
            .filter_map(|r| r.get("id").and_then(Value::as_u64))
            .collect();
        if expected.len() != requests.iter().filter(|r| r.get("id").is_some()).count() {
            return Err("INVALID_REQUEST_IDS".into());
        }
        if let Some(id) = expected.iter().max() {
            self.next_id = self.next_id.max(id.saturating_add(1));
        }
        let mut frame = serde_json::to_vec(&request).map_err(|_| "INVALID_REQUEST")?;
        if frame.len() > MAX_LINE {
            return Err("MESSAGE_LIMIT".into());
        }
        frame.push(b'\n');
        loop {
            match self.writer.as_ref().ok_or("STOPPED")?.try_send(frame) {
                Ok(()) => break,
                Err(mpsc::TrySendError::Full(returned)) => {
                    if Instant::now() >= deadline {
                        return Err("TIMEOUT".into());
                    }
                    frame = returned;
                    thread::sleep(Duration::from_millis(1));
                }
                Err(_) => return Err("PLUGIN_EXITED".into()),
            }
        }
        if expected.is_empty() {
            return Ok(Value::Null);
        }
        for _ in 0..=MAX_BATCH {
            if self.overflow.load(Ordering::SeqCst) {
                return Err("EXCESS_MESSAGES".into());
            }
            let raw = self
                .receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|_| "TIMEOUT")??;
            let response = strict_json(&raw)?;
            // Well-formed peer notifications are ignored, bounded by the same deadline/count.
            if (keys(&response, &["jsonrpc", "method", "params"])
                || keys(&response, &["jsonrpc", "method"]))
                && response["jsonrpc"] == "2.0"
                && response["method"].is_string()
                && (response.get("params").is_none()
                    || response["params"].is_object()
                    || response["params"].is_array())
            {
                continue;
            }
            let responses: Vec<&Value> = if let Some(batch) = response.as_array() {
                if !request.is_array() || batch.is_empty() || batch.len() > MAX_BATCH {
                    return Err("INVALID_RESPONSE_BATCH".into());
                }
                batch.iter().collect()
            } else {
                if request.is_array() {
                    return Err("EXPECTED_BATCH_RESPONSE".into());
                }
                vec![&response]
            };
            let mut seen = HashSet::new();
            for item in responses {
                let id = item["id"].as_u64().ok_or("INVALID_RESPONSE_ID")?;
                if !expected.contains(&id) || !seen.insert(id) {
                    return Err("WRONG_OR_DUPLICATE_ID".into());
                }
                if item["jsonrpc"] != "2.0" {
                    return Err("INVALID_RPC_VERSION".into());
                }
                if item.get("result").is_some() {
                    if !keys(item, &["jsonrpc", "id", "result"]) {
                        return Err("INVALID_RESPONSE".into());
                    }
                } else {
                    let e = &item["error"];
                    if !keys(item, &["jsonrpc", "id", "error"])
                        || !(keys(e, &["code", "message"]) || keys(e, &["code", "message", "data"]))
                        || e["code"].as_i64().is_none()
                        || !e["message"].is_string()
                    {
                        return Err("INVALID_ERROR".into());
                    }
                }
            }
            if seen != expected {
                return Err("MISSING_RESPONSE".into());
            }
            if self.overflow.load(Ordering::SeqCst) {
                return Err("EXCESS_MESSAGES".into());
            }
            return Ok(response);
        }
        Err("EXCESS_NOTIFICATIONS".into())
    }
    fn shutdown(&mut self) -> Result<(), String> {
        let result = self.call("plugin.shutdown", json!({}))?;
        if result != json!({"mode":if self.hardware {"hardware"} else {"fake"},"stopped":true}) {
            return Err("INVALID_SHUTDOWN".into());
        }
        self.writer = None;
        let deadline = Instant::now() + self.timeout;
        // Drain before reaping. A zombie still reserves its PID/group identity;
        // this prevents cleanup signaling an unrelated, reused PID after wait.
        loop {
            match self
                .receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(Err(error)) if error == "PLUGIN_EXITED" => break,
                Ok(Ok(_)) => return Err("EXCESS_MESSAGES".into()),
                Ok(Err(error)) => return Err(error),
                Err(_) => return Err("SHUTDOWN_TIMEOUT".into()),
            }
        }
        if self.overflow.load(Ordering::SeqCst) {
            return Err("EXCESS_MESSAGES".into());
        }
        // Include stderr/writer completion in the same deadline before reaping.
        while self.threads.iter().any(|handle| !handle.is_finished()) {
            if Instant::now() >= deadline {
                return Err("SHUTDOWN_TIMEOUT".into());
            }
            thread::sleep(Duration::from_millis(1));
        }
        loop {
            if let Some(status) = self.child.try_wait().map_err(|_| "WAIT_FAILED")? {
                self.reaped = true;
                if !status.success() {
                    return Err("PLUGIN_EXITED".into());
                }
                break;
            }
            if Instant::now() >= deadline {
                return Err("SHUTDOWN_TIMEOUT".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.writer = None;
        #[cfg(target_os = "linux")]
        {
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            // SAFETY: Linux kill accepts any signed pid. The child is spawned in
            // its own group; negative pid targets only that group, never ours.
            // This also closes inherited pipes of descendants remaining in it.
            if !self.reaped {
                unsafe {
                    kill(-(self.child.id() as i32), 9);
                }
            }
        }
        if !self.reaped {
            let _ = self.child.kill();
        }
        let _ = self.child.wait(); // reap even after timeout/crash
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
        let _ = self.stderr_bytes.load(Ordering::Relaxed);
    }
}

// Reject duplicate JSON keys before converting to Value; serde_json::Value alone
// would silently keep the last value. Recursion and allocation are bounded.
fn strict_json(raw: &[u8]) -> Result<Value, String> {
    let mut depth = 0usize;
    let (mut quoted, mut escaped) = (false, false);
    for &b in raw {
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
        } else if b == b'"' {
            quoted = true;
        } else if b == b'{' || b == b'[' {
            depth += 1;
            if depth > 32 {
                return Err("DEPTH_LIMIT".into());
            }
        } else if b == b'}' || b == b']' {
            depth = depth.saturating_sub(1);
        }
    }
    #[derive(Debug)]
    struct Unique(Value);
    impl<'de> Deserialize<'de> for Unique {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Unique;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("unique-key JSON")
                }
                fn visit_map<M: serde::de::MapAccess<'de>>(
                    self,
                    mut m: M,
                ) -> Result<Unique, M::Error> {
                    let mut out = serde_json::Map::new();
                    while let Some((k, v)) = m.next_entry::<String, Unique>()? {
                        if out.insert(k, v.0).is_some() {
                            return Err(serde::de::Error::custom("duplicate key"));
                        }
                    }
                    Ok(Unique(Value::Object(out)))
                }
                fn visit_seq<S: serde::de::SeqAccess<'de>>(
                    self,
                    mut s: S,
                ) -> Result<Unique, S::Error> {
                    let mut out = Vec::new();
                    while let Some(v) = s.next_element::<Unique>()? {
                        out.push(v.0);
                    }
                    Ok(Unique(Value::Array(out)))
                }
                fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Unique, E> {
                    serde_json::Number::from_f64(v)
                        .map(|n| Unique(Value::Number(n)))
                        .ok_or_else(|| E::custom("nonfinite"))
                }
                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Unique, E> {
                    Ok(Unique(v.into()))
                }
                fn visit_unit<E: serde::de::Error>(self) -> Result<Unique, E> {
                    Ok(Unique(Value::Null))
                }
            }
            d.deserialize_any(Visitor)
        }
    }
    if raw.len() > MAX_LINE {
        return Err("MESSAGE_LIMIT".into());
    }
    serde_json::from_slice::<Unique>(raw)
        .map(|v| v.0)
        .map_err(|_| "INVALID_STDOUT_JSON".into())
}
