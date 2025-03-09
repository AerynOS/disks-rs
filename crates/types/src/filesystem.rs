// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fmt, str::FromStr};

use crate::{get_kdl_entry, kdl_value_to_string};

use super::FromKdlProperty;

/// The filesystem information for a partition
/// This is used to format the partition with a filesystem
///
/// The "any" filesystem type is used to indicate that any filesystem is acceptable.
#[derive(Debug, PartialEq)]
pub struct Filesystem {
    /// The filesystem type
    pub filesystem_type: FilesystemType,

    /// The label of the filesystem
    pub label: Option<String>,

    /// The UUID of the filesystem
    pub uuid: Option<String>,
}

/// The filesystem type for a partition
#[derive(Debug, PartialEq, Default)]
pub enum FilesystemType {
    /// FAT32 filesystem
    Fat32,

    /// F2FS filesystem
    F2fs,

    /// EXT4 filesystem
    Ext4,

    /// XFS filesystem
    Xfs,

    /// Swap partition
    Swap,

    /// Any filesystem
    #[default]
    Any,
}

impl fmt::Display for FilesystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fat32 => f.write_str("fat32"),
            Self::Ext4 => f.write_str("ext4"),
            Self::F2fs => f.write_str("f2fs"),
            Self::Xfs => f.write_str("xfs"),
            Self::Swap => f.write_str("swap"),
            Self::Any => f.write_str("any"),
        }
    }
}

impl FromStr for FilesystemType {
    type Err = crate::Error;

    /// Attempt to convert a string to a filesystem type
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fat32" => Ok(Self::Fat32),
            "ext4" => Ok(Self::Ext4),
            "f2fs" => Ok(Self::F2fs),
            "xfs" => Ok(Self::Xfs),
            "swap" => Ok(Self::Swap),
            "any" => Ok(Self::Any),
            _ => Err(crate::Error::UnknownVariant),
        }
    }
}

impl FromKdlProperty<'_> for FilesystemType {
    fn from_kdl_property(entry: &kdl::KdlEntry) -> Result<Self, crate::Error> {
        let value = kdl_value_to_string(entry)?;
        let v = value.parse().map_err(|_| crate::UnsupportedValue {
            at: entry.span(),
            advice: Some("'fat32', 'ext4', 'f2fs', 'xfs' 'swap' and 'any' are supported".into()),
        })?;
        Ok(v)
    }
}

impl Filesystem {
    pub fn from_kdl_node(node: &kdl::KdlNode) -> Result<Self, crate::Error> {
        let mut filesystem_type = None;
        let mut label = None;
        let mut uuid = None;

        for entry in node.iter_children() {
            match entry.name().value() {
                "type" => filesystem_type = Some(FilesystemType::from_kdl_property(get_kdl_entry(entry, &0)?)?),
                "label" => label = Some(kdl_value_to_string(get_kdl_entry(entry, &0)?)?),
                "uuid" => uuid = Some(kdl_value_to_string(get_kdl_entry(entry, &0)?)?),
                _ => {
                    return Err(crate::UnsupportedNode {
                        at: entry.span(),
                        name: entry.name().value().into(),
                    }
                    .into())
                }
            }
        }

        Ok(Self {
            filesystem_type: filesystem_type.ok_or(crate::UnsupportedNode {
                at: node.span(),
                name: "type".into(),
            })?,
            label,
            uuid,
        })
    }
}
