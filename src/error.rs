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
use crate::parse::FailureOwned;
use crate::parse::Failure;

// Standard library imports.
use std::borrow::Cow;


// Standard library imports.
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// InvalidFile
////////////////////////////////////////////////////////////////////////////////
/// The specified file was invalid.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
pub struct InvalidFile;

impl std::error::Error for InvalidFile {}

impl std::fmt::Display for InvalidFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
        -> Result<(), std::fmt::Error> 
    {
        write!(f, "Invalid file.")
    }
}

////////////////////////////////////////////////////////////////////////////////
// MissingFile
////////////////////////////////////////////////////////////////////////////////
/// The specified file was missing.
#[derive(Debug, Clone)]
pub struct MissingFile { 
    /// The path of the missing file.
    pub path: Box<Path>,
}

impl std::error::Error for MissingFile {}

impl std::fmt::Display for MissingFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
        -> Result<(), std::fmt::Error> 
    {
        write!(f, "missing file: {}.", self.path.display())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Error
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
/// Atma palette error type.
pub enum Error {
    /// A RON error.
    RonError {
        /// The error message.
        msg: Option<String>,
        /// The error source.
        source: ron::error::Error
    },
    /// An I/O error.
    IoError {
        /// The error message.
        msg: Option<String>,
        /// The error source.
        source: std::io::Error,
    },

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

    /// A parse error occurred.
    ParseError {
        /// The error message.
        msg: Option<String>,
        /// The error source.
        source: FailureOwned,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            RonError { msg, source } => match msg {
                Some(msg) => write!(f, "{}\n{}", msg, source),
                None => write!(f, "{}", source),
            },
            
            IoError { msg, source } => match msg {
                Some(msg) => write!(f, "{}\n{}", msg, source),
                None => write!(f, "{}", source),
            },

            UndefinedCellReference { cell_ref } => write!(f, 
                "undefined cell reference: {}", cell_ref),
            
            GroupIndexOutOfBounds { group, index, max } => write!(f, 
                "group index out of bounds: {}:{} > {}", group, index, max),

            UndefinedColor { cell_ref, circular } => if *circular {
                write!(f,
                    "color is undefined due to circular cell references: {}",
                    cell_ref)
            } else {
                write!(f, "color is undefined for cell reference: {}",
                    cell_ref)
            },

            ParseError { msg, source } => match msg {
                Some(msg) => write!(f, "{}\n{}", msg, source),
                None => write!(f, "{}", source),
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            RonError { source, .. } => Some(source),
            IoError { source, .. } => Some(source),
            ParseError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<ron::error::Error> for Error {
    fn from(err: ron::error::Error) -> Self {
        Error::RonError { msg: None, source: err }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError { msg: None, source: err }
    }
}

impl From<FailureOwned> for Error {
    fn from(err: FailureOwned) -> Self {
        Error::ParseError { msg: None, source: err }
    }
}

impl<'t> From<Failure<'t>> for Error {
    fn from(err: Failure<'t>) -> Self {
        err.to_owned().into()
    }
}
