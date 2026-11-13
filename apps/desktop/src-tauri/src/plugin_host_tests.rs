use super::*;

#[test]
fn generic_ack_error_preserves_uncertainty_without_retry() {
    let data = json!({"kind":"ACK_FAILED","configurationOutcome":"unknown","stage":"writing","operationId":null,"retryable":false});
    assert!(valid_error_data(&data));
    assert_eq!(error_detail(data.to_string()), data);
    let mut unsafe_retry = data.clone();
    unsafe_retry["retryable"] = json!(true);
    assert!(!valid_error_data(&unsafe_retry));
}

// Entirely synthetic peer, including for release tests. Never construct the R1
// source, load its modules, enumerate sysfs, or open a device in these tests.
fn physical_confirmation_host(open: bool) -> DevHost {
    let source = r#"
import json,sys
session='a'*32
baseline='b'*32
for line in sys.stdin:
    request=json.loads(line)
    method=request['method']
    if method=='device.open':
        result=dict(mode='hardware',hardwareAccess=True,session=session,identification='input_signature_confirmed')
    elif method in ('device.get_state','device.set_setting'):
        result=dict(mode='hardware',hardwareAccess=True,session=session,values={'gain':request['params'].get('value',1)},baselineToken=baseline,baselineTokens={'gain':baseline},readTimes={'gain':1},outcome='read_from_device' if method=='device.get_state' else 'confirmed_by_readback')
    else:
        raise RuntimeError('unexpected method in synthetic confirmation test')
    print(json.dumps(dict(jsonrpc='2.0',id=request['id'],result=result)),flush=True)
"#;
    let mut process = peer(source);
    process.hardware = true;
    process.identity = "synthetic.confirmation.fixture".into();
    let mut host = DevHost::default();
    host.process = Some(process);
    host.hardware = true;
    host.generation = 41;
    host.capabilities = json!({"mode":"hardware","hardwareAccess":true,"settings":[{"key":"gain","schema":{"type":"integer","enum":[1,2]},"access":["read","write"],"effects":"active_read_and_configuration_write"}]});
    host.candidates.insert("c".repeat(32));
    if open {
        host.session = Some("a".repeat(32));
        host.baseline = Some("b".repeat(32));
        host.group_baselines = json!({"gain":"b".repeat(32)});
    }
    host
}

fn physical_confirmation_actions() -> [DevAction; 3] {
    [
        DevAction::OpenCandidate {
            candidate_id: "c".repeat(32),
            authorization: true,
        },
        DevAction::ReadActive {
            session: "a".repeat(32),
            authorization: true,
        },
        DevAction::Apply {
            session: "a".repeat(32),
            baseline_token: "b".repeat(32),
            key: "gain".into(),
            value: json!(2),
            authorization: true,
        },
    ]
}

#[test]
fn physical_confirmation_forged_renderer_flags_cannot_send_any_active_rpc() {
    for (index, action) in physical_confirmation_actions().into_iter().enumerate() {
        let mut host = physical_confirmation_host(index != 0);
        let next = host.process.as_ref().unwrap().next_id;
        assert_eq!(
            host.execute(action).unwrap_err(),
            "NATIVE_CONFIRMATION_REQUIRED"
        );
        assert_eq!(host.process.as_ref().unwrap().next_id, next);
        assert!(host.candidates.contains(&"c".repeat(32)));
    }
}

#[test]
fn physical_confirmation_cancel_unavailable_and_timeout_deny_before_rpc() {
    for error in [
        "NATIVE_CONFIRMATION_DENIED",
        "NATIVE_CONFIRMATION_UNAVAILABLE",
    ] {
        for (index, action) in physical_confirmation_actions().into_iter().enumerate() {
            let mut host = physical_confirmation_host(index != 0);
            let next = host.process.as_ref().unwrap().next_id;
            let result = host.execute_with_confirmation(action, |challenge| {
                assert_eq!(challenge.generation, 41);
                assert_eq!(challenge.operation, 1);
                assert_eq!(challenge.plugin_id, "synthetic.confirmation.fixture");
                Err(error.into())
            });
            assert_eq!(result.unwrap_err(), error);
            assert_eq!(host.process.as_ref().unwrap().next_id, next);
            assert!(host.candidates.contains(&"c".repeat(32)));
            assert_eq!(
                error_detail(error.into())["configurationOutcome"],
                "not_sent"
            );
        }
    }
}

