// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, str::FromStr};

use crate::kdl_value_to_string;

use super::FromKdlProperty;

/// The role assigned to a partition
#[derive(Debug, PartialEq)]
pub enum PartitionRole {
    /// Boot partition (usually ESP)
    Boot,

    /// Extended boot partition (e.g. XBOOTLDR)
    ExtendedBoot,

    /// Root filesystem
    Root,

    /// Home directory mount
    Home,

    /// Swap partition
    Swap,
}

impl fmt::Display for PartitionRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boot => f.write_str("boot"),
            Self::ExtendedBoot => f.write_str("extended-boot"),
            Self::Root => f.write_str("root"),
            Self::Home => f.write_str("home"),
            Self::Swap => f.write_str("swap"),
        }
    }
}

impl FromStr for PartitionRole {
    type Err = crate::Error;

    /// Attempt to convert a string to a partition role
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "boot" => Ok(Self::Boot),
            "extended-boot" => Ok(Self::ExtendedBoot),
            "root" => Ok(Self::Root),
            "home" => Ok(Self::Home),
            "swap" => Ok(Self::Swap),
            _ => Err(crate::Error::UnknownVariant),
        }
    }
}

impl FromKdlProperty<'_> for PartitionRole {
    fn from_kdl_property(entry: &kdl::KdlEntry) -> Result<Self, crate::Error> {
        let value = kdl_value_to_string(entry)?;
        let v = value.parse().map_err(|_| crate::UnsupportedValue {
            at: entry.span(),
            advice: Some("'boot', 'extended-boot', 'root', 'home' and 'swap' are supported".into()),
        })?;
        Ok(v)
    }
}
