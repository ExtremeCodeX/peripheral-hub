//! Frame/schema rejection without any hardware or external service.
use peripheral_hub_device_sdk::{LightingCommand, LightingEffect, RgbColor};
use peripheral_hub_ipc::*;
use peripheral_hub_macros::{KeyPress, Macro, MacroAction, MacroId, MacroRepeat};
use peripheral_hub_profiles::ProfileId;
use tokio::io::{AsyncWriteExt, duplex};

#[tokio::test]
async fn framed_roundtrip_and_invalid_lengths_fail_before_payload_allocation() {
    let (mut a, mut b) = duplex(1024);
    let request = Request {
        version: IPC_VERSION,
        request_id: 42,
        operation: Operation::Health,
    };
    write_frame(&mut a, &request).await.unwrap();
    let decoded: Request = read_frame(&mut b).await.unwrap();
    assert_eq!(decoded.request_id, 42);
    for length in [0, MAX_FRAME_BYTES as u32 + 1, u32::MAX] {
        let (mut a, mut b) = duplex(16);
        a.write_u32(length).await.unwrap();
        assert!(matches!(
            read_frame::<Request>(&mut b).await,
            Err(IpcError::InvalidLength)
        ));
    }
}

#[tokio::test]
async fn malformed_unknown_operations_and_truncated_frames_are_rejected() {
    for bytes in [
        b"not-json".as_slice(),
        br#"{"version":1,"request_id":1,"operation":{"operation":"raw_hid_write"}}"#.as_slice(),
        br#"{"version":1,"request_id":1,"operation":{"operation":"health"},"extra":true}"#
            .as_slice(),
    ] {
        let (mut a, mut b) = duplex(1024);
        a.write_u32(bytes.len() as u32).await.unwrap();
        a.write_all(bytes).await.unwrap();
        assert!(matches!(
            read_frame::<Request>(&mut b).await,
            Err(IpcError::InvalidMessage)
        ));
    }
    let (mut a, mut b) = duplex(16);
    a.write_u32(5).await.unwrap();
    a.write_all(b"x").await.unwrap();
    drop(a);
    assert!(matches!(
        read_frame::<Request>(&mut b).await,
        Err(IpcError::Io(_))
    ));
}

#[test]
fn lighting_scene_shape_is_validated() {
    let command = |zone: &str| LightingCommand {
        zone: zone.into(),
        color: RgbColor {
            red: 1,
            green: 2,
            blue: 3,
        },
        brightness: None,
        effect: LightingEffect::Static,
    };
    assert!(
        LightingScene {
            commands: vec![command("whole-device")]
        }
        .validate()
        .is_ok()
    );
    assert!(LightingScene { commands: vec![] }.validate().is_err());
    assert!(
        LightingScene {
            commands: vec![command("a"), command("a")]
        }
        .validate()
        .is_err()
    );
    let oversized: Vec<_> = (0..=MAX_SCENE_ZONES)
        .map(|index| command(&format!("z{index}")))
        .collect();
    assert!(
        LightingScene {
            commands: oversized
        }
        .validate()
        .is_err()
    );
}

#[tokio::test]
async fn macro_operations_roundtrip_through_the_frame() {
    let macro_value = Macro {
        id: MacroId::new("combo").unwrap(),
        name: "Combo".into(),
        actions: vec![
            MacroAction::Key {
                key: "f13".into(),
                press: KeyPress::Tap,
            },
            MacroAction::Delay { ms: 20 },
        ],
        repeat: MacroRepeat {
            times: 2,
            interval_ms: 5,
        },
        profile: Some(ProfileId::new("gaming").unwrap()),
    };
    let request = Request {
        version: IPC_VERSION,
        request_id: 7,
        operation: Operation::SaveMacro {
            macro_value: macro_value.clone(),
        },
    };
    let (mut a, mut b) = duplex(4096);
    write_frame(&mut a, &request).await.unwrap();
    let decoded: Request = read_frame(&mut b).await.unwrap();
    match decoded.operation {
        Operation::SaveMacro {
            macro_value: decoded_macro,
        } => assert_eq!(decoded_macro, macro_value),
        _ => panic!("unexpected operation"),
    }
}

#[test]
fn macro_fault_is_a_stable_sanitized_code() {
    let fault = Fault::from(peripheral_hub_macros::MacroError::NotFound);
    assert_eq!(fault, Fault::Macro(MacroFault::NotFound));
    let json = serde_json::to_string(&fault).unwrap();
    assert!(json.contains("macro") && json.contains("not_found"));
}

#[test]
fn wire_errors_do_not_export_backend_secrets() {
    let fault = Fault::from(peripheral_hub_device_sdk::DeviceError::TransportError(
        "PRIVATE-SERIAL/path".into(),
    ));
    assert!(!serde_json::to_string(&fault).unwrap().contains("PRIVATE"));
}
