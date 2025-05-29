// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! F2FS (Flash-Friendly File System) superblock handling
//!
//! This module implements parsing and access to the F2FS filesystem superblock,
//! which contains critical metadata about the filesystem including:
//! - Version information
//! - Layout parameters (sector size, block size, segment size etc)
//! - Block addresses for key filesystem structures
//! - Volume name and UUID
//! - Encryption settings
//! - Device information

use crate::{Detection, UnicodeError};
use uuid::Uuid;
use zerocopy::*;

/// Maximum length of volume name
pub const MAX_VOLUME_LEN: usize = 512;
/// Maximum number of supported extensions
pub const MAX_EXTENSION: usize = 64;
/// Length of each extension entry
pub const EXTENSION_LEN: usize = 8;
/// Length of version string
pub const VERSION_LEN: usize = 256;
/// Maximum number of devices in array
pub const MAX_DEVICES: usize = 8;
/// Maximum number of quota types
pub const MAX_QUOTAS: usize = 3;
/// Length of stop reason string
pub const MAX_STOP_REASON: usize = 32;
/// Maximum number of recorded errors
pub const MAX_ERRORS: usize = 16;

/// Represents the F2FS superblock structure that exists on disk
#[derive(Debug, FromBytes, Unaligned)]
#[repr(C, packed)]
pub struct F2FS {
    /// Magic number to identify F2FS filesystem
    pub magic: U32<LittleEndian>,
    /// Major version of filesystem
    pub major_ver: U16<LittleEndian>,
    /// Minor version of filesystem
    pub minor_ver: U16<LittleEndian>,
    /// Log2 of sector size in bytes
    pub log_sectorsize: U32<LittleEndian>,
    /// Log2 of sectors per block
    pub log_sectors_per_block: U32<LittleEndian>,
    /// Log2 of block size in bytes
    pub log_blocksize: U32<LittleEndian>,
    /// Log2 of blocks per segment
    pub log_blocks_per_seg: U32<LittleEndian>,
    /// Number of segments per section
    pub segs_per_sec: U32<LittleEndian>,
    /// Number of sections per zone
    pub secs_per_zone: U32<LittleEndian>,
    /// Checksum offset within superblock
    pub checksum_offset: U32<LittleEndian>,
    /// Total block count
    pub block_count: U64<LittleEndian>,
    /// Total section count
    pub section_count: U32<LittleEndian>,
    /// Total segment count
    pub segment_count: U32<LittleEndian>,
    /// Number of segments for checkpoint
    pub segment_count_ckpt: U32<LittleEndian>,
    /// Number of segments for SIT
    pub segment_count_sit: U32<LittleEndian>,
    /// Number of segments for NAT
    pub segment_count_nat: U32<LittleEndian>,
    /// Number of segments for SSA
    pub segment_count_ssa: U32<LittleEndian>,
    /// Number of segments for main area
    pub segment_count_main: U32<LittleEndian>,
    /// First segment block address
    pub segment0_blkaddr: U32<LittleEndian>,
    /// Checkpoint block address
    pub cp_blkaddr: U32<LittleEndian>,
    /// SIT block address
    pub sit_blkaddr: U32<LittleEndian>,
    /// NAT block address
    pub nat_blkaddr: U32<LittleEndian>,
    /// SSA block address
    pub ssa_blkaddr: U32<LittleEndian>,
    /// Main area block address
    pub main_blkaddr: U32<LittleEndian>,
    /// Root inode number
    pub root_ino: U32<LittleEndian>,
    /// Node inode number
    pub node_ino: U32<LittleEndian>,
    /// Meta inode number
    pub meta_ino: U32<LittleEndian>,
    /// Filesystem UUID
    pub uuid: [u8; 16],
    /// Volume name in UTF-16
    pub volume_name: [U16<LittleEndian>; MAX_VOLUME_LEN],
    /// Number of supported extensions
    pub extension_count: U32<LittleEndian>,
    /// List of supported extensions
    pub extension_list: [[u8; EXTENSION_LEN]; MAX_EXTENSION],
    /// Checkpoint payload
    pub cp_payload: U32<LittleEndian>,
    /// Filesystem version string
    pub version: [u8; VERSION_LEN],
    /// Initial filesystem version
    pub init_version: [u8; VERSION_LEN],
    /// Feature flags
    pub feature: U32<LittleEndian>,
    /// Encryption level
    pub encryption_level: u8,
    /// Encryption password salt
    pub encryption_pw_salt: [u8; 16],
    /// Array of attached devices
    pub devs: [Device; MAX_DEVICES],
    /// Quota file inode numbers
    pub qf_ino: [U32<LittleEndian>; MAX_QUOTAS],
    /// Number of hot extensions
    pub hot_ext_count: u8,
    /// Character encoding
    pub s_encoding: U16<LittleEndian>,
    /// Encoding flags
    pub s_encoding_flags: U16<LittleEndian>,
    /// Filesystem stop reason
    pub s_stop_reason: [u8; MAX_STOP_REASON],
    /// Recent errors
    pub s_errors: [u8; MAX_ERRORS],
    /// Reserved space
    pub reserved: [u8; 258],
    /// Superblock checksum
    pub crc: U32<LittleEndian>,
}

/// Represents a device entry in the F2FS superblock
#[derive(Debug, Clone, Copy, FromBytes)]
#[repr(C, packed)]
pub struct Device {
    /// Device path
    pub path: [u8; 64],
    /// Total number of segments on device
    pub total_segments: U32<LittleEndian>,
}

impl Detection for F2FS {
    type Magic = U32<LittleEndian>;

    const OFFSET: u64 = START_POSITION;

    const MAGIC_OFFSET: u64 = START_POSITION;

    const SIZE: usize = std::mem::size_of::<F2FS>();

    fn is_valid_magic(magic: &Self::Magic) -> bool {
        *magic == MAGIC
    }
}

/// F2FS superblock magic number for validation
pub const MAGIC: U32<LittleEndian> = U32::new(0xF2F52010);
/// Starting position of superblock in bytes
pub const START_POSITION: u64 = 1024;

impl F2FS {
    /// Returns the filesystem UUID as a hyphenated string
    pub fn uuid(&self) -> Result<String, UnicodeError> {
        Ok(Uuid::from_bytes(self.uuid).hyphenated().to_string())
    }

    /// Returns the volume label as a UTF-16 decoded string
    ///
    /// Handles null termination and invalid UTF-16 sequences
    pub fn label(&self) -> Result<String, UnicodeError> {
        // Convert the array of U16<LittleEndian> to u16
        let vol = self.volume_name.map(|x| x.get());
        let prelim_label = String::from_utf16(&vol)?;
        // Need valid grapheme step and skip (u16)\0 nul termination in fixed block size
        Ok(prelim_label.trim_end_matches('\0').to_owned())
    }
}
