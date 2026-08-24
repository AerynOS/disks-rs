// SPDX-FileCopyrightText: Copyright © 2026 aerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Resolving a mount point to the devices beneath it.
//!
//! A mounted filesystem may sit directly on a partition, or on a stack of
//! device-mapper layers: btrfs on LVM on LUKS on a partiton. This module
//! answers what is underneath, all the way down.
//!
//! Three things here are deliberate.
//!
//! **Directory entries are sorted.** `read_dir` order is not stable, and with
//! several physical volumes under one container the layers come back in a
//! different order each run. Anything derived from that order then changes
//! between runs of the same command on the same machine.
//!
//! **Errors surface.** A layer whose probe fails is reported, never dropped.
//! Silently returning a shorter chain produces a description of the machine
//! that is wrong rather than incomplete, and the caller cannot tell.
//!
//! **Mult-device and non-path sources are refused by name.** A colon-joined
//! bcachefs source and a ZFS dataset are both legal things for the kernel to
//! report, and neither resolves to a single device.

use crate::{Error, MountSource, MountTable};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use superblock::Superblock;

/// Path of sysfs relative to a system root.
pub const SYSFS_DIR: &str = "sys";
/// Path of devfs relative to a system root.
pub const DEVFS_DIR: &str = "dev";

/// Reads sysfs and devfs beneath a given root.
#[derive(Debug, Clone)]
pub struct Probe {
    sysfs: PathBuf,
    devfs: PathBuf,
}

impl Probe {
    /// Probe beneath a system root.
    ///
    /// A root of `/` reads the running machine. ANy other root reads a fixture
    /// tree, which is what makes this testable without privileged access to
    /// real block devices.
    #[must_use]
    pub fn new(sysroot: impl AsRef<Path>) -> Self {
        let sysroot = sysroot.as_ref();
        Self {
            sysfs: sysroot.join(SYSFS_DIR),
            devfs: sysroot.join(DEVFS_DIR),
        }
    }

    /// Probe with sysfs and devfs given separately.
    #[must_use]
    pub fn with_paths(sysfs: impl Into<PathBuf>, devfs: impl Into<PathBuf>) -> Self {
        Self {
            sysfs: sysfs.into(),
            devfs: devfs.into(),
        }
    }

    /// The sysfs directory being read.
    #[must_use]
    pub fn sysfs(&self) -> &Path {
        &self.sysfs
    }

    /// The devfs directory being read.
    #[must_use]
    pub fn devfs(&self) -> &Path {
        &self.devfs
    }

    /// The device backing a mountpoint.
    ///
    /// Refuses a source that is not one device, rather than choosing among
    /// several or treating a dataset name as a path.
    pub fn device_for_mountpoint(&self, mounts: &MountTable, mountpoint: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let mountpoint = mountpoint.as_ref();
        let entry = mounts.for_mountpoint(mountpoint).ok_or_else(|| Error::NotMounted {
            path: mountpoint.to_path_buf(),
        })?;

        match &entry.source {
            MountSource::Single(device) => Ok(device.clone()),
            MountSource::Multi(devices) => Err(Error::UnsupportedSource {
                mountpoint: mountpoint.to_path_buf(),
                description: format!("{} devices", devices.len()),
            }),
            MountSource::Other(source) => Err(Error::UnsupportedSource {
                mountpoint: mountpoint.to_path_buf(),
                description: source.clone(),
            }),
        }
    }

    /// Every device beneath this one, nearest first.
    ///
    /// The device itself is not included. An empty result means the device has
    /// no backing devices, which is the ordinary case for a partition.
    ///
    /// Siblings are visited in sorted order and each is fully descended before
    /// the next, so the same stack always produces the same list.
    pub fn backing_devices(&self, device: impl AsRef<Path>) -> Result<Vec<PathBuf>, Error> {
        let mut found = Vec::new();
        self.collect_backing(device.as_ref(), &mut found)?;
        Ok(found)
    }

    fn collect_backing(&self, device: &Path, found: &mut Vec<PathBuf>) -> Result<(), Error> {
        let slaves = self.block_sysfs_path(device)?.join("slaves");
        if !slaves.is_dir() {
            return Ok(());
        }

        for name in read_dir_sorted(&slaves)? {
            let backing = self.devfs.join(name);

            // Guard against a cycle in a malformed tree rather than recursing
            // until the stack gives out.
            if found.contains(&backing) {
                continue;
            }
            found.push(backing.clone());
            self.collect_backing(&backing, found)?;
        }
        Ok(())
    }

