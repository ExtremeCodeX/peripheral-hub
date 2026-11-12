//! Development supervisor. Only validated identities in host-owned XDG locations.
use crate::plugin_host::{DevAction, DevHost};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum StarterAction {
    Start {},
    StartPlugin {
        plugin_id: String,
    },
    Capabilities {},
    Open {},
    Read {
        session: String,
    },
    Set {
        session: String,
        baseline_token: String,
        value: Value,
    },
    SetSetting {
        session: String,
        baseline_token: String,
        key: String,
        value: Value,
    },
    Stop {},
}

#[derive(Default)]
pub struct StarterHost(DevHost);

impl StarterHost {
    pub fn execute(&mut self, request: StarterAction) -> Result<Value, String> {
        self.execute_with_confirmation(request, |_| Err("NATIVE_CONFIRMATION_REQUIRED".into()))
    }

    pub fn execute_with_confirmation(
        &mut self,
        request: StarterAction,
        confirm: impl FnOnce(&crate::plugin_host::PhysicalConfirmation) -> Result<(), String>,
    ) -> Result<Value, String> {
        // Independent guard also covers direct CLI/test calls in release.
        if !cfg!(debug_assertions) {
            return Err("DEV_ONLY".into());
        }
        let action = match request {
            StarterAction::Start {} => DevAction::Start {
                plugin_id: "dev.starter".into(),
                development: true,
            },
            StarterAction::StartPlugin { plugin_id } => DevAction::Start {
                plugin_id,
                development: true,
            },
            StarterAction::Capabilities {} => DevAction::Capabilities {},
            StarterAction::Open {} => DevAction::Open {},
            StarterAction::Read { session } => DevAction::Read { session },
            StarterAction::Set {
                session,
                baseline_token,
                value,
            } => DevAction::Set {
                session,
                baseline_token,
                key: "sample_value".into(),
                value,
            },
            StarterAction::Stop {} => DevAction::Stop {},
            StarterAction::SetSetting {
                session,
                baseline_token,
                key,
                value,
            } => DevAction::Set {
                session,
                baseline_token,
                key,
                value,
            },
        };
        self.0.execute_with_confirmation(action, confirm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    #[cfg(debug_assertions)]
    fn named_plugin_capabilities_controls_and_restart_without_host_rebuild() {
        let mut host = StarterHost::default();
        let start = || StarterAction::StartPlugin {
            plugin_id: "dev.test-mouse".into(),
        };
        assert_eq!(
            host.execute(start()).unwrap_err(),
            "NATIVE_CONFIRMATION_REQUIRED"
        );
        for _ in 0..2 {
            let hello = host
                .execute_with_confirmation(start(), |op| {
                    assert_eq!(op.plugin_id, "dev.test-mouse");
                    assert_eq!(op.params["sha256"].as_str().unwrap().len(), 64);
                    Ok(())
                })
                .unwrap();
            assert_eq!(hello["pluginId"], "dev.test-mouse");
            let caps = host.execute(StarterAction::Capabilities {}).unwrap();
            assert_eq!(caps["settings"][0]["key"], "gain");
            let session = host.execute(StarterAction::Open {}).unwrap()["session"]
                .as_str()
                .unwrap()
                .to_owned();
            let state = host
                .execute(StarterAction::Read {
                    session: session.clone(),
                })
                .unwrap();
            assert_eq!(state["values"]["gain"], 50);
            let changed = host
                .execute(StarterAction::SetSetting {
                    session,
                    baseline_token: state["baselineToken"].as_str().unwrap().into(),
                    key: "gain".into(),
                    value: json!(75),
                })
                .unwrap();
            assert_eq!(changed["values"]["gain"], 75);
            host.execute(StarterAction::Stop {}).unwrap();
        }
        assert!(host
            .execute_with_confirmation(
                StarterAction::StartPlugin {
                    plugin_id: "dev.foo/../../bin/sh".into()
                },
                |_| panic!("invalid path must fail before consent")
            )
            .is_err());
    }
    #[test]
    fn starter_closed_actions_reject_arbitrary_execution_and_hardware() {
        for request in [
            json!({"action":"start","plugin_id":"local.r1.hardware"}),
            json!({"action":"start","path":"/bin/sh"}),
            json!({"action":"start","source":"x"}),
            json!({"action":"discover"}),
            json!({"action":"apply","authorization":true}),
            json!({"action":"set","session":"x","baseline_token":"x","value":50,"key":"dpi"}),
        ] {
            assert!(serde_json::from_value::<StarterAction>(request).is_err());
        }
    }
    #[test]
    fn starter_release_denies_even_direct_calls() {
        if !cfg!(debug_assertions) {
            for action in [StarterAction::Start {}, StarterAction::Stop {}] {
                assert_eq!(
                    StarterHost::default().execute(action).unwrap_err(),
                    "DEV_ONLY"
                );
            }
        }
    }
    #[test]
    #[cfg(debug_assertions)]
    fn starter_real_process_roundtrip_conflict_restart_and_isolation() {
        let mut starter = StarterHost::default();
        let mut unrelated = DevHost::default();
        unrelated
            .execute(DevAction::Start {
                plugin_id: "dev.fake".into(),
                development: true,
            })
            .unwrap();
        assert_eq!(
            starter
                .execute_with_confirmation(StarterAction::Start {}, |_| Ok(()))
                .unwrap()["pluginId"],
            "dev.starter"
        );
        assert_eq!(
            starter.execute(StarterAction::Start {}).unwrap_err(),
            "ALREADY_STARTED"
        );
        assert_eq!(
            starter.execute(StarterAction::Capabilities {}).unwrap()["settings"][0]["effects"],
            "simulation_only"
        );
        let session = starter.execute(StarterAction::Open {}).unwrap()["session"]
            .as_str()
            .unwrap()
            .to_owned();
        let before = starter
            .execute(StarterAction::Read {
                session: session.clone(),
            })
            .unwrap();
        let baseline = before["baselineToken"].as_str().unwrap().to_owned();
        assert_eq!(before["values"]["sample_value"], 50);
        assert_eq!(
            starter
                .execute(StarterAction::Set {
                    session: session.clone(),
                    baseline_token: baseline.clone(),
                    value: json!(true)
                })
                .unwrap_err(),
            "INVALID_SETTING"
        );
        let changed = starter
            .execute(StarterAction::Set {
                session: session.clone(),
                baseline_token: baseline.clone(),
                value: json!(75),
            })
            .unwrap();
        assert_eq!(changed["values"]["sample_value"], 75);
        assert_eq!(changed["outcome"], "simulated");
        assert_eq!(changed["hardwareAccess"], false);
        assert_eq!(
            starter
                .execute(StarterAction::Set {
                    session: session.clone(),
                    baseline_token: baseline,
                    value: json!(100)
                })
                .unwrap_err(),
            "CONFLICT"
        );
        // Stop a different host; it must not stop the starter process.
        unrelated.execute(DevAction::Stop {}).unwrap();
        assert_eq!(
            starter
                .execute(StarterAction::Read {
                    session: session.clone()
                })
                .unwrap()["values"]["sample_value"],
            75
        );
        unrelated
            .execute(DevAction::Start {
                plugin_id: "dev.fake".into(),
                development: true,
            })
            .unwrap();
        starter.execute(StarterAction::Stop {}).unwrap();
        assert_eq!(
            unrelated.execute(DevAction::Capabilities {}).unwrap()["hardwareAccess"],
            false
        );
        unrelated.execute(DevAction::Stop {}).unwrap();
        assert!(starter
            .execute(StarterAction::Read {
                session: session.clone()
            })
            .is_err());
        starter
            .execute_with_confirmation(StarterAction::Start {}, |_| Ok(()))
            .unwrap();
        let next = starter.execute(StarterAction::Open {}).unwrap()["session"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_ne!(session, next);
        assert_eq!(
            starter
                .execute(StarterAction::Read { session: next })
                .unwrap()["values"]["sample_value"],
            50
        );
        starter.execute(StarterAction::Stop {}).unwrap();
    }

    #[test]
    #[cfg(debug_assertions)]
    fn starter_requires_host_consent_for_each_snapshot_start() {
        let mut host = StarterHost::default();
        assert_eq!(
            host.execute(StarterAction::Start {}).unwrap_err(),
            "NATIVE_CONFIRMATION_REQUIRED"
        );
        assert_eq!(
            host.execute_with_confirmation(StarterAction::Start {}, |op| {
                assert_eq!(op.method, "process.start.dev");
                assert_eq!(op.params["sha256"].as_str().unwrap().len(), 64);
                Err("NATIVE_CONFIRMATION_DENIED".into())
            })
            .unwrap_err(),
            "NATIVE_CONFIRMATION_DENIED"
        );
        assert_eq!(
            host.execute(StarterAction::Capabilities {}).unwrap_err(),
            "NOT_STARTED"
        );
    }
}
