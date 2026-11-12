//! Passive Linux USB metadata only. No /dev access, HID imports, RPC or plugins.
//! The kernel's sysfs links at the USB root are expected; nested links are not followed.
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub source: &'static str,
    pub devices: Vec<UsbDevice>,
    pub issues: BTreeSet<&'static str>,
    pub hid_access: &'static str,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
    pub vid: String,
    pub pid: String,
    pub usb_revision: Option<String>,
    pub interfaces: Vec<UsbInterface>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbInterface {
    pub number: Option<String>,
    pub class: Option<String>,
    pub subclass: Option<String>,
    pub protocol: Option<String>,
    pub hidraw: Vec<String>,
}

// The root is private and fixed in production; never supplied via IPC/CLI.
pub fn enumerate() -> Inventory {
    if !cfg!(target_os = "linux") {
        return Inventory {
            source: "linux-sysfs",
            devices: vec![],
            issues: BTreeSet::from(["UNSUPPORTED_PLATFORM"]),
            hid_access: "not_checked",
        };
    }
    scan(&Sysfs, Path::new("/sys/bus/usb/devices"))
}

trait Metadata {
    fn text(&self, path: &Path) -> io::Result<String>;
    fn directories(&self, path: &Path, root: bool) -> io::Result<Vec<PathBuf>>;
}
struct Sysfs;
impl Metadata for Sysfs {
    fn text(&self, path: &Path) -> io::Result<String> {
        let mut value = String::new();
        fs::File::open(path)?.take(17).read_to_string(&mut value)?;
        if value.len() > 16 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(value.trim().into())
    }
    fn directories(&self, path: &Path, root: bool) -> io::Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for (index, entry) in fs::read_dir(path)?.take(1025).enumerate() {
            if index == 1024 {
                return Err(io::ErrorKind::InvalidData.into());
            }
            let entry = entry?;
            if root || entry.file_type()?.is_dir() {
                entries.push(entry.path());
            }
        }
        entries.sort();
        Ok(entries)
    }
}
fn issue(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::PermissionDenied => "SYSFS_PERMISSION_DENIED",
        io::ErrorKind::NotFound => "SYSFS_CHANGED_OR_UNAVAILABLE",
        io::ErrorKind::InvalidData => "SYSFS_INVALID_METADATA",
        _ => "SYSFS_UNAVAILABLE",
    }
}
fn hex(
    fs: &impl Metadata,
    path: &Path,
    width: usize,
    optional: bool,
    issues: &mut BTreeSet<&'static str>,
) -> Option<String> {
    match fs.text(path) {
        Ok(s) if s.len() == width && s.bytes().all(|b| b.is_ascii_hexdigit()) => {
            Some(s.to_ascii_lowercase())
        }
        Ok(_) => {
            issues.insert("SYSFS_INVALID_METADATA");
            None
        }
        Err(e) => {
            if !(optional && e.kind() == io::ErrorKind::NotFound) {
                issues.insert(issue(&e));
            }
            None
        }
    }
}
fn directories(
    fs: &impl Metadata,
    path: &Path,
    root: bool,
    issues: &mut BTreeSet<&'static str>,
) -> Vec<PathBuf> {
    fs.directories(path, root).unwrap_or_else(|e| {
        issues.insert(issue(&e));
        vec![]
    })
}
fn basename(path: &Path) -> &str {
    path.file_name().and_then(|s| s.to_str()).unwrap_or("")
}
fn hidraw(fs: &impl Metadata, path: &Path, issues: &mut BTreeSet<&'static str>) -> Vec<String> {
    let mut names = Vec::new();
    // Interface / HID device / hidraw / hidrawN. Do not walk subsystem/device links.
    for hid in directories(fs, path, false, issues) {
        for group in directories(fs, &hid, false, issues) {
            if basename(&group) != "hidraw" {
                continue;
            }
            for node in directories(fs, &group, false, issues) {
                if basename(&node)
                    .strip_prefix("hidraw")
                    .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                {
                    names.push(basename(&node).to_owned());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}
fn scan(fs: &impl Metadata, root: &Path) -> Inventory {
    let mut inventory = Inventory {
        source: "linux-sysfs",
        devices: vec![],
        issues: BTreeSet::new(),
        hid_access: "not_checked",
    };
    let paths = directories(fs, root, true, &mut inventory.issues);
    for path in &paths {
        let name = basename(path);
        if name.contains(':') || name.is_empty() {
            continue;
        }
        let Some(vid) = hex(fs, &path.join("idVendor"), 4, false, &mut inventory.issues) else {
            continue;
        };
        let Some(pid) = hex(fs, &path.join("idProduct"), 4, false, &mut inventory.issues) else {
            continue;
        };
        let mut device = UsbDevice {
            vid,
            pid,
            usb_revision: hex(fs, &path.join("bcdDevice"), 4, true, &mut inventory.issues),
            interfaces: vec![],
        };
        for interface in &paths {
            if !basename(interface).starts_with(&format!("{name}:")) {
                continue;
            }
            let class = hex(
                fs,
                &interface.join("bInterfaceClass"),
                2,
                false,
                &mut inventory.issues,
            );
            if class.as_deref().is_some_and(|c| c != "03") {
                continue;
            }
            device.interfaces.push(UsbInterface {
                number: hex(
                    fs,
                    &interface.join("bInterfaceNumber"),
                    2,
                    false,
                    &mut inventory.issues,
                ),
                class,
                subclass: hex(
                    fs,
                    &interface.join("bInterfaceSubClass"),
                    2,
                    false,
                    &mut inventory.issues,
                ),
                protocol: hex(
                    fs,
                    &interface.join("bInterfaceProtocol"),
                    2,
                    false,
                    &mut inventory.issues,
                ),
                hidraw: hidraw(fs, interface, &mut inventory.issues),
            });
        }
        // Keep devices with missing HID interfaces visible to the integration too:
        // missing metadata is not evidence of absence or incompatibility.
        inventory.devices.push(device);
    }
    inventory
}

#[cfg(test)]
#[path = "passive_usb_tests.rs"]
mod tests;