    /// The disk containing a partition, when the device is one.
    ///
    /// `None` for a while disk, or for a device-mapper node, neither of which
    /// has a parent block device in sysfs.
    #[must_use]
    pub fn parent_device(&self, device: impl AsRef<Path>) -> Option<PathBuf> {
        let sysfs_path = self.block_sysfs_path(device.as_ref()).ok()?;
        let resolved = fs::canonicalize(&sysfs_path).unwrap_or(sysfs_path);
        let parent = resolved.parent()?.file_name()?;

        // A partition's sysfs directory lives inside its disk's; a whole disk
        // sits directly under `block`, which is the end of the walk.
        if parent == "block" {
            return None;
        }
        Some(self.devfs.join(parent))
    }

    /// Read and identify the superblock on a device.
    pub fn superblock(&self, device: impl AsRef<Path>) -> Result<Superblock, Error> {
        let device = device.as_ref();
        let mut file = fs::File::open(device).map_err(|source| Error::io(device, source))?;

        Superblock::from_reader(&mut file).map_err(|source| Error::Superblock {
            path: device.to_path_buf(),
            source,
        })
    }

    /// The sysfs directory for a device, by its name.
    ///
    /// Canonicalisation is best effort. On a running machine it resolves
    /// `/dev/mapper/root` to the `dm-0` that sysfs is keyed by; against
    /// a fixture tree there is nothing to resolve and the anme is used
    /// as given.
    fn block_sysfs_path(&self, device: &Path) -> Result<PathBuf, Error> {
        let canonical = fs::canonicalize(device).unwrap_or_else(|_| device.to_path_buf());
        let name = canonical.file_name().ok_or_else(|| Error::Unnamed {
            path: device.to_path_buf(),
        })?;
        Ok(self.sysfs.join("class").join("block").join(name))
    }
}

