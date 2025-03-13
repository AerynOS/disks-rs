// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::io;

#[cfg(feature = "kdl")]
use miette::{Diagnostic, NamedSource, SourceSpan};
#[cfg(feature = "kdl")]
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "kdl")]
use crate::KdlType;

/// Error type for the provisioning crate
#[cfg_attr(feature = "kdl", derive(Diagnostic))]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    Kdl(#[from] kdl::KdlError),

    #[cfg(feature = "kdl")]
    #[error("unknown type")]
    UnknownType,

    #[error("unknown variant")]
    UnknownVariant,

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    InvalidArguments(#[from] InvalidArguments),

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    InvalidType(#[from] InvalidType),

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    UnsupportedNode(#[from] UnsupportedNode),

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    MissingEntry(#[from] MissingEntry),

    #[cfg(feature = "kdl")]
    #[error("missing node: {0}")]
    MissingNode(&'static str),

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    MissingProperty(#[from] MissingProperty),

    #[cfg(feature = "kdl")]
    #[diagnostic(transparent)]
    #[error(transparent)]
    UnsupportedValue(#[from] UnsupportedValue),
}

#[cfg(feature = "kdl")]
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

#[cfg(feature = "kdl")]
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

#[cfg(feature = "kdl")]
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

#[cfg(feature = "kdl")]
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

#[cfg(feature = "kdl")]
/// Error for unsupported node types
#[derive(Debug, Diagnostic, Error)]
#[error("unsupported node: {name}")]
#[diagnostic(severity(error))]
pub struct UnsupportedNode {
    #[label]
    pub at: SourceSpan,

    pub name: String,
}

#[cfg(feature = "kdl")]
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

#[cfg(feature = "kdl")]
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

#[cfg(feature = "kdl")]
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
