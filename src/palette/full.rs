////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application-level palette wrapper.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::Cell;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::color::Color;
use crate::error::FileError;
use crate::error::FileErrorContext as _;
use crate::error::PaletteError;
use crate::palette::BasicPalette;
use crate::palette::Expr;
use crate::palette::History;
use crate::palette::Operation;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use ron::ser::PrettyConfig;
use ron::ser::to_string_pretty;
use log::*;

// Standard library imports.
use std::borrow::Cow;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;



////////////////////////////////////////////////////////////////////////////////
// DEFAULT_PALETTE_PATH
////////////////////////////////////////////////////////////////////////////////
/// The default path for the [`Palette`] file, relative to the application root.
///
/// [`Palette`]: struct.Palette.html
pub const DEFAULT_PALETTE_PATH: &'static str = "palette.atma";

////////////////////////////////////////////////////////////////////////////////
// Palette
////////////////////////////////////////////////////////////////////////////////
/// The Palette object.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Palette {
    /// The path the palette was initially loaded from.
    #[serde(skip)]
    load_path: Option<PathBuf>,
    /// The internal palette data.
    inner: BasicPalette,
    /// The command history for the palette.
    history: Option<History>,
}


impl Palette {
    /// Constructs a new `Palette`.
    pub fn new() -> Self {
        Palette {
            load_path: None,
            inner: BasicPalette::new(),
            history: None,
        }
    }

    /// Returns the given `Palette` with a new undo/redo history.
    pub fn with_history(mut self) -> Self {
        self.history = Some(History::new());
        self
    }

