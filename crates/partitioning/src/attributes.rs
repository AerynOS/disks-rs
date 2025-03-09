// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use gpt::partition_types;
use types::{Filesystem, PartitionRole};
use uuid::Uuid;

/// Represents the table attributes of a GPT partition
#[derive(Debug, Clone)]
pub struct GptAttributes {
    /// The type GUID that identifies the partition type
    pub type_guid: partition_types::Type,
    /// Optional name for the partition
    pub name: Option<String>,
    /// Optional UUID for the partition
    pub uuid: Option<Uuid>,
}

impl Default for GptAttributes {
    fn default() -> Self {
        Self {
            type_guid: partition_types::BASIC,
            name: None,
            uuid: None,
        }
    }
}

/// Represents attributes specific to different partition table types
#[derive(Debug, Clone)]
pub enum TableAttributes {
    /// GPT partition attributes
    Gpt(GptAttributes),
    //Mbr(MbrAttributes),
}

impl TableAttributes {
    /// Returns a reference to the GPT attributes if this is a GPT partition
    ///
    /// # Returns
    /// - `Some(&GptAttributes)` if this is a GPT partition
    /// - `None` if this is not a GPT partition
    pub fn as_gpt(&self) -> Option<&GptAttributes> {
        match self {
            TableAttributes::Gpt(attr) => Some(attr),
        }
    }
}

/// Represents the attributes of a partition
#[derive(Debug, Clone)]
pub struct PartitionAttributes {
    pub table: TableAttributes,
    pub role: Option<PartitionRole>,
    pub filesystem: Option<Filesystem>,
}
