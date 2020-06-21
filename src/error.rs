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
    UnrecognizedCellReference {
        /// The failing reference.
        cell_ref: CellRef,
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            RonError { msg, source } => match msg {
                Some(msg) => write!(f, "{}", msg),
                None => write!(f, "{}", source),
            },
            IoError { msg, source } => match msg {
                Some(msg) => write!(f, "{}", msg),
                None => write!(f, "{}", source),
            },
            UnrecognizedCellReference { cell_ref } => write!(f, 
                "Invalid cell reference: {}", cell_ref),
            
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            RonError { source, .. } => Some(source),
            IoError { source, .. } => Some(source),
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
