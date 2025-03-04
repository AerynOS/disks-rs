// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{
    get_kdl_entry, get_kdl_property, get_property_str, Constraints, Context, FromKdlProperty, FromKdlType,
    PartitionRole, PartitionTypeGuid, PartitionTypeKDL,
};

/// Command to create a partition
#[derive(Debug)]
pub struct Command {
    /// The disk ID to create the partition on
    pub disk: String,

    /// The reference ID of the partition
    pub id: String,

    /// The role, if any, of the partition
    pub role: Option<PartitionRole>,

    /// The GUID of the partition type
    pub partition_type: Option<PartitionTypeGuid>,

    pub constraints: Constraints,
}

/// Generate a command to create a partition
pub(crate) fn parse(context: Context<'_>) -> Result<super::Command, crate::Error> {
    let disk = get_property_str(context.node, "disk")?;
    let id = get_property_str(context.node, "id")?;
    let role = if let Ok(role) = get_kdl_property(context.node, "role") {
        Some(PartitionRole::from_kdl_property(role)?)
    } else {
        None
    };

    let constraints =
        if let Some(constraints) = context.node.iter_children().find(|n| n.name().value() == "constraints") {
            Constraints::from_kdl_node(constraints)?
        } else {
            return Err(crate::Error::MissingNode("constraints"));
        };

    let partition_type = if let Some(partition_type) = context.node.iter_children().find(|n| n.name().value() == "type")
    {
        match PartitionTypeKDL::from_kdl_type(get_kdl_entry(partition_type, &0)?)? {
            PartitionTypeKDL::GUID => Some(PartitionTypeGuid::from_kdl_node(partition_type)?),
        }
    } else {
        None
    };

    // TODO: Load constraints etc
    Ok(super::Command::CreatePartition(Box::new(Command {
        disk,
        id,
        role,
        constraints,
        partition_type,
    })))
}
