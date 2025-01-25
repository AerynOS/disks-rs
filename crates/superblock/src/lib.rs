// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Superblock detection and handling for various filesystems
//!
//! This module provides functionality to detect and read superblocks from different
//! filesystem types including Btrfs, Ext4, F2FS, LUKS2, and XFS.

use std::io::{self, BufReader, Cursor, Read, Seek};

use thiserror::Error;
use zerocopy::FromBytes;

pub mod btrfs;
pub mod ext4;
pub mod f2fs;
pub mod luks2;
pub mod xfs;

/// Common interface for superblock detection
pub trait Detection: Sized + FromBytes {
    /// The magic number type for this superblock
    type Magic: FromBytes + PartialEq + Eq;

    /// The offset in bytes where the superblock is located
    const OFFSET: u64;

    /// The offset within the superblock where the magic number is located
    const MAGIC_OFFSET: u64;

    /// The size in bytes of the superblock
    const SIZE: usize;

    /// Check if the magic number is valid for this superblock type
    fn is_valid_magic(magic: &Self::Magic) -> bool;
}

/// Errors that can occur when reading superblocks
#[derive(Debug, Error)]
pub enum Error {
    /// No known filesystem superblock was detected
    #[error("unknown superblock")]
    UnknownSuperblock,

    /// Invalid JSON
    #[error("invalid json")]
    InvalidJson(#[from] serde_json::Error),

    /// The requested feature is not implemented for this filesystem type
    #[error("unsupported feature")]
    UnsupportedFeature,

    /// Error decoding UTF-8 string data
    #[error("invalid utf8 in decode: {0}")]
    Utf8Decoding(#[from] std::str::Utf8Error),

    /// Error decoding UTF-16 string data
    #[error("invalid utf16 in decode: {0}")]
    Utf16Decoding(#[from] std::string::FromUtf16Error),

    /// An I/O error occurred
    #[error("io: {0}")]
    IO(#[from] io::Error),
}

/// Attempts to detect a superblock of the given type from the reader
pub fn detect_superblock<T: Detection, R: Read + Seek>(reader: &mut R) -> Result<Option<T>, Error> {
    let mut reader = BufReader::new(reader);
    reader.seek(io::SeekFrom::Start(T::MAGIC_OFFSET))?;
    let mut magic_buf = vec![0u8; std::mem::size_of::<T::Magic>()];
    reader.read_exact(&mut magic_buf)?;

    match T::Magic::read_from_bytes(&magic_buf) {
        Ok(magic) if T::is_valid_magic(&magic) => {
            reader.seek(io::SeekFrom::Start(T::OFFSET))?;
            let mut block_buf = vec![0u8; T::SIZE];
            reader.read_exact(&mut block_buf)?;
            if let Ok(block) = FromBytes::read_from_bytes(&block_buf) {
                Ok(Some(block))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/// Supported filesystem types that can be detected and read
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Btrfs filesystem
    Btrfs,
    /// Ext4 filesystem
    Ext4,
    /// LUKS2 encrypted container
    LUKS2,
    /// F2FS (Flash-Friendly File System)
    F2FS,
    /// XFS filesystem
    XFS,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Kind::Btrfs => f.write_str("btrfs"),
            Kind::Ext4 => f.write_str("ext4"),
            Kind::LUKS2 => f.write_str("luks2"),
            Kind::F2FS => f.write_str("f2fs"),
            Kind::XFS => f.write_str("xfs"),
        }
    }
}

pub enum Superblock {
    Btrfs(Box<btrfs::Btrfs>),
    Ext4(Box<ext4::Ext4>),
    F2FS(Box<f2fs::F2FS>),
    LUKS2(Box<luks2::Luks2>),
    XFS(Box<xfs::XFS>),
}

impl Superblock {
    /// Returns the filesystem type of this superblock
    pub fn kind(&self) -> Kind {
        match self {
            Superblock::Btrfs(_) => Kind::Btrfs,
            Superblock::Ext4(_) => Kind::Ext4,
            Superblock::F2FS(_) => Kind::F2FS,
            Superblock::LUKS2(_) => Kind::LUKS2,
            Superblock::XFS(_) => Kind::XFS,
        }
    }

    /// Returns the filesystem UUID if available
    pub fn uuid(&self) -> Result<String, Error> {
        match self {
            Superblock::Btrfs(block) => block.uuid(),
            Superblock::Ext4(block) => block.uuid(),
            Superblock::F2FS(block) => block.uuid(),
            Superblock::LUKS2(block) => block.uuid(),
            Superblock::XFS(block) => block.uuid(),
        }
    }

    /// Returns the volume label if available
    pub fn label(&self) -> Result<String, Error> {
        match self {
            Superblock::Btrfs(block) => block.label(),
            Superblock::Ext4(block) => block.label(),
            Superblock::F2FS(block) => block.label(),
            Superblock::LUKS2(block) => block.label(),
            Superblock::XFS(block) => block.label(),
        }
    }
}

impl Superblock {
    /// Attempt to detect and read a filesystem superblock from raw bytes
    ///
    /// This is more efficient than using a reader as it avoids multiple seeks
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut cursor = Cursor::new(bytes);

        // Try each filesystem type in order of likelihood
        if let Some(sb) = detect_superblock::<ext4::Ext4, _>(&mut cursor)? {
            return Ok(Self::Ext4(Box::new(sb)));
        }
        if let Some(sb) = detect_superblock::<btrfs::Btrfs, _>(&mut cursor)? {
            return Ok(Self::Btrfs(Box::new(sb)));
        }
        if let Some(sb) = detect_superblock::<f2fs::F2FS, _>(&mut cursor)? {
            return Ok(Self::F2FS(Box::new(sb)));
        }
        if let Some(sb) = detect_superblock::<xfs::XFS, _>(&mut cursor)? {
            return Ok(Self::XFS(Box::new(sb)));
        }
        if let Some(sb) = detect_superblock::<luks2::Luks2, _>(&mut cursor)? {
            return Ok(Self::LUKS2(Box::new(sb)));
        }

        Err(Error::UnknownSuperblock)
    }

    /// Attempt to detect and read a filesystem superblock from a reader
    ///
    /// Note: This will read the minimum necessary bytes to detect the superblock,
    /// which is more efficient than reading the entire device.
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, Error> {
        // Preallocate a fixed buffer for the largest superblock we need to read
        let mut bytes = vec![0u8; 128 * 1024]; // 128KB covers all superblock offsets
        reader.rewind()?;
        reader.read_exact(&mut bytes)?;

        Self::from_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Cursor, Read},
    };

    use crate::Kind;

    use super::Superblock;

    #[test_log::test]
    fn test_determination() {
        let tests = vec![
            (
                "btrfs",
                Kind::Btrfs,
                "blsforme testing",
                "829d6a03-96a5-4749-9ea2-dbb6e59368b2",
            ),
            (
                "ext4",
                Kind::Ext4,
                "blsforme testing",
                "731af94c-9990-4eed-944d-5d230dbe8a0d",
            ),
            (
                "f2fs",
                Kind::F2FS,
                "blsforme testing",
                "d2c85810-4e75-4274-bc7d-a78267af7443",
            ),
            ("luks+ext4", Kind::LUKS2, "", "be373cae-2bd1-4ad5-953f-3463b2e53e59"),
            ("xfs", Kind::XFS, "BLSFORME", "45e8a3bf-8114-400f-95b0-380d0fb7d42d"),
        ];

        // Pre-allocate a buffer for determination tests
        let mut memory: Vec<u8> = Vec::with_capacity(512 * 1024);

        for (fsname, kind, label, uuid) in tests.into_iter() {
            // Swings and roundabouts: Unpack ztd image in memory to get the Seekable trait we need
            // While each Superblock API is non-seekable, we enforce superblock::for_reader to be seekable
            // to make sure we pre-read a blob and pass it in for rewind/speed.
            memory.clear();

            let mut fi = fs::File::open(format!("tests/{fsname}.img.zst")).expect("Cannot find test image");
            let mut stream = zstd::stream::Decoder::new(&mut fi).expect("Unable to decode stream");
            stream
                .read_to_end(&mut memory)
                .expect("Could not unpack filesystem in memory");

            let mut cursor = Cursor::new(&mut memory);
            let block = Superblock::from_reader(&mut cursor).expect("Failed to find right block implementation");
            eprintln!("{fsname}.img.zstd: superblock matched to {}", block.kind());
            assert_eq!(block.kind(), kind);
            assert_eq!(block.label().unwrap(), label);
            assert_eq!(block.uuid().unwrap(), uuid);

            // Is it possible to get the JSON config out of LUKS2?
            if let Superblock::LUKS2(block) = block {
                let config = block.read_config(&mut cursor).expect("Cannot read LUKS2 config");
                eprintln!("{}", serde_json::to_string_pretty(&config).unwrap());
                assert!(config.config.json_size > 0);
                assert!(config.config.keyslots_size > 0);

                let keyslot = config.keyslots.get(&0).unwrap();
                assert_eq!(keyslot.area.encryption, "aes-xts-plain64");
            }
        }
    }
}
