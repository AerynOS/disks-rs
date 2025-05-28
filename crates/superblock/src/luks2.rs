// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! # LUKS2 superblock support
//!
//! This module provides functionality for reading and parsing LUKS2 (Linux Unified Key Setup 2)
//! superblocks and their associated metadata.
//!
//! LUKS2 is a disk encryption format that uses the dm-crypt subsystem. It stores metadata
//! like encryption parameters, key slots and segment information in JSON format.
//!

use std::io;

use snafu::Snafu;

mod config;
mod superblock;

pub use config::*;
pub use superblock::*;

/// Errors that can occur when parsing LUKS config
#[derive(Debug, Snafu)]
pub enum ConfigError {
    /// An I/O error occurred
    #[snafu(display("io"))]
    Io { source: io::Error },

    /// Invalid JSON
    #[snafu(display("invalid json"))]
    InvalidJson { source: serde_json::Error },

    /// Error decoding UTF-8 string data
    #[snafu(display("invalid utf8 in decode"))]
    InvalidUtf8 { source: std::str::Utf8Error },
}
