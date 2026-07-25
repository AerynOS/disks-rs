// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{DEVFS_DIR, SECTOR_SIZE, SYSFS_DIR, sysfs};
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// Represents a partition on a disk device
/// - Size in sectors
#[derive(Debug, Default)]
pub struct Partition {
    /// Name of the partition
    pub name: String,
    /// Partition number on the disk
    pub number: u32,
    /// Starting sector of the partition
    pub start: u64,
    /// Ending sector of the partition
    pub end: u64,
    /// Size of partition in sectors
    pub size: u64,
    /// Path to the partition node in sysfs
    pub node: PathBuf,
    /// Path to the partition device in /dev
    pub device: PathBuf,
}

impl fmt::Display for Partition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{name} {size:.2} GiB",
            name = self.name,
            size = self.size_bytes() as f64 / (1024.0 * 1024.0 * 1024.0)
        )
    }
}

impl Partition {
    /// Creates a new Partition instance from a sysfs path and partition name.
    ///
    /// # Arguments
    /// * `sysroot` - Base path to sysfs
    /// * `name` - Name of the partition
    ///
    /// # Returns
    /// * `Some(Partition)` if partition exists and is valid
    /// * `None` if partition doesn't exist or is invalid
    pub fn from_sysfs_path(sysroot: &Path, name: &str) -> Option<Self> {
        let node = sysroot.join(SYSFS_DIR).join(name);
        let partition_no: u32 = sysfs::read(&node, "partition")?;
        let start = sysfs::read(&node, "start")?;
        let size = sysfs::read(&node, "size")?;
        Some(Self {
            name: name.to_owned(),
            number: partition_no,
            start,
            size,
            end: start + size,
            node,
            device: sysroot.join(DEVFS_DIR).join(name),
        })
    }

    /// Absolute start offset of the partition, in bytes
    pub fn start_bytes(&self) -> u64 {
        self.start * SECTOR_SIZE
    }

    /// Absolute end offset of the partition, in bytes
    pub fn end_bytes(&self) -> u64 {
        self.end * SECTOR_SIZE
    }

    /// Size of the partition, in bytes
    pub fn size_bytes(&self) -> u64 {
        self.size * SECTOR_SIZE
    }
}
