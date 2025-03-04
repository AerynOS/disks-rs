// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{get_kdl_entry, kdl_value_to_storage_size};

/// Constraints for partition size, 1:1 mapping to SizeRequirements in
/// partitioning strategy internals.
#[derive(Debug, Default, Clone, Copy)]
pub enum Constraints {
    /// Exact size in bytes
    Exact(u64),
    /// Minimum size in bytes, using more if available
    AtLeast(u64),
    /// Between min and max bytes
    Range { min: u64, max: u64 },
    /// Use all remaining space
    Remaining,

    /// Default constraints
    #[default]
    Invalid,
}

impl Constraints {
    pub fn from_kdl_node(node: &kdl::KdlNode) -> Result<Self, crate::Error> {
        let range = node
            .iter_children()
            .find(|n| n.name().value() == "min")
            .zip(node.iter_children().find(|n| n.name().value() == "max"));

        if let Some((min, max)) = range {
            let min = kdl_value_to_storage_size(get_kdl_entry(min, &0)?)? as u64;
            let max = kdl_value_to_storage_size(get_kdl_entry(max, &0)?)? as u64;

            Ok(Self::Range {
                min: min as u64,
                max: max as u64,
            })
        } else if let Some(min) = node.iter_children().find(|n| n.name().value() == "min") {
            let min = kdl_value_to_storage_size(get_kdl_entry(min, &0)?)? as u64;
            Ok(Self::AtLeast(min as u64))
        } else if let Some(exact) = node.iter_children().find(|n| n.name().value() == "exactly") {
            let exact = kdl_value_to_storage_size(get_kdl_entry(exact, &0)?)? as u64;
            Ok(Self::Exact(exact as u64))
        } else if node.iter_children().any(|n| n.name().value() == "remaining") {
            Ok(Self::Remaining)
        } else {
            Err(crate::Error::MissingProperty(crate::MissingProperty {
                at: node.span(),
                id: "min, max, exactly or remaining",
                advice: Some("add one of these properties".into()),
            }))
        }
    }
}
