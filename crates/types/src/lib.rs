// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use kdl::{KdlEntry, KdlValue};

mod kdl_helpers;
pub use kdl_helpers::*;
mod errors;
pub use errors::*;

pub use gpt;

mod partition_table;
pub use partition_table::*;
mod partition_role;
pub use partition_role::*;

mod units;
pub use units::*;
pub mod constraints;
pub use constraints::*;
pub mod filesystem;
pub use filesystem::*;
mod partition_type;
pub use partition_type::*;

/// The type of a KDL value
#[derive(Debug)]
pub enum KdlType {
    /// A boolean value
    Boolean,
    /// A string value
    String,
    /// A null value
    Null,
    /// An integer value
    Integer,
}

impl KdlType {
    // Determine the kdl value type
    pub fn for_value(value: &KdlValue) -> Result<Self, Error> {
        if value.is_bool() {
            Ok(Self::Boolean)
        } else if value.is_string() {
            Ok(Self::String)
        } else if value.is_null() {
            Ok(Self::Null)
        } else if value.is_integer() {
            Ok(Self::Integer)
        } else {
            Err(Error::UnknownType)
        }
    }
}

impl fmt::Display for KdlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KdlType::Boolean => f.write_str("boolean"),
            KdlType::String => f.write_str("string"),
            KdlType::Null => f.write_str("null"),
            KdlType::Integer => f.write_str("int"),
        }
    }
}

pub trait FromKdlProperty<'a>: Sized {
    fn from_kdl_property(entry: &'a KdlEntry) -> Result<Self, Error>;
}

pub trait FromKdlType<'a>: Sized {
    fn from_kdl_type(id: &'a KdlEntry) -> Result<Self, Error>;
}
