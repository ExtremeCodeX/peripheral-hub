//! Linux/Unix endpoint security and stale-instance behavior; no privilege changes.
#![cfg(unix)]
use peripheral_hub_ipc::*;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
fn tempdir() -> std::io::Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))?;
    Ok(directory)
}

#[tokio::test]
async fn private_socket_duplicate_instance_and_clean_restart() {
    let directory = tempdir().unwrap();
    let endpoint = Endpoint::new(directory.path().join("daemon.sock"));
    let listener = Listener::bind(&endpoint).await.unwrap();
    assert_eq!(
        fs::metadata(endpoint.path()).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(Listener::bind(&endpoint).await.is_err());
    let mut client = Client::new(endpoint.clone(), std::time::Duration::from_millis(100)).unwrap();
    let task = tokio::spawn(async move { client.call(Operation::Health).await });
    let mut stream = listener.accept().await.unwrap();
    let request: Request = read_frame(&mut stream).await.unwrap();
    write_frame(
        &mut stream,
        &Response {
            version: IPC_VERSION,
            request_id: request.request_id,
            result: Err(Fault::Busy),
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        task.await.unwrap().unwrap().result,
        Err(Fault::Busy)
    ));
    drop(stream);
    drop(listener);
    assert!(!endpoint.path().exists());
    assert!(Listener::bind(&endpoint).await.is_ok());
}

#[tokio::test]
async fn stale_socket_is_recovered_but_other_files_and_symlinks_are_preserved() {
    let directory = tempdir().unwrap();
    let endpoint = Endpoint::new(directory.path().join("daemon.sock"));
    let stale = std::os::unix::net::UnixListener::bind(endpoint.path()).unwrap();
    fs::set_permissions(endpoint.path(), fs::Permissions::from_mode(0o600)).unwrap();
    drop(stale);
    let listener = Listener::bind(&endpoint).await.unwrap();
    drop(listener);
    fs::write(endpoint.path(), b"preserve user file").unwrap();
    assert!(Listener::bind(&endpoint).await.is_err());
    assert_eq!(fs::read(endpoint.path()).unwrap(), b"preserve user file");
    fs::remove_file(endpoint.path()).unwrap();
    let target = directory.path().join("private-file");
    fs::write(&target, b"keep").unwrap();
    symlink(&target, endpoint.path()).unwrap();
    assert!(Listener::bind(&endpoint).await.is_err());
    assert_eq!(fs::read(target).unwrap(), b"keep");
}

#[tokio::test]
async fn permissive_directories_and_symlink_lockfiles_are_rejected() {
    let directory = tempdir().unwrap();
    let endpoint = Endpoint::new(directory.path().join("daemon.sock"));
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Listener::bind(&endpoint).await.is_err());
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let target = directory.path().join("target");
    fs::write(&target, b"keep").unwrap();
    symlink(&target, directory.path().join("daemon.sock.lock")).unwrap();
    assert!(Listener::bind(&endpoint).await.is_err());
    assert_eq!(fs::read(target).unwrap(), b"keep");
}