/// Directory entry names, sorted.
fn read_dir_sorted(dir: &Path) -> Result<Vec<OsString>, Error> {
    let mut names = Vec::new();

    for entry in fs::read_dir(dir).map_err(|source| Error::io(dir, source))? {
        let entry = entry.map_err(|source| Error::io(dir, source))?;
        names.push(entry.file_name());
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs, process,
        time::{SystemTime, UNIX_EPOCH},
    };

    /// A throwaway directory tree, removed when the test ends.
    ///
    /// Hand-rolled rather than pulling in a dependency, since the only thing
    /// needed is a unique path and cleanup on drop.
    struct TempTree {
        root: PathBuf,
    }

    impl TempTree {
        fn new(name: &str) -> Self {
            let unique = format!(
                "topology-test-{name}-{}-{:?}",
                process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|since| since.as_nanos())
                    .unwrap_or_default()
            );
            let root = env::temp_dir().join(unique);
            fs::create_dir_all(&root).expect("create temp tree");
            Self { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        /// Register a block device with an optional set of backing devices.
        fn block(&self, name: &str, backing: &[&str]) -> &Self {
            let device_dir = self.root.join("sys/class/block").join(name);

            fs::create_dir_all(&device_dir).expect("create block dir");
            if !backing.is_empty() {
                let slaves = device_dir.join("slaves");
                fs::create_dir_all(&slaves).expect("create slaves dir");
                for entry in backing {
                    fs::create_dir_all(slaves.join(entry)).expect("create slave entry");
                }
            }
            fs::create_dir_all(self.root.join("dev")).expect("create devfs");
            fs::write(self.root.join("dev").join(name), b"").expect("create device node");
            self
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn a_partition_has_no_backing_devices() {
        let tree = TempTree::new("plain");
        tree.block("nvme0n1p3", &[]);

        let probe = Probe::new(tree.path());
        let backing = probe.backing_devices(tree.path().join("dev/nvme0n1p3")).unwrap();

        assert!(backing.is_empty())
    }

    #[test]
    fn a_stack_is_walked_to_the_bottom() {
        // btrfs on LVM on LUKS on a partition
        let tree = TempTree::new("stack");
        tree.block("dm-2", &["dm-1"])
            .block("dm-1", &["dm-0"])
            .block("dm-0", &["sda2"])
            .block("sda2", &[]);

        let probe = Probe::new(tree.path());
        let backing = probe.backing_devices(tree.path().join("dev/dm-2")).unwrap();
        let names: Vec<&str> = backing
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap())
            .collect();

        assert_eq!(names, vec!["dm-1", "dm-0", "sda2"]);
    }

    #[test]
    fn a_non_dir_slaves_entry_stops_the_walk() {
        let tree = TempTree::new("unreadable");
        tree.block("dm-0", &["dm-1"]);

        // dm-1 is names as a backing device but has no sysfs entry, and its
        // slaves directory is a file rather than a directory
        let broken = tree.path().join("sys/class/block/dm-1");
        fs::create_dir_all(&broken).unwrap();
        fs::write(broken.join("slaves"), b"not a directory").unwrap();

        let probe = Probe::new(tree.path());
        // a file where a directory belongs is not a directory, so the walk
        // ends there rather than pretending the layer is absent
        let backing = probe.backing_devices(tree.path().join("dev/dm-0")).unwrap();
        assert_eq!(backing.len(), 1);
    }

    #[test]
    fn a_cycle_terminates() {
        let tree = TempTree::new("cycle");
        tree.block("dm-0", &["dm-1"]).block("dm-1", &["dm-0"]);

        let probe = Probe::new(tree.path());
        let backing = probe.backing_devices(tree.path().join("dev/dm-0")).unwrap();

        // both are reported once and the walk stops
        assert_eq!(backing.len(), 2);
    }

    #[test]
    fn mountpoint_resolution() {
        let tree = TempTree::new("mounts");
        let probe = Probe::new(tree.path());
        let mounts = MountTable::parse(
            "/dev/nvme0n1p3 / btrfs rw,subvol=/@ 0 0\n\
             /dev/sda1:/dev/sdb1 /srv bcachefs rw 0 0\n\
             rpool/ROOT/os /zfs zfs rw 0 0\n\
             tmpfs /run tmpfs rw 0 0\n",
        );

        assert_eq!(
            probe.device_for_mountpoint(&mounts, "/").unwrap(),
            PathBuf::from("/dev/nvme0n1p3")
        );

        // a multi-device source is refused by name, not resolved to one of them
        let error = probe.device_for_mountpoint(&mounts, "/srv").unwrap_err();
        assert!(matches!(error, Error::UnsupportedSource { .. }));
        assert!(error.to_string().contains("2 devices"));

        // so is a dataset name, and it says which
        let error = probe.device_for_mountpoint(&mounts, "/zfs").unwrap_err();
        assert!(error.to_string().contains("rpool/ROOT/os"));

        // and a pseudo-filesystem
        assert!(matches!(
            probe.device_for_mountpoint(&mounts, "/run"),
            Err(Error::UnsupportedSource { .. })
        ));

        // an unmounted path is distinguishable from an unsupported one
        assert!(matches!(
            probe.device_for_mountpoint(&mounts, "/nowhere"),
            Err(Error::NotMounted { .. })
        ));
    }

    #[test]
    fn errors_name_the_path_that_failed() {
        let tree = TempTree::new("errors");
        let probe = Probe::new(tree.path());
        let missing = tree.path().join("dev/absent");

        // `Superblock` is not `Debug`, so match rather than unwrap_err
        let Err(error) = probe.superblock(&missing) else {
            panic!("reading a missing device should fail");
        };
        assert!(error.to_string().contains("absent"));
    }

    #[test]
    fn siblings_are_ordered_the_same_way_every_time() {
        // several physical volumes under one container
        let tree = TempTree::new("siblings");
        tree.block("dm-0", &["sdd1", "sda1", "sdc1", "sdb1"])
            .block("sda1", &[])
            .block("sdb1", &[])
            .block("sdc1", &[])
            .block("sdd1", &[]);

        let probe = Probe::new(tree.path());
        let first = probe.backing_devices(tree.path().join("dev/dm-0")).unwrap();
        let second = probe.backing_devices(tree.path().join("dev/dm-0")).unwrap();
        assert_eq!(first, second);

        let names: Vec<&str> = first
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["sda1", "sdb1", "sdc1", "sdd1"]);
    }
}
