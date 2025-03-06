// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::fs;

use disks::BlockDevice;
use gpt::{mbr, partition_types, GptConfig};
use thiserror::Error;

use crate::planner::{Change, Planner};

/// Errors that can occur when writing changes to disk
#[derive(Debug, Error)]
pub enum WriteError {
    /// Device size has changed since the plan was created
    #[error("Device size changed since planning")]
    DeviceSizeChanged,

    /// A partition ID was used multiple times
    #[error("Duplicate partition ID: {0}")]
    DuplicatePartitionId(u32),

    /// Error from GPT library
    #[error("GPT error: {0}")]
    Gpt(#[from] gpt::GptError),

    /// Error from MBR handling
    #[error("GPT error: {0}")]
    Mbr(#[from] gpt::mbr::MBRError),

    /// Underlying I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A writer that applies the layouts from the Planner to the disk.
pub struct DiskWriter<'a> {
    /// The block device to write to
    pub device: &'a BlockDevice,
    /// The planner containing the changes to apply
    pub planner: &'a Planner,
}

impl<'a> DiskWriter<'a> {
    /// Create a new DiskWriter.
    pub fn new(device: &'a BlockDevice, planner: &'a Planner) -> Self {
        Self { device, planner }
    }

    /// Simulate changes without writing to disk
    pub fn simulate(&self) -> Result<(), WriteError> {
        let mut device = fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(self.device.device())?;
        self.validate_changes(&device)?;
        self.apply_changes(&mut device, false)?;
        Ok(())
    }

    /// Actually write changes to disk
    pub fn write(&self) -> Result<(), WriteError> {
        let mut device = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.device.device())?;

        self.validate_changes(&device)?;
        self.apply_changes(&mut device, true)?;
        Ok(())
    }

    /// Validate all planned changes before applying them by checking:
    /// - Device size matches the planned size
    /// - No duplicate partition IDs exist
    fn validate_changes(&self, device: &fs::File) -> Result<(), WriteError> {
        // Verify device size matches what we planned for
        let metadata = device.metadata()?;
        if metadata.len() != self.device.size() {
            return Err(WriteError::DeviceSizeChanged);
        }

        // Verify partition IDs don't conflict
        let mut used_ids = std::collections::HashSet::new();
        for change in self.planner.changes() {
            match change {
                Change::AddPartition { partition_id, .. } => {
                    if !used_ids.insert(*partition_id) {
                        return Err(WriteError::DuplicatePartitionId(*partition_id));
                    }
                }
                Change::DeletePartition { partition_id, .. } => {
                    used_ids.remove(partition_id);
                }
            }
        }

        Ok(())
    }

    /// Apply the changes to disk by:
    /// - Creating or opening the GPT table
    /// - Applying each change in sequence
    fn apply_changes(&self, device: &mut fs::File, writable: bool) -> Result<(), WriteError> {
        let mut gpt_table = if self.planner.wipe_disk() {
            if writable {
                let mbr = mbr::ProtectiveMBR::with_lb_size(
                    u32::try_from((self.device.size() / 512) - 1).unwrap_or(0xFF_FF_FF_FF),
                );
                mbr.overwrite_lba0(device)?;
            }

            GptConfig::default()
                .writable(writable)
                .logical_block_size(gpt::disk::LogicalBlockSize::Lb512)
                .create_from_device(device, None)?
        } else {
            GptConfig::default().writable(writable).open_from_device(device)?
        };

        let _layout = self.planner.current_layout();
        let changes = self.planner.changes();

        for change in changes {
            match change {
                Change::DeletePartition {
                    partition_id,
                    original_index,
                } => {
                    if let Some(id) = gpt_table.remove_partition(*partition_id) {
                        println!(
                            "Deleted partition {} (index {}): {:?}",
                            partition_id, original_index, id
                        );
                    }
                }
                Change::AddPartition {
                    start,
                    end,
                    partition_id,
                } => {
                    let start_lba = *start / 512;
                    let size_lba = (*end - *start) / 512;
                    let part_type = partition_types::BASIC;

                    let id = gpt_table.add_partition_at("", *partition_id, start_lba, size_lba, part_type, 0)?;
                    println!("Added partition {}: {:?}", partition_id, id);
                }
            }
        }

        eprintln!("GPT is now: {gpt_table:?}");

        Ok(())
    }
}
