// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, str::FromStr};

use crate::kdl_value_to_string;

use super::FromKdlProperty;

/// The type of partition table to create
#[derive(Debug, PartialEq)]
pub enum PartitionTableType {
    /// GUID Partition Table
    Gpt,

    /// Master Boot Record
    Msdos,
}

impl fmt::Display for PartitionTableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gpt => f.write_str("gpt"),
            Self::Msdos => f.write_str("msdos"),
        }
    }
}

impl FromStr for PartitionTableType {
    type Err = crate::Error;

    /// Attempt to convert a string to a partition table type
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "gpt" => Ok(Self::Gpt),
            "msdos" => Ok(Self::Msdos),
            _ => Err(crate::Error::UnknownVariant),
        }
    }
}

impl FromKdlProperty<'_> for PartitionTableType {
    fn from_kdl_property(entry: &kdl::KdlEntry) -> Result<Self, crate::Error> {
        let value = kdl_value_to_string(entry)?;
        let v = value.parse().map_err(|_| crate::UnsupportedValue {
            at: entry.span(),
            advice: Some("'gpt' and 'msdos' are supported".into()),
        })?;
        Ok(v)
    }
}
