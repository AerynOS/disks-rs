// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Fat32
//!
//! This module implements parsing and access to the FAT32 filesystem boot sector,
//! which contains critical metadata about the filesystem including:
//! - Version information
//! - Volume name and UUID
//! - Encryption settings

use crate::{Detection, Error};
use zerocopy::*;

/// Starting position of superblock in bytes
pub const START_POSITION: u64 = 0;

const MAGIC: [u8; 2] = [0x55, 0xAA];

#[repr(C, packed)]
#[derive(FromBytes, Unaligned, Debug)]
pub struct Fat {
    /// Boot strap short or near jump
    pub ignored: [u8; 3],
    /// Name - can be used to special case partition manager volumes
    pub system_id: [u8; 8],
    /// Bytes per logical sector
    pub sector_size: U16<LittleEndian>,
    /// Sectors per cluster
    pub sec_per_clus: u8,
    /// Reserved sectors
    pub _reserved: U16<LittleEndian>,
    /// Number of FATs
    pub fats: u8,
    /// Root directory entries
    pub dir_entries: U16<LittleEndian>,
    /// Number of sectors
    pub sectors: U16<LittleEndian>,
    /// Media code
    pub media: u8,
    /// Sectors/FAT
    pub fat_length: U16<LittleEndian>,
    /// Sectors per track
    pub secs_track: U16<LittleEndian>,
    /// Number of heads
    pub heads: U16<LittleEndian>,
    /// Hidden sectors (unused)
    pub hidden: U32<LittleEndian>,
    /// Number of sectors (if sectors == 0)
    pub total_sect: U32<LittleEndian>,

    // Shared memory region for FAT16 and FAT32
    // Best way is to use a union with zerocopy, however that requires having to use `--cfg zerocopy_derive_union_into_bytes` https://github.com/google/zerocopy/issues/1792
    pub shared: [u8; 54], // The size of the union fields in bytes
}

#[derive(FromBytes, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Fat16And32Fields {
    // Physical drive number
    pub drive_number: u8,
    // Mount state
    pub state: u8,
    // Extended boot signature
    pub signature: u8,
    // Volume ID
    pub vol_id: U32<LittleEndian>,
    // Volume label
    pub vol_label: [u8; 11],
    // File system type
    pub fs_type: [u8; 8],
}

#[derive(FromBytes, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Fat16Fields {
    pub common: Fat16And32Fields,
}

#[derive(FromBytes, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Fat32Fields {
    // FAT32-specific fields
    /// Sectors/FAT
    pub fat32_length: U32<LittleEndian>,
    /// FAT mirroring flags
    pub fat32_flags: U16<LittleEndian>,
    /// Major, minor filesystem version
    pub fat32_version: [u8; 2],
    /// First cluster in root directory
    pub root_cluster: U32<LittleEndian>,
    /// Filesystem info sector
    pub info_sector: U16<LittleEndian>,
    /// Backup boot sector
    pub backup_boot: U16<LittleEndian>,
    /// Unused
    pub reserved2: [U16<LittleEndian>; 6],

    pub common: Fat16And32Fields,
}

impl Detection for Fat {
    type Magic = [u8; 2];

    const OFFSET: u64 = START_POSITION;

    const MAGIC_OFFSET: u64 = 0x1FE;

    const SIZE: usize = std::mem::size_of::<Fat>();

    fn is_valid_magic(magic: &Self::Magic) -> bool {
        *magic == MAGIC
    }
}

pub enum FatType {
    Fat16,
    Fat32,
}

impl Fat {
    pub fn fat_type(&self) -> Result<FatType, Error> {
        // this is how the linux kernel does it in https://github.com/torvalds/linux/blob/master/fs/fat/inode.c
        if self.fat_length == 0 && self.fat32().fat32_length != 0 {
            Ok(FatType::Fat32)
        } else {
            Ok(FatType::Fat16)
        }
    }

    /// Returns the filesystem id
    pub fn uuid(&self) -> Result<String, Error> {
        match self.fat_type()? {
            FatType::Fat16 => vol_id(self.fat16().common.vol_id),
            FatType::Fat32 => vol_id(self.fat32().common.vol_id),
        }
    }

    /// Returns the volume label
    pub fn label(&self) -> Result<String, Error> {
        match self.fat_type()? {
            FatType::Fat16 => vol_label(&self.fat16().common.vol_label),
            FatType::Fat32 => vol_label(&self.fat32().common.vol_label),
        }
    }

    fn fat16(&self) -> &Fat16Fields {
        let bytes: &[u8; size_of::<Fat16Fields>()] = first_n_bytes(&self.shared);
        transmute_ref!(bytes)
    }

    fn fat32(&self) -> &Fat32Fields {
        transmute_ref!(&self.shared)
    }
}

fn first_n_bytes<const N: usize, const M: usize>(arr: &[u8; M]) -> &[u8; N] {
    const {
        assert!(M >= N, "input array must be at least as large as output array");
    }

    <&[u8; N]>::try_from(&arr[..N]).unwrap()
}

fn vol_label(vol_label: &[u8; 11]) -> Result<String, Error> {
    Ok(String::from_utf8_lossy(vol_label).trim_end_matches(' ').to_string())
}

fn vol_id(vol_id: U32<LittleEndian>) -> Result<String, Error> {
    Ok(format!("{:04X}-{:04X}", vol_id >> 16, vol_id & 0xFFFF))
}
