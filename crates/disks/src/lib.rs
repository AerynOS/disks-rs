// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

mod disk;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub use disk::*;
use partition::Partition;
pub mod loopback;
pub mod mmc;
pub mod mock;
pub mod nvme;
pub mod partition;
pub mod scsi;
mod sysfs;
pub mod virt;

const SYSFS_DIR: &str = "sys/class/block";
const DEVFS_DIR: &str = "dev";

/// A block device on the system which can be either a physical disk or a partition.
#[derive(Debug)]
pub enum BlockDevice {
    /// A physical disk device
    Disk(Box<Disk>),
    Loopback(Box<loopback::Device>),
}

impl BlockDevice {
    /// Discovers all block devices present in the system.
    ///
    /// # Returns
    ///
    /// A vector of discovered block devices or an IO error if the discovery fails.
    pub fn discover() -> io::Result<Vec<BlockDevice>> {
        Self::discover_in_sysroot("/")
    }

    /// Returns the total number of sectors on the block device.
    pub fn sectors(&self) -> u64 {
        match self {
            BlockDevice::Disk(disk) => disk.sectors(),
            BlockDevice::Loopback(device) => device.disk().map_or(0, |d| d.sectors()),
        }
    }

    /// Returns the total size of the block device in bytes.
    pub fn size(&self) -> u64 {
        self.sectors() * 512
    }

    /// Returns the partitions on the block device.
    pub fn partitions(&self) -> &[Partition] {
        match self {
            BlockDevice::Disk(disk) => disk.partitions(),
            BlockDevice::Loopback(device) => device.disk().map_or(&[], |d| d.partitions()),
        }
    }

    /// Returns the path to the partition with the given index.
    /// No attempt is made to verify the existence of the partition.
    pub fn partition_path(&self, index: usize) -> PathBuf {
        if let BlockDevice::Disk(disk) = self {
            match **disk {
                Disk::Scsi(_) | Disk::Virtual(_) => {
                    // Add N to the end of the device name
                    return disk.device_path().join(format!("{}{}", disk.name(), index));
                }
                Disk::Mock(ref d) if d.parts_prefix => {
                    return PathBuf::from("/dev").join(format!("{}p{}", disk.name(), index));
                }
                Disk::Mock(_) => {
                    return PathBuf::from("/dev").join(format!("{}{}", disk.name(), index));
                }
                _ => {}
            }
        }
        // Add pN to the end of the device name
        self.device().with_file_name(format!("{}p{}", self.name(), index))
    }

    /// Creates a mock block device with a specified number of sectors.
    pub fn mock_device(disk: mock::MockDisk) -> Self {
        BlockDevice::Disk(Box::new(Disk::Mock(disk)))
    }

    /// Creates a loopback block device from a file path.
    pub fn loopback_device(device: loopback::Device) -> Self {
        BlockDevice::Loopback(Box::new(device))
    }

    /// Creates a BlockDevice from a specific device path
    ///
    /// # Arguments
    ///
    /// * `device_path` - Path to the block device (e.g. "/dev/sda")
    ///
    /// # Returns
    ///
    /// The block device or an IO error if creation fails.
    pub fn from_sysfs_path(sysfs_root: impl AsRef<Path>, name: impl AsRef<str>) -> io::Result<BlockDevice> {
        let name = name.as_ref();
        let sysfs_dir = sysfs_root.as_ref();

        if let Some(disk) = scsi::Disk::from_sysfs_path(sysfs_dir, name) {
            return Ok(BlockDevice::Disk(Box::new(Disk::Scsi(disk))));
        } else if let Some(disk) = nvme::Disk::from_sysfs_path(sysfs_dir, name) {
            return Ok(BlockDevice::Disk(Box::new(Disk::Nvme(disk))));
        } else if let Some(disk) = mmc::Disk::from_sysfs_path(sysfs_dir, name) {
            return Ok(BlockDevice::Disk(Box::new(Disk::Mmc(disk))));
        } else if let Some(device) = virt::Disk::from_sysfs_path(sysfs_dir, name) {
            return Ok(BlockDevice::Disk(Box::new(Disk::Virtual(device))));
        } else if let Some(device) = loopback::Device::from_sysfs_path(sysfs_dir, name) {
            return Ok(BlockDevice::Loopback(Box::new(device)));
        }

        Err(io::Error::new(io::ErrorKind::NotFound, "Device not found"))
    }

