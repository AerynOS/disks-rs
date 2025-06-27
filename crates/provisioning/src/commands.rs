// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::Context;

mod create_partition;
mod create_partition_table;
mod find_disk;

/// A command
#[derive(Debug)]
pub enum Command {
    CreatePartition(Box<create_partition::Command>),
    CreatePartitionTable(Box<create_partition_table::Command>),
    FindDisk(Box<find_disk::Command>),
}

/// Command execution function
type CommandExec = for<'a> fn(Context<'a>) -> Result<Command, crate::Error>;

fn command(name: &str) -> Option<CommandExec> {
    Some(match name {
        "find-disk" => find_disk::parse,
        "create-partition" => create_partition::parse,
        "create-partition-table" => create_partition_table::parse,
        _ => return None,
    })
}

/// Parse a command from a node if possible
pub(crate) fn parse_command(context: Context<'_>) -> Result<Command, crate::Error> {
    let name = context.node.name().value();
    let func = command(name).ok_or_else(|| crate::UnsupportedNode {
        at: context.node.span(),
        name: name.into(),
    })?;

    func(context)
}
