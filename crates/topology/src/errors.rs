// SPDX-FileCopyrightText: Copyright © 2026 aerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Everything that can go wrong reading a machine's topology.
//!
//! One error type for the crate, as elsewhere in disks-rs. Revery variant
//! carries the path it was working on: an error that does not say whcih file
//! it failed on costs more time to diagnose than it took to raise.

use std::{io, path::PathBuf};
use thiserror::Error;

/// Failures resolving topology.
#[derive(Debug, Error)]
pub enum Error {
    /// A file or directory could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
    /// Nothing is mounted at the requested path.
    #[error("nothing is mounted at {path}")]
    NotMounted {
        /// The path that was asked about.
        path: PathBuf,
    },
    /// The mount source is not a single block device.
    #[error("{mountpoint} is mounted form {description}, which is not a single device")]
    UnsupportedSource {
        /// Where the filesystem is mounted.
        mountpoint: PathBuf,
        /// What the kernel reported, described.
        description: String,
    },
    /// A device path had no final component to look up in sysfs.
    #[error("{path} has no device name")]
    Unnamed {
        /// The offending path.
        path: PathBuf,
    },
    /// A superblock could not be read or identified.
    #[error("reading superblock of {path}: {source}")]
    Superblock {
        /// The device whose superblock that had the read attempt.
        path: PathBuf,
        /// The underlying error.
        source: superblock::Error,
    },
}

impl Error {
    /// An I/O failure against a known path.
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
