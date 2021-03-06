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
use crate::parse::AtmaScanner;

// External library imports.
use tephra::position::Lf;
use tephra::result::Failure;
use tephra::result::FailureOwned;

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
    source: FailureOwned<Lf>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.msg { write!(f, "{}", msg)?; }
        Ok(())
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<FailureOwned<Lf>> for ParseError {
    fn from(err: FailureOwned<Lf>) -> Self {
        ParseError { msg: Some("parse error".to_owned()), source: err }
    }
}

impl<'t> From<Failure<'t, AtmaScanner, Lf>> for ParseError {
    fn from(err: Failure<'t, AtmaScanner, Lf>) -> Self {
        FailureOwned::from(err).into()
    }
}

////////////////////////////////////////////////////////////////////////////////
// ScriptError
////////////////////////////////////////////////////////////////////////////////
/// A script error occurred.
#[derive(Debug)]
pub struct ScriptError {
    /// The error message.
    pub msg: Cow<'static, str>,
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.msg)
    }
}

impl std::error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
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

    /// A file parse error.
    ParseError {
        /// The error message.
        msg: Option<String>,
        /// The error source.
        source: FailureOwned<Lf>,
    }
}

impl FileError {
    /// Returns true if the error is an IoError of the given ErrorKind
    pub fn is_io_error_kind(&self, kind: std::io::ErrorKind) -> bool {
        match self {
            FileError::IoError { source, .. } => { source.kind() == kind },
            _ => false
        }
    }

    /// Returns a mutable reference to the error's message.
    fn msg_mut(&mut self) -> &mut Option<String> {
        match self {
            FileError::IoError { msg, .. } => msg,
            FileError::RonError { msg, .. } => msg,
            FileError::ParseError { msg, .. } => msg,
        }
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::IoError { msg, .. } => {
                if let Some(msg) = msg { write!(f, "{}", msg)?; }
            },

            FileError::RonError { msg, .. } => {
                if let Some(msg) = msg { write!(f, "{}", msg)?; }
            },

            FileError::ParseError { msg, .. } => {
                if let Some(msg) = msg { write!(f, "{}", msg)?; }
            },
        }
        Ok(())
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Some(&self.source)
        match self {
            FileError::IoError { source, .. } => Some(source),
            FileError::RonError { source, .. } => Some(source),
            FileError::ParseError { source, .. } => Some(source),
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


impl From<ParseError> for FileError {
    fn from(err: ParseError) -> Self {
        FileError::ParseError { msg: err.msg, source: err.source }
    }
}

impl From<FailureOwned<Lf>> for FileError {
    fn from(err: FailureOwned<Lf>) -> Self {
        FileError::ParseError {
            msg: Some("file parse error".to_owned()),
            source: err,
        }
    }
}

impl<'t> From<Failure<'t, AtmaScanner, Lf>> for FileError {
    fn from(err: Failure<'t, AtmaScanner, Lf>) -> Self {
        FailureOwned::from(err).into()
    }
}

////////////////////////////////////////////////////////////////////////////////
// FileErrorContext
////////////////////////////////////////////////////////////////////////////////
/// Trait for enabling convenient construction of `FileError`s.
pub trait FileErrorContext<T> {
    /// Wrap the error value with additional context.
    fn context<S>(self, msg: S) -> Result<T, FileError>
        where S: ToString;

    /// Wrap the error value with additional context. The given closure is
    /// evaluated lazily when an error occurs.
    fn with_context<F, S>(self, msg: F) -> Result<T, FileError>
        where
            F: FnOnce() -> S,
            S: ToString;
}

impl<T, E> FileErrorContext<T> for Result<T, E> where E: Into<FileError> {

    fn context<S>(self, msg: S) -> Result<T, FileError>
        where S: ToString
    {
        self.map_err(|err| {
            let mut file_error = err.into();
            *file_error.msg_mut() = Some(msg.to_string());
            file_error
        })
    }

    fn with_context<F, S>(self, msg_fn: F) -> Result<T, FileError>
        where
            F: FnOnce() -> S,
            S: ToString
    {
        self.map_err(|err| {
            let mut file_error = err.into();
            *file_error.msg_mut() = Some((msg_fn)().to_string());
            file_error
        })
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

    /// All available positions in the palette are assigned.
    AllPositionsAssigned,

    /// An expression or value was given an invalid value.
    InvalidInputValue {
        /// A description of the invalid input.
        msg: Cow<'static, str>,
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

            PaletteError::AllPositionsAssigned => write!(f,
                "All palette positions are already assigned."),

            PaletteError::InvalidInputValue { msg } => write!(f,
                "Invalid input value: {}", msg)
        }
    }
}

impl std::error::Error for PaletteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