    /// Returns the name of the block device.
    pub fn name(&self) -> &str {
        match self {
            BlockDevice::Disk(disk) => disk.name(),
            BlockDevice::Loopback(device) => device.name(),
        }
    }

    /// Returns the path to the block device in /dev.
    pub fn device(&self) -> &Path {
        match self {
            BlockDevice::Disk(disk) => disk.device_path(),
            BlockDevice::Loopback(device) => device.device_path(),
        }
    }

    /// Discovers block devices in a specified sysroot directory.
    ///
    /// # Arguments
    ///
    /// * `sysroot` - Path to the system root directory
    ///
    /// # Returns
    ///
    /// A vector of discovered block devices or an IO error if the discovery fails.
    pub fn discover_in_sysroot(sysroot: impl AsRef<str>) -> io::Result<Vec<BlockDevice>> {
        let sysroot = sysroot.as_ref();
        let sysfs_dir = PathBuf::from(sysroot).join(SYSFS_DIR);
        let mut devices = Vec::new();

        // Iterate over all block devices in sysfs and collect their filenames
        let mut entries = fs::read_dir(&sysfs_dir)?
            .filter_map(Result::ok)
            .filter_map(|e| Some(e.file_name().to_str()?.to_owned()))
            .collect::<Vec<_>>();
        entries.sort();

        // For all the discovered block devices, try to create a Disk instance
        // At this point we completely ignore partitions. They come later.
        for entry in entries {
            if let Ok(device) = BlockDevice::from_sysfs_path(sysroot, &entry) {
                devices.push(device);
            }
        }

        Ok(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover() {
        let devices = BlockDevice::discover().unwrap();
        for device in &devices {
            match device {
                BlockDevice::Disk(disk) => {
                    println!("{}: {disk}", disk.name());
                    for partition in disk.partitions() {
                        println!("├─{} {partition}", partition.name);
                    }
                }
                BlockDevice::Loopback(device) => {
                    if let Some(file) = device.file_path() {
                        if let Some(disk) = device.disk() {
                            println!("Loopback device: {} (backing file: {})", device.name(), file.display());
                            println!("└─Disk: {} ({})", disk.name(), disk.model().unwrap_or("Unknown"));
                            for partition in disk.partitions() {
                                println!("  ├─{} {partition}", partition.name);
                            }
                        } else {
                            println!("Loopback device: {} (backing file: {})", device.name(), file.display());
                        }
                    } else {
                        println!("Loopback device: {}", device.name());
                    }
                }
            }
        }
    }

    #[test]
    fn test_partition_paths() {
        // Create a mock SCSI disk
        let scsi_disk = mock::MockDisk::new_with_name("sda", 1000, false);
        let device = BlockDevice::mock_device(scsi_disk);
        assert_eq!(device.partition_path(1).to_str().unwrap(), "/dev/sda1");
        assert_eq!(device.partition_path(2).to_str().unwrap(), "/dev/sda2");

        // Create a mock NVMe disk
        let nvme_disk = mock::MockDisk::new_with_name("nvme0n1", 1000, true);
        let device = BlockDevice::mock_device(nvme_disk);
        assert_eq!(device.partition_path(1).to_str().unwrap(), "/dev/nvme0n1p1");
        assert_eq!(device.partition_path(2).to_str().unwrap(), "/dev/nvme0n1p2");
    }
}
