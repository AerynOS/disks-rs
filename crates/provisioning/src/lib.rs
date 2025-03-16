// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{fs, path::Path, sync::Arc};

use itertools::{Either, Itertools};
use kdl::{KdlDocument, KdlNode};
use miette::{Diagnostic, NamedSource, Severity};

mod provisioner;
pub use provisioner::*;

mod commands;
use commands::*;

pub use types::*;

/// Command evaluation context
pub struct Context<'a> {
    /// The node being parsed
    pub(crate) node: &'a KdlNode,
}

/// A strategy definition
#[derive(Debug)]
pub struct StrategyDefinition {
    /// The name of the strategy
    pub name: String,

    /// A brief summary of the strategy
    pub summary: String,

    /// The strategy that this strategy inherits from
    pub inherits: Option<String>,

    /// The commands to execute
    pub commands: Vec<Command>,
}

/// A parser for provisioning strategies
#[derive(Debug)]
pub struct Parser {
    pub strategies: Vec<StrategyDefinition>,
}

impl Parser {
    /// Create a new parser from a file path
    pub fn new_for_path<P>(file: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        let file = file.as_ref();
        let name = file.to_string_lossy();
        let txt = fs::read_to_string(file).map_err(|e| ParseError {
            src: NamedSource::new(&name, Arc::new("".to_string())),
            diagnostics: vec![e.into()],
        })?;
        Self::new(&name, &txt)
    }

    /// Create a new parser from a string
    pub fn new(name: &str, contents: &str) -> Result<Self, ParseError> {
        let source = Arc::new(contents.to_string());
        let ns = NamedSource::new(name, source).with_language("KDL");
        let mut errors = vec![];

        // Parse the document and collect any errors
        let d = KdlDocument::parse_v2(ns.inner()).map_err(|e| ParseError {
            src: ns.clone(),
            diagnostics: vec![e.into()],
        })?;

        let mut strategies = vec![];

        for node in d.nodes() {
            match node.name().value() {
                "strategy" => match Self::parse_strategy(node) {
                    Ok(strategy) => strategies.push(strategy),
                    Err(e) => errors.extend(e),
                },
                _ => {
                    errors.push(
                        UnsupportedNode {
                            at: node.span(),
                            name: node.name().to_string(),
                        }
                        .into(),
                    );
                }
            }
        }

        if !errors.is_empty() {
            return Err(ParseError {
                src: ns,
                diagnostics: errors,
            });
        }

        Ok(Self { strategies })
    }

    // Parse a strategy node
    fn parse_strategy(node: &KdlNode) -> Result<StrategyDefinition, Vec<Error>> {
        let mut errors = vec![];
        let name = match get_property_str(node, "name") {
            Ok(name) => name,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };
        let summary = match get_property_str(node, "summary") {
            Ok(summary) => summary,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };
        let inherits = if node.entry("inherits").is_some() {
            match get_property_str(node, "inherits") {
                Ok(inherits) => Some(inherits),
                Err(e) => {
                    errors.push(e);
                    None
                }
            }
        } else {
            None
        };

        // Collect all failures in this strategy
        let (commands, child_errors): (Vec<_>, Vec<_>) =
            node.iter_children()
                .partition_map(|node| match parse_command(Context { node }) {
                    Ok(cmd) => Either::Left(cmd),
                    Err(e) => Either::Right(e),
                });

        errors.extend(child_errors);

        let fatal_errors = errors
            .iter()
            .filter(|e| matches!(e.severity().unwrap_or(Severity::Error), Severity::Error));

        // If we have any fatal errors, bail out
        // TODO: Add an error sink to allow bubbling up of warnings/diagnostics
        // for tooling integration
        if fatal_errors.clone().next().is_some() {
            return Err(errors);
        }

        let strategy = StrategyDefinition {
            name,
            summary,
            inherits,
            commands,
        };

        Ok(strategy)
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    #[test]
    //#[should_panic]
    fn test_basic() -> miette::Result<()> {
        let _p = Parser::new_for_path("tests/use_whole_disk.kdl")?;
        eprintln!("p: {_p:?}");
        Ok(())
    }
}
