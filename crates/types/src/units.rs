// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, str::FromStr};

#[cfg(feature = "kdl")]
use crate::UnsupportedValue;

#[cfg(feature = "kdl")]
use super::FromKdlType;

/// Storage unit
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum StorageUnit {
    /// Bytes
    #[default]
    Bytes = 1,

    // as 1000s,
    /// Kilobytes
    Kilobytes = 1000,
    /// Megabytes
    Megabytes = 1_000_000,
    /// Gigabytes
    Gigabytes = 1_000_000_000,
    /// Terabytes
    Terabytes = 1_000_000_000_000,

    // as 1024s,
    /// Kibibytes
    Kibibytes = 1024,
    /// Mebibytes
    Mebibytes = 1024 * 1024,
    /// Gibibytes
    Gibibytes = 1024 * 1024 * 1024,
    /// Tebibytes
    Tebibytes = 1024 * 1024 * 1024 * 1024,
}

impl fmt::Display for StorageUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageUnit::Bytes => f.write_str("bytes"),
            StorageUnit::Kilobytes => f.write_str("kilobytes"),
            StorageUnit::Megabytes => f.write_str("megabytes"),
            StorageUnit::Gigabytes => f.write_str("gigabytes"),
            StorageUnit::Terabytes => f.write_str("terabytes"),
            StorageUnit::Kibibytes => f.write_str("kibibytes"),
            StorageUnit::Mebibytes => f.write_str("mebibytes"),
            StorageUnit::Gibibytes => f.write_str("gibibytes"),
            StorageUnit::Tebibytes => f.write_str("tebibytes"),
        }
    }
}

impl FromStr for StorageUnit {
    type Err = crate::Error;

    /// Attempt to convert a string to a storage unit
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "b" => Ok(Self::Bytes),
            "kb" => Ok(Self::Kilobytes),
            "mb" => Ok(Self::Megabytes),
            "gb" => Ok(Self::Gigabytes),
            "tb" => Ok(Self::Terabytes),
            "kib" => Ok(Self::Kibibytes),
            "mib" => Ok(Self::Mebibytes),
            "gib" => Ok(Self::Gibibytes),
            "tib" => Ok(Self::Tebibytes),
            _ => Err(crate::Error::UnknownVariant),
        }
    }
}

#[cfg(feature = "kdl")]
impl FromKdlType<'_> for StorageUnit {
    fn from_kdl_type(id: &kdl::KdlEntry) -> Result<Self, crate::Error> {
        let ty_id = if let Some(ty) = id.ty() {
            ty.value().to_lowercase()
        } else {
            "b".into()
        };

        let v = ty_id.parse().map_err(|_| UnsupportedValue {
            at: id.span(),
            advice: Some("'b', 'kb', 'mb', 'gb', 'tb', 'kib', 'mib', 'gib', 'tib' are supported".into()),
        })?;
        Ok(v)
    }
}
