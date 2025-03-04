// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{io, sync::Arc};

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use crate::KdlType;

/// Error type for the provisioning crate
#[derive(Diagnostic, Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Kdl(#[from] kdl::KdlError),

    #[error("unknown type")]
    UnknownType,

    #[error("unknown variant")]
    UnknownVariant,

    #[diagnostic(transparent)]
    #[error(transparent)]
    InvalidArguments(#[from] InvalidArguments),

    #[diagnostic(transparent)]
    #[error(transparent)]
    InvalidType(#[from] InvalidType),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UnsupportedNode(#[from] UnsupportedNode),

    #[diagnostic(transparent)]
    #[error(transparent)]
    MissingEntry(#[from] MissingEntry),

    #[error("missing node: {0}")]
    MissingNode(&'static str),

    #[diagnostic(transparent)]
    #[error(transparent)]
    MissingProperty(#[from] MissingProperty),

    #[diagnostic(transparent)]
    #[error(transparent)]
    UnsupportedValue(#[from] UnsupportedValue),
}

/// Merged error for parsing failures
/// Returns a list of diagnostics for the user
#[derive(Debug, Diagnostic, Error)]
#[error("failed to parse KDL")]
#[diagnostic(severity(error))]
pub struct ParseError {
    #[source_code]
    pub src: NamedSource<Arc<String>>,
    #[related]
    pub diagnostics: Vec<Error>,
}

/// Error for invalid types
#[derive(Debug, Diagnostic, Error)]
#[error("invalid type, expected {expected_type}")]
#[diagnostic(severity(error))]
pub struct InvalidType {
    #[label]
    pub at: SourceSpan,

    /// The expected type
    pub expected_type: KdlType,
}

/// Error for missing mandatory properties
#[derive(Debug, Diagnostic, Error)]
#[error("missing property: {id}")]
#[diagnostic(severity(error))]
pub struct MissingProperty {
    #[label]
    pub at: SourceSpan,

    pub id: &'static str,

    #[help]
    pub advice: Option<String>,
}

/// Error for missing mandatory properties
#[derive(Debug, Diagnostic, Error)]
#[error("missing entry: {id}")]
#[diagnostic(severity(error))]
pub struct MissingEntry {
    #[label]
    pub at: SourceSpan,

    pub id: String,

    #[help]
    pub advice: Option<String>,
}

/// Error for unsupported node types
#[derive(Debug, Diagnostic, Error)]
#[error("unsupported node: {name}")]
#[diagnostic(severity(error))]
pub struct UnsupportedNode {
    #[label]
    pub at: SourceSpan,

    pub name: String,
}

/// Error for unsupported values
#[derive(Debug, Diagnostic, Error)]
#[error("unsupported value")]
#[diagnostic(severity(error))]
pub struct UnsupportedValue {
    #[label]
    pub at: SourceSpan,

    #[help]
    pub advice: Option<String>,
}

/// Error for invalid arguments
#[derive(Debug, Diagnostic, Error)]
#[error("invalid arguments")]
#[diagnostic(severity(error))]
pub struct InvalidArguments {
    #[label]
    pub at: SourceSpan,

    #[help]
    pub advice: Option<String>,
}

/// Error for missing types
#[derive(Debug, Diagnostic, Error)]
#[error("missing type")]
#[diagnostic(severity(error))]
pub struct MissingType {
    #[label]
    pub at: SourceSpan,

    #[help]
    pub advice: Option<String>,
}
