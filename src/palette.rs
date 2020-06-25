////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Main Palette definition.
////////////////////////////////////////////////////////////////////////////////


// Local imports.
use crate::basic::BasicPalette;
use crate::cell::Cell;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::error::Error;
use crate::expr::Expr;
use crate::history::History;
use crate::operation::Operation;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use ron::ser::PrettyConfig;
use ron::ser::to_string_pretty;

// Standard library imports.
use std::borrow::Cow;
use std::convert::TryInto;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;


/// The main Atma Palette object.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize)]
pub struct Palette {
    basic: BasicPalette,
    history: History,
}


impl Palette {
    /// Constructs a new `Palette`.
    pub fn new() -> Self {
        Palette {
            basic: BasicPalette::new(),
            history: History::new(),
        }
    }

    /// Constructs a new `Palette` by parsing data from the file at the given
    /// path.
    pub fn new_from_path(path: &Path) -> Result<Self, Error>  {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| Error::IoError { 
                msg: Some(format!("Failed to open file {:?}", path)),
                source: e,
            })?;
        Palette::new_from_file(&mut file)
    }

    /// Constructs a new `Palette` by parsing data from the given file.
    pub fn new_from_file(file: &mut File) -> Result<Self, Error>  {
        Palette::parse_ron_from_file(file)
    }

    /// Writes the `Palette` to the file at the given path.
    pub fn write_to_path(&self, path: &Path) -> Result<(), Error>  {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| Error::IoError { 
                msg: Some(format!("Failed to open file {:?}", path)),
                source: e,
            })?;
        self.write_to_file(&mut file)
    }

    /// Writes the `Palette` to the given file.
    pub fn write_to_file(&self, file: &mut File) -> Result<(), Error>  {
        self.generate_ron_into_file(file)
    }

    /// Parses a `Palette` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .map_err(|e| Error::IoError { 
                msg: Some("Failed to read file metadata".to_owned()),
                source: e,
            })?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .map_err(|e| Error::IoError { 
                msg: Some("Failed to read palette file".to_owned()),
                source: e,
            })?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .map_err(|e| Error::RonError { 
                msg: Some("Failed deserializing RON file".to_owned()),
                source: e,
            })?;
        let palette = Palette::deserialize(&mut d)
            .map_err(|e| Error::RonError { 
                msg: Some("Failed parsing RON file".to_owned()),
                source: e,
            })?;
        d.end()
            .map_err(|e| Error::RonError { 
                msg: Some("Failed parsing RON file".to_owned()),
                source: e,
            })?;
        Ok(palette)
    }

    /// Generates a RON formatted `Palette` by serializing into the given file.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
        let pretty = PrettyConfig::new()
            .with_depth_limit(2)
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let s = to_string_pretty(self, pretty)?;

        file.write_all(s.as_bytes())?;
        Ok(())
    }


    ////////////////////////////////////////////////////////////////////////////
    // Operations
    ////////////////////////////////////////////////////////////////////////////

    /// Applies a sequence of `Operation`s to the palette.
    ///
    /// ### Parameters
    /// + `op`: The operation to apply.
    pub fn apply_operations(&mut self, ops: &[Operation]) -> Result<(), Error> {
        self.basic.apply_operations(ops, Some(&mut self.history))
    }

    /// Unapplies the latest set of applied operations.
    ///
    /// Returns the number of undo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer undo operations
    /// recorded than requested.
    pub fn undo(&mut self, count: usize) -> usize {
        self.basic.undo(&mut self.history, count)
    }

    /// Reapplies the latest set of undone operations.
    ///
    /// Returns the number of redo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer redo operations
    /// recorded than requested.
    pub fn redo(&mut self, count: usize) -> usize {
        self.basic.redo(&mut self.history, count)
    }


    ////////////////////////////////////////////////////////////////////////////
    // Accessors
    ////////////////////////////////////////////////////////////////////////////

    /// Retreives a reference to the `Cell` associated with the given `CellRef`.
    pub fn cell<'name>(&self, cell_ref: CellRef<'name>)
        -> Result<&Cell, Error>
    {
        self.basic.cell(cell_ref)
    }

    /// Retreives a mutable reference to the `Cell` associated with the given
    /// `CellRef`.
    pub fn cell_mut<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<&mut Cell, Error>
    {
        self.basic.cell_mut(cell_ref)
    }
    
}

impl Default for Palette {
    fn default() -> Self {
        Palette::new()
    }
}
