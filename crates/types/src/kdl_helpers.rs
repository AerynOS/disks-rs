// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use crate::{Error, InvalidType, MissingEntry, MissingProperty, StorageUnit};
use kdl::{KdlEntry, KdlNode, KdlValue, NodeKey};

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

// Get a property from a node
pub fn get_kdl_property<'a>(node: &'a KdlNode, name: &'static str) -> Result<&'a KdlEntry, Error> {
    let entry = node.entry(name).ok_or_else(|| MissingProperty {
        at: node.span(),
        id: name,
        advice: Some(format!("add `{name}=...` to bind the property")),
    })?;

    Ok(entry)
}

pub fn get_kdl_entry<'a, T>(node: &'a KdlNode, id: &'a T) -> Result<&'a KdlEntry, Error>
where
    T: Into<NodeKey> + ToString + Clone,
{
    let entry = node.entry(id.clone()).ok_or_else(|| MissingEntry {
        at: node.span(),
        id: id.to_string(),
        advice: None,
    })?;

    Ok(entry)
}

// Get a string property from a value
pub fn kdl_value_to_string(entry: &kdl::KdlEntry) -> Result<String, Error> {
    let value = entry.value().as_string().ok_or(InvalidType {
        at: entry.span(),
        expected_type: KdlType::String,
    })?;

    Ok(value.to_owned())
}

// Get an integer property from a value
pub fn kdl_value_to_integer(entry: &kdl::KdlEntry) -> Result<i128, Error> {
    let value = entry.value().as_integer().ok_or(InvalidType {
        at: entry.span(),
        expected_type: KdlType::Integer,
    })?;

    Ok(value)
}

// Convert a KDL value to a storage size
pub fn kdl_value_to_storage_size(entry: &kdl::KdlEntry) -> Result<u64, Error> {
    let value = kdl_value_to_integer(entry)?;
    let units = StorageUnit::from_kdl_type(entry)?;
    Ok(value as u64 * units as u64)
}

// Get a string property from a node
pub fn get_property_str(node: &KdlNode, name: &'static str) -> Result<String, Error> {
    let value = get_kdl_property(node, name).and_then(kdl_value_to_string)?;
    Ok(value.to_owned())
}
