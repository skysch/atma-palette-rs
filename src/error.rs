////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellRef;

// Standard library imports.
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// ParseError
////////////////////////////////////////////////////////////////////////////////
/// A parse error occurred.
#[derive(Debug)]
pub struct ParseError {
    /// The error message.
    msg: Option<String>,
    /// The error source.
    source: crate::parse::FailureOwned,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.msg { writeln!(f, "{}", msg)?; }
        writeln!(f, "{}", self.source)
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<crate::parse::FailureOwned> for ParseError {
    fn from(err: crate::parse::FailureOwned) -> Self {
        ParseError { msg: None, source: err }
    }
}

impl<'t> From<crate::parse::Failure<'t>> for ParseError {
    fn from(err: crate::parse::Failure<'t>) -> Self {
        err.to_owned().into()
    }
}

////////////////////////////////////////////////////////////////////////////////
// FileError
////////////////////////////////////////////////////////////////////////////////
/// A file error occurred.
#[derive(Debug)]
pub enum FileError {
    /// An I/O error.
    IoError {
        /// The error message.
        msg: Option<String>,
        /// The error source.
        source: std::io::Error,
    },

    /// A RON error.
    RonError {
        /// The error message.
        msg: Option<String>,
        /// The error source.
        source: ron::error::Error
    },
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::IoError { msg, source } => {
                if let Some(msg) = msg { writeln!(f, "{}", msg)?; }
                writeln!(f, "{}", source)
            },

            FileError::RonError { msg, source } => {
                if let Some(msg) = msg { writeln!(f, "{}", msg)?; }
                writeln!(f, "{}", source)
            },
        }
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Some(&self.source)
        match self {
            FileError::IoError { source, .. } => Some(source),
            FileError::RonError { source, .. } => Some(source),
        }
    }
}

impl From<ron::error::Error> for FileError {
    fn from(err: ron::error::Error) -> Self {
        FileError::RonError { msg: None, source: err }
    }
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::IoError { msg: None, source: err }
    }
}

////////////////////////////////////////////////////////////////////////////////
// PaletteError
////////////////////////////////////////////////////////////////////////////////
/// A palette error occurred.
#[derive(Debug)]
pub enum PaletteError {
    /// An attempt to resolve a CellRef failed.
    UndefinedCellReference {
        /// The failing reference.
        cell_ref: CellRef<'static>,
    },

    /// An group index was out of bounds.
    GroupIndexOutOfBounds {
        /// The group.
        group: Cow<'static, str>,
        /// The given index.
        index: u32,
        /// The maximum index.
        max: u32,
    },

    /// An attempt to resolve a cell's color failed.
    UndefinedColor {
        /// The failing reference.
        cell_ref: CellRef<'static>,
        /// Whether the color is undefined due to a circular reference.
        circular: bool,
    },
}

impl std::fmt::Display for PaletteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {

            PaletteError::UndefinedCellReference { cell_ref } => {
                write!(f, "undefined cell reference: {}", cell_ref)
            },
            
            PaletteError::GroupIndexOutOfBounds { group, index, max } => {
                write!(f, 
                    "group index out of bounds: {}:{} > {}",
                    group,
                    index,
                    max)
            },

            PaletteError::UndefinedColor { cell_ref, circular } => {
                write!(f,
                    "color is undefined {} cell references: {}",
                    if *circular { "due to circular" } else { "for" },
                    cell_ref)
            },
        }
    }
}

impl std::error::Error for PaletteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