#[test]
fn physical_confirmation_approval_is_bound_to_exact_validated_operation() {
    for (index, action) in physical_confirmation_actions().into_iter().enumerate() {
        let mut host = physical_confirmation_host(index != 0);
        let expected_method = ["device.open", "device.get_state", "device.set_setting"][index];
        let next = host.process.as_ref().unwrap().next_id;
        let result = host
            .execute_with_confirmation(action, |challenge| {
                assert_eq!(challenge.method, expected_method);
                assert_eq!(challenge.generation, 41);
                assert_eq!(challenge.operation, 1);
                assert_eq!(challenge.params["authorization"], true);
                if index == 0 {
                    assert_eq!(challenge.params["candidateId"], "c".repeat(32));
                } else {
                    assert_eq!(challenge.params["session"], "a".repeat(32));
                }
                if index == 2 {
                    assert_eq!(challenge.params["key"], "gain");
                    assert_eq!(challenge.params["value"], 2);
                    assert_eq!(challenge.params["baselineToken"], "b".repeat(32));
                    assert!(challenge.description().contains("gain para 2"));
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(result["mode"], "hardware");
        assert_eq!(host.process.as_ref().unwrap().next_id, next + 1);
    }
}

#[test]
fn physical_confirmation_never_caches_approval_for_next_operation_or_generation() {
    let mut host = physical_confirmation_host(true);
    let mut previous = None;
    host.execute_with_confirmation(
        physical_confirmation_actions().into_iter().nth(1).unwrap(),
        |challenge| {
            previous = Some(challenge.clone());
            Ok(())
        },
    )
    .unwrap();
    let next = host.process.as_ref().unwrap().next_id;
    assert_eq!(
        host.execute(physical_confirmation_actions().into_iter().nth(2).unwrap())
            .unwrap_err(),
        "NATIVE_CONFIRMATION_REQUIRED"
    );
    assert_eq!(host.process.as_ref().unwrap().next_id, next);
    // Represent a new process generation; no prior decision is stored or
    // accepted as an input. Its first operation still invokes the provider.
    host.generation += 1;
    let old = previous.unwrap();
    let result = host.execute_with_confirmation(
        physical_confirmation_actions().into_iter().nth(1).unwrap(),
        |challenge| {
            assert_ne!(challenge.generation, old.generation);
            assert_ne!(challenge.operation, old.operation);
            Err("NATIVE_CONFIRMATION_DENIED".into())
        },
    );
    assert_eq!(result.unwrap_err(), "NATIVE_CONFIRMATION_DENIED");
    assert_eq!(host.process.as_ref().unwrap().next_id, next);
}

#[test]
fn physical_confirmation_validation_precedes_prompt_and_rpc() {
    let mut host = physical_confirmation_host(true);
    let next = host.process.as_ref().unwrap().next_id;
    for (session, baseline, value, error) in [
        ("invalid".into(), "b".repeat(32), 2, "INVALID_SESSION"),
        ("a".repeat(32), "invalid".into(), 2, "CONFLICT"),
        ("a".repeat(32), "b".repeat(32), 999, "INVALID_SETTING"),
    ] {
        assert_eq!(
            host.execute_with_confirmation(
                DevAction::Apply {
                    session,
                    baseline_token: baseline,
                    key: "gain".into(),
                    value: json!(value),
                    authorization: true,
                },
                |_| panic!("invalid operation must not show a prompt")
            )
            .unwrap_err(),
            error
        );
    }
    assert_eq!(host.process.as_ref().unwrap().next_id, next);
}

#[test]
fn physical_confirmation_holds_host_exclusive_until_decision() {
    let state = std::sync::Arc::new(std::sync::Mutex::new(physical_confirmation_host(true)));
    let concurrent = state.clone();
    let mut host = state.lock().unwrap();
    host.execute_with_confirmation(
        physical_confirmation_actions().into_iter().nth(1).unwrap(),
        |_| {
            // Same exclusivity as plugin_poc: reentrant IPC/stop/start cannot swap
            // the process, session or baseline while the GTK dialog is pending.
            assert!(concurrent.try_lock().is_err());
            Ok(())
        },
    )
    .unwrap();
}

#[cfg(debug_assertions)]
#[test]
fn physical_confirmation_simulation_never_requests_native_permission() {
    let mut host = started();
    let generation = host.generation;
    let opened = host
        .execute_with_confirmation(DevAction::Open {}, |_| panic!("simulation must not prompt"))
        .unwrap();
    host.execute_with_confirmation(
        DevAction::Read {
            session: opened["session"].as_str().unwrap().into(),
        },
        |_| panic!("simulation must not prompt"),
    )
    .unwrap();
    host.execute(DevAction::Stop {}).unwrap();
    host.execute(DevAction::Start {
        plugin_id: "dev.fake".into(),
        development: true,
    })
    .unwrap();
    assert_eq!(host.generation, generation + 1);
    host.execute(DevAction::Stop {}).unwrap();
}

// Pure policy tests, safe to run under --release: never construct the hardware
// source, spawn Python, enumerate sysfs, or open/read/write a device.
#[test]
fn local_plugin_policy_release_cannot_enable_simulation_or_arbitrary_sources() {
    for id in ["dev.fake", "dev.starter", "dev.another-mouse"] {
        assert_eq!(allowed_plugin(id, true, true), Some(false));
        assert_eq!(allowed_plugin(id, false, true), None);
        for development in [true, false] {
            assert_eq!(allowed_plugin(id, development, false), None);
            if !cfg!(debug_assertions) {
                let mut host = DevHost::default();
                assert_eq!(
                    host.execute(DevAction::Start {
                        plugin_id: id.into(),
                        development
                    })
                    .unwrap_err(),
                    "UNAUTHORIZED_PLUGIN"
                );
                assert!(host.process.is_none());
            }
        }
    }
    for id in [
        "https://example.invalid/plugin.py",
        "/bin/sh",
        "../local.r1.hardware",
        "local.r1.hardware;id",
        "local.r1.hardware/",
        "",
        "r1",
    ] {
        for debug in [true, false] {
            assert_eq!(allowed_plugin(id, true, debug), None);
        }
        let mut host = DevHost::default();
        assert_eq!(
            host.execute(DevAction::Start {
                plugin_id: id.into(),
                development: true,
            })
            .unwrap_err(),
            "UNAUTHORIZED_PLUGIN"
        );
        assert!(host.process.is_none());
    }
    for extra in [
        json!({"path":"/bin/sh"}),
        json!({"args":["-c","id"]}),
        json!({"source":"print('bad')"}),
        json!({"shell":"id"}),
    ] {
        let mut request =
            json!({"action":"start","plugin_id":"local.r1.hardware","development":false});
        request
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        assert!(serde_json::from_value::<DevAction>(request).is_err());
    }
}

fn started() -> DevHost {
    let mut host = DevHost::default();
    host.execute(DevAction::Start {
        plugin_id: "dev.fake".into(),
        development: true,
    })
    .unwrap();
    host
}
fn peer(source: &str) -> Process {
    // Invalid-response assertions must reach parsing/handshake validation, not
    // accidentally test Python startup under load (measured >200 ms). Match the
    // existing host budget; fault-injection tests opt into the short deadline.
    Process::spawn(source, TIMEOUT).unwrap()
}
fn short_peer(source: &str) -> Process {
    Process::spawn(source, Duration::from_millis(200)).unwrap()
}
fn hello() -> Value {
    json!({"jsonrpc":"2.0","id":1,"method":"plugin.hello","params":{"apiVersions":["0.1"]}})
}

#[test]
fn structured_schemas_fail_closed_and_are_bounded() {
    let signed = json!({"type":"integer", "enum":[-2,0,2]});
    assert!(valid_schema(&signed, 0));
    assert!(matches_schema(&signed, &json!(-2), 0));
    assert!(!matches_schema(&signed, &json!(-2.0), 0));
    let snapshot = json!({"mode":"fake","hardwareAccess":false,
        "session":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "baselineToken":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "values":{"example":1}, "outcome":"simulated",
        "baselineTokens":{"example":"cccccccccccccccccccccccccccccccc"},
        "readTimes":{"example":1.0}});
    assert!(valid_result("device.get_state", &snapshot));
    for field in ["baselineTokens", "readTimes"] {
        let mut bad = snapshot.clone();
        bad[field] = json!({"different_key":1});
        assert!(!valid_result("device.get_state", &bad));
    }
    for schema in [
        json!({"type":"object","properties":{},"additionalProperties":true}),
        json!({"type":"array","items":{"type":"boolean"},"minItems":0,"maxItems":33}),
        json!({"type":"number","minimum":0,"maximum":100,"multipleOf":0}),
        json!({"$ref":"file:///dev/hidraw0"}),
        json!({"type":"string","pattern":".*"}),
        json!({"type":"integer","enum":[true]}),
    ] {
        assert!(!valid_schema(&schema, 0));
    }
    let mut s = json!({"type":"boolean"});
    for _ in 0..10 {
        s = json!({"type":"object","properties":{"nested":s}});
    }
    assert!(!valid_schema(&s, 0));
}

#[test]
fn local_identity_handshake_cannot_impersonate_another_plugin() {
    let mut process = Process::spawn(SOURCE, TIMEOUT).unwrap();
    process.identity = "local.r1.simulated".into();
    assert_eq!(process.handshake().unwrap_err(), "INVALID_HANDSHAKE");
}

#[test]
fn start_read_write_close_stop_and_no_autostart() {
    let mut host = DevHost::default();
    assert!(host.process.is_none());
    assert_eq!(host.execute(DevAction::Open {}).unwrap_err(), "NOT_STARTED");
    host = started();
    host.execute(DevAction::Capabilities {}).unwrap();
    assert!(host
        .execute(DevAction::Start {
            plugin_id: "dev.fake".into(),
            development: true
        })
        .is_err());
    let opened = host.execute(DevAction::Open {}).unwrap();
    assert_eq!(opened["hardwareAccess"], false);
    let session = opened["session"].as_str().unwrap().to_owned();
    assert_eq!(
        host.execute(DevAction::Read {
            session: "other".into()
        })
        .unwrap_err(),
        "INVALID_SESSION"
    );
    let state = host
        .execute(DevAction::Read {
            session: session.clone(),
        })
        .unwrap();
    let baseline = state["baselineToken"].as_str().unwrap().to_owned();
    assert_eq!(
        host.execute(DevAction::Set {
            session: session.clone(),
            baseline_token: baseline.clone(),
            key: "polling_rate".into(),
            value: json!(42)
        })
        .unwrap_err(),
        "INVALID_SETTING"
    );
    let result = host
        .execute(DevAction::Set {
            session: session.clone(),
            baseline_token: baseline.clone(),
            key: "polling_rate".into(),
            value: json!(1000),
        })
        .unwrap();
    assert_eq!(result["outcome"], "simulated");
    assert_eq!(result["values"]["polling_rate"], 1000);
    assert_eq!(
        host.execute(DevAction::Set {
            session: session.clone(),
            baseline_token: baseline,
            key: "polling_rate".into(),
            value: json!(500)
        })
        .unwrap_err(),
        "CONFLICT"
    );
    host.execute(DevAction::Close {
        session: session.clone(),
    })
    .unwrap();
    host.execute(DevAction::Stop {}).unwrap();
    host.execute(DevAction::Stop {}).unwrap();
    assert!(host.process.is_none());
    host.execute(DevAction::Start {
        plugin_id: "dev.fake".into(),
        development: true,
    })
    .unwrap();
    host.execute(DevAction::Open {}).unwrap();
    assert_eq!(
        host.execute(DevAction::Read { session }).unwrap_err(),
        "INVALID_SESSION"
    );
    host.execute(DevAction::Stop {}).unwrap();
}

#[test]
fn frontend_cannot_supply_paths_args_shell_or_unapproved_plugin() {
    let mut host = DevHost::default();
    for plugin_id in ["/bin/sh", "../dev.fake", "dev.fake;id", "r1"] {
        assert_eq!(
            host.execute(DevAction::Start {
                plugin_id: plugin_id.into(),
                development: true
            })
            .unwrap_err(),
            "UNAUTHORIZED_PLUGIN"
        );
    }
    assert!(host
        .execute(DevAction::Start {
            plugin_id: "dev.fake".into(),
            development: false
        })
        .is_err());
    for request in [
        json!({"action":"start","plugin_id":"dev.fake","development":true,"path":"/bin/sh"}),
        json!({"action":"start","plugin_id":"dev.fake","development":true,"args":["-c","id"]}),
        json!({"action":"start","plugin_id":"dev.fake","development":true,"shell":"id"}),
        json!({"action":"open","candidateId":"/dev/hidraw0"}),
        json!({"action":"rpc","method":"shell"}),
    ] {
        assert!(serde_json::from_value::<DevAction>(request).is_err());
    }
    assert!(host.process.is_none());
}

#[test]
fn valid_and_invalid_handshakes() {
    let mut good = Process::spawn(SOURCE, TIMEOUT).unwrap();
    assert_eq!(
        good.call("plugin.hello", json!({"apiVersions":["0.1"]}))
            .unwrap()["hardwareAccess"],
        false
    );
    good.shutdown().unwrap();
    assert!(good.reaped); // Drop must not signal a PID/group after wait reaped it.
    let mut bad = Process::spawn(SOURCE, TIMEOUT).unwrap();
    assert_eq!(
        bad.call("plugin.hello", json!({"apiVersions":["9"]}))
            .unwrap_err(),
        "PLUGIN_RPC_ERROR"
    );
    for source in [
        "import sys,json;sys.stdin.readline();print(json.dumps({'jsonrpc':'2.0','id':1,'result':{'apiVersion':'9','mode':'fake'}}),flush=True)",
        "import sys,json;sys.stdin.readline();print(json.dumps({'jsonrpc':'2.0','id':1,'result':{'apiVersion':'0.1','mode':'hardware'}}),flush=True)",
    ] {
        let mut process = peer(source);
        assert_eq!(process.handshake().unwrap_err(), "INVALID_HANDSHAKE");
    }
}

#[test]
fn delayed_invalid_handshake_reaches_validation_not_fault_injection_timeout() {
    let mut process = peer("import sys,json,time;sys.stdin.readline();time.sleep(0.3);print(json.dumps({'jsonrpc':'2.0','id':1,'result':{'apiVersion':'9','mode':'fake'}}),flush=True)");
    assert_eq!(process.timeout, TIMEOUT);
    assert_eq!(process.handshake().unwrap_err(), "INVALID_HANDSHAKE");
}

#[test]
fn batch_notifications_invalid_elements_and_serial_execution() {
    let mut process = Process::spawn(SOURCE, TIMEOUT).unwrap();
    let reply = process
        .exchange(json!([
            {"jsonrpc":"2.0","method":"plugin.hello","params":{"apiVersions":["0.1"]}},
            {"jsonrpc":"2.0","id":1,"method":"plugin.get_capabilities","params":{}},
            {"jsonrpc":"2.0","method":"unknown"},
            {"jsonrpc":"2.0","id":2,"method":"device.open","params":{"candidateId":"fake-device"}}
        ]))
        .unwrap();
    assert_eq!(reply.as_array().unwrap().len(), 2);
    assert!(reply[0]["result"]["settings"].is_array());
    let session = reply[1]["result"]["session"].clone();
    let reply = process
        .exchange(json!([
            {"jsonrpc":"2.0","method":"device.close","params":{"session":session}},
            {"jsonrpc":"2.0","id":3,"method":"device.get_state","params":{"session":session}},
            {"jsonrpc":"2.0","id":4,"method":"device.close","params":{"session":session}},
            {"jsonrpc":"2.0","id":5,"method":"device.get_state","params":{"session":session}}
        ]))
        .unwrap();
    assert_eq!(reply[0]["result"]["outcome"], "simulated");
    assert_eq!(reply[2]["error"]["data"]["kind"], "INVALID_SESSION");
    // Host rejects malformed ID-null response elements; the independent Python
    // conformance suite checks their correct -32600 responses on the wire.
    process.shutdown().unwrap();
}

#[test]
fn pure_notifications_do_not_leave_responses() {
    let mut process = Process::spawn(SOURCE, TIMEOUT).unwrap();
    process.exchange(json!([{"jsonrpc":"2.0","method":"plugin.hello","params":{"apiVersions":["0.1"]}},{"jsonrpc":"2.0","method":"unknown"}])).unwrap();
    process.call("plugin.get_capabilities", json!({})).unwrap();
    process.shutdown().unwrap();
}

#[test]
fn wrong_duplicate_missing_response_ids_and_bad_schemas() {
    for (wire, batch) in [
        (r#"{"jsonrpc":"2.0","id":2,"result":{}}"#, false),
        (
            r#"[{"jsonrpc":"2.0","id":1,"result":{}},{"jsonrpc":"2.0","id":1,"result":{}}]"#,
            true,
        ),
        (r#"[{"jsonrpc":"2.0","id":1,"result":{}}]"#, true),
        (
            r#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":1,"message":"x"}}"#,
            false,
        ),
        (
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":true,"message":"x"}}"#,
            false,
        ),
        (r#"{"jsonrpc":"1.0","id":1,"result":{}}"#, false),
        (r#"{"jsonrpc":"2.0","id":1,"id":1,"result":{}}"#, false),
    ] {
        let source = format!("import sys;sys.stdin.readline();print({wire:?},flush=True)");
        let mut process = peer(&source);
        let request = if batch {
            json!([hello(),{"jsonrpc":"2.0","id":2,"method":"plugin.get_capabilities"}])
        } else {
            hello()
        };
        assert!(process.exchange(request).is_err(), "{wire}");
    }
    assert!(!valid_result(
        "device.get_state",
        &json!({"mode":"hardware"})
    ));
    assert!(!valid_result(
        "device.open",
        &json!({"mode":"fake","hardwareAccess":false,"session":"bad","identification":"simulated"})
    ));
}

#[test]
fn crash_timeout_stdout_contamination_and_oversize_are_bounded() {
    for (source, expected) in [
        ("import sys;sys.stdin.readline();sys.exit(2)","PLUGIN_EXITED"),
        ("import time;time.sleep(60)","TIMEOUT"),
        ("import sys;sys.stdin.readline();print('debug log',flush=True)","INVALID_STDOUT_JSON"),
        ("import sys;sys.stdin.readline();print('x'*262145,flush=True)","INVALID_STDOUT_FRAME"),
        ("import sys;sys.stdin.readline();sys.stdout.buffer.write(b'\\xff\\n');sys.stdout.flush()","INVALID_STDOUT_JSON"),
        ("import sys;sys.stdin.readline();print('{',flush=True)","INVALID_STDOUT_JSON"),
    ] {
        let start = Instant::now();
        let mut process = if expected == "TIMEOUT" { short_peer(source) } else { peer(source) };
        assert_eq!(process.exchange(hello()).unwrap_err(),expected);
        drop(process);
        assert!(start.elapsed()<Duration::from_secs(3));
    }
}

#[test]
fn bounded_writer_when_peer_does_not_read() {
    let start = Instant::now();
    let mut process = short_peer("import time;time.sleep(60)");
    assert_eq!(
        process
            .exchange(
                json!({"jsonrpc":"2.0","id":1,"method":"x","params":{"large":"x".repeat(200000)}})
            )
            .unwrap_err(),
        "TIMEOUT"
    );
    drop(process);
    assert!(start.elapsed() < Duration::from_secs(3));
}

#[test]
fn stderr_is_drained_without_stdout_pollution() {
    let source =
        format!("import sys\nsys.stderr.write('private'*200000)\nsys.stderr.flush()\n{SOURCE}");
    let mut process = Process::spawn(&source, TIMEOUT).unwrap();
    process
        .call("plugin.hello", json!({"apiVersions":["0.1"]}))
        .unwrap();
    process.shutdown().unwrap();
    assert!(process.stderr_bytes.load(Ordering::Relaxed) >= 1400000);
}

#[test]
fn excess_stdout_and_notification_flood_fail_closed() {
    let mut process = peer("import sys,json;sys.stdin.readline();print('\\n'.join([json.dumps({'jsonrpc':'2.0','id':1,'result':{}})]*40),flush=True);sys.stdin.readline()");
    let first = process.exchange(hello());
    if first.is_ok() {
        assert!(process.call("next", json!({})).is_err());
    }
    let mut process = peer("import sys,json;sys.stdin.readline();print('\\n'.join([json.dumps({'jsonrpc':'2.0','method':'event','params':{}})]*40),flush=True);sys.stdin.readline()");
    assert!(process.exchange(hello()).is_err());
}

#[test]
fn shutdown_timeout_kills_and_reaps_child() {
    let mut process = peer("import sys,json,time;sys.stdin.readline();print(json.dumps({'jsonrpc':'2.0','id':1,'result':{}}),flush=True);sys.stdin.readline();print(json.dumps({'jsonrpc':'2.0','id':2,'result':{'mode':'fake','stopped':True}}),flush=True);time.sleep(60)");
    // Startup has its own normal budget; only the stuck shutdown is timed short.
    process.exchange(hello()).unwrap();
    process.timeout = Duration::from_millis(200);
    assert_eq!(process.shutdown().unwrap_err(), "SHUTDOWN_TIMEOUT");
    let pid = process.child.id();
    drop(process);
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
}

#[test]
#[cfg(target_os = "linux")]
fn cleanup_kills_descendants_retaining_pipes_in_child_group() {
    let start = Instant::now();
    let mut process = peer("import os,time,sys,json\nif os.fork()==0: time.sleep(60);os._exit(0)\nsys.stdin.readline()\nprint(json.dumps({'jsonrpc':'2.0','id':1,'result':{}}),flush=True)\ntime.sleep(60)");
    process.exchange(hello()).unwrap();
    drop(process);
    assert!(start.elapsed() < Duration::from_secs(3));
}

#[test]
fn serialized_backend_state_refuses_concurrent_operation() {
    let state = std::sync::Mutex::new(DevHost::default());
    let _operation = state.lock().unwrap();
    assert!(state.try_lock().is_err());
}

#[test]
fn timeout_destroys_session_and_manual_restart_does_not_replay() {
    let mut host = started();
    let session = host.execute(DevAction::Open {}).unwrap()["session"]
        .as_str()
        .unwrap()
        .to_owned();
    let state = host
        .execute(DevAction::Read {
            session: session.clone(),
        })
        .unwrap();
    let baseline_token = state["baselineToken"].as_str().unwrap().to_owned();
    // Only a private test seam can substitute a peer. It simulates losing the
    // response after an in-memory mutation, never touches a device.
    host.process = Some(short_peer(
        "import sys,time;sys.stdin.readline();simulated_value=1000;time.sleep(60)",
    ));
    assert_eq!(
        host.execute(DevAction::Set {
            session: session.clone(),
            baseline_token,
            key: "polling_rate".into(),
            value: json!(1000)
        })
        .unwrap_err(),
        "TIMEOUT"
    );
    assert!(host.process.is_none() && host.session.is_none() && host.baseline.is_none());
    host.execute(DevAction::Start {
        plugin_id: "dev.fake".into(),
        development: true,
    })
    .unwrap();
    assert_eq!(
        host.execute(DevAction::Read { session }).unwrap_err(),
        "INVALID_SESSION"
    );
    let session = host.execute(DevAction::Open {}).unwrap()["session"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        host.execute(DevAction::Read { session }).unwrap()["values"]["polling_rate"],
        500
    );
    host.execute(DevAction::Stop {}).unwrap();
}

#[test]
fn host_rejects_result_schema_and_cross_session_response() {
    for wire in [
        r#"{"jsonrpc":"2.0","id":1,"result":{"mode":"hardware"}}"#,
        r#"{"jsonrpc":"2.0","id":1,"result":{"mode":"fake","hardwareAccess":false,"session":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","baselineToken":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","values":{"polling_rate":500},"outcome":"simulated"}}"#,
    ] {
        let mut host = started();
        let session = host.execute(DevAction::Open {}).unwrap()["session"]
            .as_str()
            .unwrap()
            .to_owned();
        host.process = Some(peer(&format!(
            "import sys;sys.stdin.readline();print({wire:?},flush=True)"
        )));
        assert!(host.execute(DevAction::Read { session }).is_err());
        assert!(host.process.is_none() && host.session.is_none() && host.baseline.is_none());
    }
}

#[test]
fn capability_validation_is_generic_and_bounded() {
    let caps = json!({"mode":"fake","hardwareAccess":false,"settings":[{"key":"example_level","access":["read","write"],"schema":{"type":"integer","enum":[1,2]},"effects":"simulation_only"}]});
    assert!(valid_result("plugin.get_capabilities", &caps));
    assert!(allows(&caps, "example_level", &json!(2), "write"));
    assert!(!allows(&caps, "example_level", &json!(3), "write"));
    assert!(!allows(&caps, "example_level", &json!(true), "write"));
    let mut bad = caps.clone();
    bad["settings"][0]["effects"] = json!("hardware");
    assert!(!valid_result("plugin.get_capabilities", &bad));
}

#[test]
fn trailing_stdout_after_shutdown_is_not_silently_discarded() {
    let mut process = peer("import sys,json;sys.stdin.readline();print(json.dumps({'jsonrpc':'2.0','id':1,'result':{'mode':'fake','stopped':True}}),flush=True);print('unexpected',flush=True)");
    assert!(process.shutdown().is_err());
}

#[test]
fn parser_limits_duplicates_nonfinite_and_utf8() {
    for raw in [
        b"{\"x\":1,\"x\":2}".as_slice(),
        b"NaN",
        b"Infinity",
        b"\xff",
        b"{} {}",
    ] {
        assert!(strict_json(raw).is_err());
    }
    assert!(strict_json(format!("{}0{}", "[".repeat(33), "]".repeat(33)).as_bytes()).is_err());
    assert!(strict_json(&vec![b' '; MAX_LINE + 1]).is_err());
    assert_eq!(
        strict_json(br#"{"x":[true,false,null,1,-1,0.5,"ok"]}"#).unwrap()["x"][5],
        0.5
    );
}

#[test]
fn hardware_validation_is_generic_and_does_not_accept_fake_evidence() {
    let caps = json!({"mode":"hardware","hardwareAccess":true,"settings":[{"key":"gain","schema":{"type":"integer","enum":[1,2]},"access":["read","write"],"effects":"active_read_and_configuration_write"}]});
    assert!(valid_mode_result("plugin.get_capabilities", &caps, true));
    assert!(!valid_mode_result("plugin.get_capabilities", &caps, false));
    let mut bad = caps.clone();
    bad["settings"][0]["effects"] = json!("simulation_only");
    assert!(!valid_mode_result("plugin.get_capabilities", &bad, true));
    let state = json!({"mode":"hardware","hardwareAccess":true,"session":"a".repeat(32),"baselineToken":"b".repeat(32),
        "values":{"gain":1},"baselineTokens":{"gain":"c".repeat(32)},"readTimes":{"gain":1.0},"outcome":"confirmed_by_readback"});
    assert!(valid_mode_result("device.set_setting", &state, true));
    assert!(!valid_mode_result("device.get_state", &state, true));
    let mut fake = state.clone();
    fake["outcome"] = json!("simulated");
    assert!(!valid_mode_result("device.set_setting", &fake, true));
    let mut unknown = state.clone();
    unknown["outcome"] = json!("accepted");
    assert!(!valid_mode_result("device.set_setting", &unknown, true));
}

#[test]
fn structured_remote_errors_are_sanitized_and_drop_hardware_generation() {
    for kind in [
        "PERMISSION_DENIED",
        "IDENTITY_UNCONFIRMED",
        "BACKUP_FAILED",
        "VERIFICATION_FAILED",
        "CONFLICT",
    ] {
        let data = json!({"kind":kind,"configurationOutcome":if kind=="VERIFICATION_FAILED" {"unconfirmed"} else {"not_sent"},
            "stage":"reading","operationId":"a".repeat(32),"retryable":false});
        let wire=json!({"jsonrpc":"2.0","id":1,"error":{"code":-32008,"message":"untrusted private path ignored","data":data}}).to_string();
        let mut host = started();
        host.hardware = true;
        host.process = Some(peer(&format!(
            "import sys;sys.stdin.readline();print({wire:?},flush=True)"
        )));
        host.process.as_mut().unwrap().hardware = true;
        let error = host.execute(DevAction::Discover {}).unwrap_err();
        assert_eq!(error_detail(error), data);
        assert!(host.process.is_none() && host.session.is_none());
    }
    let bad = json!({"kind":"/private/path","configurationOutcome":"not_sent","stage":"reading","operationId":null,"retryable":false});
    assert!(!valid_error_data(&bad));
}

#[test]
fn hardware_unknown_dispatch_errors_never_claim_not_sent_or_replay() {
    let detail = error_detail(operation_error(
        true,
        "device.set_setting",
        "INVALID_STDOUT_JSON",
    ));
    assert_eq!(detail["kind"], "PROTOCOL_VIOLATION");
    assert_eq!(detail["configurationOutcome"], "unknown");
    assert_eq!(
        error_detail(operation_error(true, "device.set_setting", "TIMEOUT"))
            ["configurationOutcome"],
        "unknown"
    );
    assert_eq!(
        error_detail(operation_error(true, "device.get_state", "TIMEOUT"))["configurationOutcome"],
        "not_sent"
    );
    assert_eq!(
        error_detail("HOST_TASK_FAILED".into())["configurationOutcome"],
        "unknown"
    );
    assert!(serde_json::from_value::<DevAction>(json!({"action":"apply","session":"x","baseline_token":"x","key":"x","value":1,"authorization":true,"path":"/dev/hidraw2"})).is_err());
}
