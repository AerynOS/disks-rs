// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use disks::{BasicDisk, DiskInit};
use log::{debug, error, info};
use std::{
    fs::File,
    io,
    os::fd::{AsFd, AsRawFd},
    path::{Path, PathBuf},
};
use thiserror::Error;

pub use gpt;
use linux_raw_sys::ioctl::BLKPG;
use nix::libc;

/// Errors that can occur during partition operations
#[derive(Error, Debug)]
pub enum Error {
    /// IO operation error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// GPT-specific error
    #[error("GPT error: {0}")]
    Gpt(#[from] gpt::GptError),
}

/// Represents a block device partition for IOCTL operations
#[repr(C)]
struct BlkpgPartition {
    start: i64,
    length: i64,
    pno: i32,
    devname: [u8; 64],
    volname: [u8; 64],
}

/// IOCTL structure for partition operations
#[repr(C)]
struct BlkpgIoctl {
    op: i32,
    flags: i32,
    datalen: i32,
    data: *mut BlkpgPartition,
}

const BLKPG_ADD_PARTITION: i32 = 1;
const BLKPG_DEL_PARTITION: i32 = 2;

/// Adds a new partition to the specified block device
///
/// # Arguments
/// * `fd` - File descriptor for the block device
/// * `partition_number` - Number to assign to the new partition
/// * `start` - Starting offset in bytes
/// * `length` - Length of partition in bytes
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub(crate) fn add_partition<F>(fd: F, partition_number: i32, start: i64, length: i64) -> io::Result<()>
where
    F: AsRawFd,
{
    debug!("Initiating partition addition - Number: {partition_number}, Start: {start}, Length: {length}");
    let mut part = BlkpgPartition {
        start,
        length,
        pno: partition_number,
        devname: [0; 64],
        volname: [0; 64],
    };

    let mut ioctl = BlkpgIoctl {
        op: BLKPG_ADD_PARTITION,
        flags: 0,
        datalen: std::mem::size_of::<BlkpgPartition>() as i32,
        data: &mut part,
    };

    let res = unsafe { libc::ioctl(fd.as_raw_fd(), BLKPG as _, &mut ioctl) };
    if res < 0 {
        let err = io::Error::last_os_error();
        error!("Partition creation failed: {err}");
        return Err(err);
    }
    info!("Successfully created partition {partition_number}");
    Ok(())
}

/// Deletes a partition from the specified block device
///
/// # Arguments
/// * `fd` - File descriptor for the block device
/// * `partition_number` - Number of the partition to delete
///
/// # Returns
/// `io::Result<()>` indicating success or failure
pub(crate) fn delete_partition<F>(fd: F, partition_number: i32) -> io::Result<()>
where
    F: AsRawFd,
{
    info!("Initiating deletion of partition {partition_number}");
    let mut part = BlkpgPartition {
        start: 0,
        length: 0,
        pno: partition_number,
        devname: [0; 64],
        volname: [0; 64],
    };

    let mut ioctl = BlkpgIoctl {
        op: BLKPG_DEL_PARTITION,
        flags: 0,
        datalen: std::mem::size_of::<BlkpgPartition>() as i32,
        data: &mut part,
    };

    let res = unsafe { libc::ioctl(fd.as_raw_fd(), BLKPG as _, &mut ioctl) };
    if res < 0 {
        let err = io::Error::last_os_error();
        error!("Failed to delete partition {partition_number}: {err}");
        return Err(err);
    }
    info!("Successfully removed partition {partition_number}");
    Ok(())
}

/// Removes all kernel partitions for the specified block device
///
/// # Arguments
/// * `path` - Path to the block device
///
/// # Returns
/// `Result<(), Error>` indicating success or partition operation failure
pub fn remove_kernel_partitions<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    debug!("Beginning partition cleanup process for {:?}", path.as_ref());
    let file = File::open(&path)?;

    // Find the disk for enumeration purposes
    let base_name = path
        .as_ref()
        .file_name()
        .ok_or(Error::Io(io::Error::from(io::ErrorKind::InvalidInput)))?
        .to_string_lossy()
        .to_string();
    let disk = BasicDisk::from_sysfs_path(&PathBuf::from("/"), &base_name)
        .ok_or(Error::Io(io::Error::from(io::ErrorKind::InvalidInput)))?;

    for partition in disk.partitions() {
        let _ = delete_partition(file.as_raw_fd(), partition.number as i32);
    }

    info!("Successfully removed all kernel partitions");
    Ok(())
}

/// Creates kernel partitions based on the current GPT table
///
/// # Arguments
/// * `path` - Path to the block device
///
/// # Returns
/// `Result<(), Error>` indicating success or partition operation failure
pub fn create_kernel_partitions<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    info!("Creating kernel partitions from GPT for {:?}", path.as_ref());
    let file = File::open(&path)?;

    // Read GPT table
    debug!("Reading GPT partition table");
    let gpt = gpt::GptConfig::new().writable(false).open(&path)?;
    let partitions = gpt.partitions();
    let block_size = 512;
    info!("Located {} partitions (block size: {})", partitions.len(), block_size);

    // Add partitions from GPT
    debug!("Beginning partition creation from GPT table");
    for (i, partition) in partitions.iter() {
        add_partition(
            file.as_fd(),
            *i as i32,
            partition.first_lba as i64 * block_size,
            (partition.last_lba - partition.first_lba + 1) as i64 * block_size,
        )?;
    }

    info!("GPT partition creation completed successfully");
    Ok(())
}

/// Updates kernel partition representations to match the GPT table
///
/// # Arguments
/// * `path` - Path to the block device
///
/// # Returns
/// `Result<(), Error>` indicating success or partition operation failure
pub fn sync_gpt_partitions<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    info!("Initiating GPT partition synchronization for {:?}", path.as_ref());

    remove_kernel_partitions(&path)?;
    create_kernel_partitions(&path)?;

    info!("GPT partition synchronization completed successfully");
    Ok(())
}
