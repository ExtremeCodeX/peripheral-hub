//! Frame/schema rejection without any hardware or external service.
use peripheral_hub_ipc::*;
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
fn wire_errors_do_not_export_backend_secrets() {
    let fault = Fault::from(peripheral_hub_device_sdk::DeviceError::TransportError(
        "PRIVATE-SERIAL/path".into(),
    ));
    assert!(!serde_json::to_string(&fault).unwrap().contains("PRIVATE"));
}
