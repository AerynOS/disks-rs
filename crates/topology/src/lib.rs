// SPDX-FileCopyrightText: Copyright © 2026 aerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Resolves the device topology of a system that already exists.
//!
//! The rest of disks-rs is built to *create* a disk layout.
//! This crate reads on back: which mount point sists on which device,
//! through which layers, and what each of those layers is.
//!
//! Facts out, decisions somewhere else. Nothing here interprets
//! what it finds, and the crate is unable to produce kernel
//! cmdline.

mod mounts;

// Re-exports
pub mod mounts::*;
