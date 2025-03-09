// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use kdl::{KdlEntry, KdlNode, NodeKey};

use crate::{Error, FromKdlType, InvalidType, KdlType, MissingEntry, MissingProperty, StorageUnit};

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
