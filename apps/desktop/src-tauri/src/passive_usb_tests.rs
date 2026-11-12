use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = Path::new("/tmp/opencode").join(format!(
            "passive-usb-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }
    fn device(&self, name: &str, pid: &str) {
        self.write(&format!("{name}/idVendor"), "1D57\n");
        self.write(&format!("{name}/idProduct"), pid);
        // Deliberately not configuration 1. Never assume :1.2.
        for (key, value) in [
            ("bInterfaceNumber", "02"),
            ("bInterfaceClass", "03"),
            ("bInterfaceSubClass", "00"),
            ("bInterfaceProtocol", "00"),
        ] {
            self.write(&format!("{name}:2.2/{key}"), value);
        }
        fs::create_dir_all(self.0.join(format!("{name}:2.2/hid-device/hidraw/hidraw7"))).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn empty_and_unavailable_are_distinct() {
    let fixture = Fixture::new();
    let empty = scan(&Sysfs, &fixture.0);
    assert!(empty.devices.is_empty() && empty.issues.is_empty());
    assert!(scan(&Sysfs, &fixture.0.join("missing"))
        .issues
        .contains("SYSFS_CHANGED_OR_UNAVAILABLE"));
}
#[test]
fn cable_receiver_multiple_and_non_hid_are_generic_metadata() {
    let f = Fixture::new();
    f.device("1-1", "fa61");
    f.device("1-2", "fa60");
    f.write("1-1:2.3/bInterfaceClass", "08");
    let found = scan(&Sysfs, &f.0);
    assert!(found.issues.is_empty());
    assert_eq!(found.devices.len(), 2);
    assert_eq!(found.devices[0].vid, "1d57");
    assert_eq!(found.devices[0].pid, "fa61");
    assert_eq!(found.devices[1].pid, "fa60");
    assert_eq!(found.devices[0].interfaces.len(), 1);
    assert_eq!(found.devices[0].interfaces[0].number.as_deref(), Some("02"));
    assert_eq!(found.devices[0].interfaces[0].hidraw, ["hidraw7"]);
    assert_eq!(found.hid_access, "not_checked");
}
#[test]
fn inaccessible_metadata_is_partial_not_absent_and_never_claims_hid_permission() {
    struct Denied;
    impl Metadata for Denied {
        fn text(&self, p: &Path) -> io::Result<String> {
            if p.ends_with("bInterfaceNumber") {
                Err(io::ErrorKind::PermissionDenied.into())
            } else {
                Sysfs.text(p)
            }
        }
        fn directories(&self, p: &Path, root: bool) -> io::Result<Vec<PathBuf>> {
            Sysfs.directories(p, root)
        }
    }
    let f = Fixture::new();
    f.device("1-1", "fa60");
    let found = scan(&Denied, &f.0);
    assert_eq!(found.devices.len(), 1);
    assert!(found.devices[0].interfaces[0].number.is_none());
    assert!(found.issues.contains("SYSFS_PERMISSION_DENIED"));
    assert_eq!(found.hid_access, "not_checked");
}
#[test]
fn malformed_or_disappeared_attributes_do_not_confirm_identity() {
    let f = Fixture::new();
    f.device("1-1", "fa61");
    f.write("1-1:2.2/bInterfaceNumber", "not-a-number");
    fs::remove_file(f.0.join("1-1:2.2/bInterfaceProtocol")).unwrap();
    let found = scan(&Sysfs, &f.0);
    assert!(found.devices[0].interfaces[0].number.is_none());
    assert!(found.issues.contains("SYSFS_INVALID_METADATA"));
    assert!(found.issues.contains("SYSFS_CHANGED_OR_UNAVAILABLE"));
}
#[test]
fn ignores_serial_product_descriptors_and_links_without_exposing_paths() {
    let f = Fixture::new();
    f.device("1-1", "fa60");
    f.write("1-1/serial", "SECRET");
    f.write("1-1/product", "SECRET");
    f.write("1-1:2.2/hid-device/report_descriptor", "SECRET");
    #[cfg(unix)]
    std::os::unix::fs::symlink("/dev", f.0.join("1-1:2.2/device")).unwrap();
    let output = serde_json::to_string(&scan(&Sysfs, &f.0)).unwrap();
    for secret in [
        "SECRET",
        "/dev",
        "/sys",
        "1-1",
        "serial",
        "report_descriptor",
    ] {
        assert!(!output.contains(secret));
    }
}