    /// Returns the given `Palette` with the given load_path.
    pub fn with_load_path<P>(mut self, path: P) -> Self
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
        self
    }

    /// Returns the `Palette`'s load path.
    pub fn load_path(&self) -> Option<&Path> {
        self.load_path.as_ref().map(AsRef::as_ref)
    }

    /// Sets the `Palette`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
    }

    /// Constructs a new `Palette` by parsing data from the file at the given
    /// path.
    pub fn read_from_path<P>(path: &P) -> Result<Self, FileError>
        where P: AsRef<Path> + Debug
    {
        let path = path.as_ref().to_owned();
        let mut file = OpenOptions::new()
            .read(true)
            .open(&path)
            .with_context(|| format!("Failed to open file {:?}", path))?;
        let mut palette = Palette::read_from_file(&mut file)?;
        palette.load_path = Some(path);
        Ok(palette)
    }

    /// Constructs a new `Palette` by parsing data from the given file.
    pub fn read_from_file(file: &mut File) -> Result<Self, FileError> {
        Palette::parse_ron_from_file(file)
    }

    /// Parses a `Palette` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, FileError> {
        let len = file.metadata()
            .context("Failed to read file metadata")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .context("Failed to read palette file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .context("Failed deserializing RON file")?;
        let palette = Palette::deserialize(&mut d)
            .context("Failed parsing RON file")?;
        d.end()
            .context("Failed parsing RON file")?;
        Ok(palette)
    }

    /// Writes the `Palette` to the file at the given path.
    pub fn write_to_path<P>(&self, path: &P) -> Result<(), FileError>
        where P: AsRef<Path> + Debug
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .with_context(|| format!("Failed to open file {:?}", path))?;
        self.write_to_file(&mut file)
    }

    /// Write the `Palette` into the file is was loaded from. Returns true if
    /// the data was written.
    pub fn write_to_load_path(&self) -> Result<bool, FileError> {
        match &self.load_path {
            Some(path) => {
                self.write_to_path(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Writes the `Palette` to the given file.
    pub fn write_to_file(&self, file: &mut File) -> Result<(), FileError> {
        self.generate_ron_into_file(file)
    }

    /// Generates a RON formatted `Palette` by serializing into the given file.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), FileError> {
        let pretty = PrettyConfig::new()
            .with_depth_limit(2)
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let s = to_string_pretty(self, pretty)?;

        file.write_all(s.as_bytes())?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////
    // Accessors
    ////////////////////////////////////////////////////////////////////////////

    /// Retreives a reference to the `Cell` associated with the given `CellRef`.
    pub fn cell<'name>(&self, cell_ref: &CellRef<'name>)
        -> Result<&Cell, PaletteError>
    {
        self.inner.cell(cell_ref)
    }

    /// Retreives a mutable reference to the `Cell` associated with the given
    /// `CellRef`.
    pub fn cell_mut<'name>(&mut self, cell_ref: &CellRef<'name>)
        -> Result<&mut Cell, PaletteError>
    {
        self.inner.cell_mut(cell_ref)
    }

    #[allow(unused)] // TODO: Remove this.
    /// Returns a reference to the inner `BasicPalette`.
    pub(in crate) fn inner(&self) -> &BasicPalette {
        &self.inner
    }

    /// Returns a `mut` reference to the inner `BasicPalette`.
    pub(in crate) fn inner_mut(&mut self) -> &mut BasicPalette {
        &mut self.inner
    }

    ////////////////////////////////////////////////////////////////////////////
    // Commands
    ////////////////////////////////////////////////////////////////////////////
    /// Inserts the given colors into the palette.
    ///
    /// ### Parameters
    ///
    /// + `colors`: The [`Colors`] to insert.
    /// + `name`: The name of the colors. Creates a name association for a
    /// single color, or a group association for multiple colors.
    /// + `position`: The starting [`Position`] of the colors.
    ///
    /// [`Colors`]: ../color/struct.Color.html
    /// [`Position`]: ../cell/struct.Position.html
    pub fn insert_colors<'name, S>(
        &mut self,
        colors: &[Color],
        name: Option<S>,
        position: Option<Position>)
        -> Result<(), PaletteError>
        where S: ToString
    {
        use Operation::*;
        // Get start index.
        let mut idx = self.inner
            .unoccupied_index_or_next(0)
            .expect("no free indices"); // TODO: Handle with an error.
        // Get start position.
        let mut next = position.unwrap_or(Position::ZERO);
        next = self.inner
            .unoccupied_position_or_next(next)
            .expect("no free positions"); // TODO: Handle with an error.

        // Convert name into proper format.
        let name: Option<Cow<'static, str>> = name
            .map(|n| n.to_string().into());

        let mut ops = Vec::with_capacity(colors.len() * 2);
        for color in colors {
            debug!("Inserting color {} at {}", color, next);
            // insert_cell
            ops.push(InsertCell {
                idx,
                cell: Cell::new_with_expr(Expr::Color(color.clone())),
            });
            if let Some(name) = &name {
                // assign_group
                ops.push(AssignGroup {
                    cell_ref: CellRef::Index(idx),
                    group: name.clone(),
                    idx: None,
                });
            }

            // Shift to next position.
            idx = self.inner
                .unoccupied_index_or_next(idx.wrapping_add(1))
                .expect("no free indices"); 
            next = self.inner
                .unoccupied_position_or_next(next.wrapping_succ())
                .expect("no free positions");
        }

        match name {
            Some(name) if colors.len() == 1 => ops.push(AssignName {
                selector: PositionSelector::from(next),
                name,
            }),
            _ => (),
        }

        self.apply_operations(&ops[..])
    }

    ////////////////////////////////////////////////////////////////////////////
    // Operations
    ////////////////////////////////////////////////////////////////////////////

    /// Applies a sequence of `Operation`s to the palette.
    ///
    /// ### Parameters
    /// + `op`: The operation to apply.
    pub fn apply_operations(&mut self, ops: &[Operation])
        -> Result<(), PaletteError>
    {
        self.inner.apply_operations(ops, self.history.as_mut())
    }

    /// Unapplies the latest set of applied operations.
    ///
    /// Returns the number of undo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer undo operations
    /// recorded than requested.
    pub fn undo(&mut self, count: usize) -> usize {
        if let Some(history) = self.history.as_mut() {
            self.inner.undo(history, count)
        } else {
            0
        }
    }

    /// Reapplies the latest set of undone operations.
    ///
    /// Returns the number of redo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer redo operations
    /// recorded than requested.
    pub fn redo(&mut self, count: usize) -> usize {
        if let Some(history) = self.history.as_mut() {
            self.inner.redo(history, count)
        } else {
            0
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::new()
    }
}

#[cfg(test)]
impl PartialEq for Palette {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
        // NOTE: This comparison ignores the command history.
    }
}
