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
use crate::cell::CellSelection;
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::command::Positioning;
use crate::command::CursorBehavior;
use crate::error::FileError;
use crate::error::FileErrorContext as _;
use crate::error::PaletteError;
use crate::palette::BasicPalette;
use crate::palette::InsertExpr;
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
    pub fn read_from_path<P>(path: P) -> Result<Self, FileError>
        where P: AsRef<Path> + Debug
    {
        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .with_context(|| format!("Failed to open file {:?}", path))?;
        let mut palette = Palette::read_from_file(&mut file)?;
        palette.load_path = Some(path.to_owned());
        Ok(palette)
    }

    /// Writes the `Palette` to the file at the given path.
    pub fn write_to_path<P>(&self, path: P) -> Result<(), FileError>
        where P: AsRef<Path> + Debug
    {
        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .with_context(|| format!(
                "Failed to create/open file {:?} for writing",
                path))?;
        self.write_to_file(&mut file)
    }

    /// Writes the `Palette` to a new file at the given path.
    pub fn write_to_path_if_new<P>(&self, path: P) -> Result<(), FileError>
        where P: AsRef<Path> + Debug
    {
        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("Failed to create file {:?}.", path))?;
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

    /// Write the `Palette` into a new file using the load path. Returns true if
    /// the data was written.
    pub fn write_to_load_path_if_new(&self) -> Result<bool, FileError> {
        match &self.load_path {
            Some(path) => {
                self.write_to_path_if_new(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
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
    
    /// Writes the `Palette` to the given file.
    pub fn write_to_file(&self, file: &mut File) -> Result<(), FileError> {
        self.generate_ron_into_file(file)
    }

    /// Generates a RON formatted `Palette` by serializing into the given file.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), FileError> {
        debug!("Serializing & writing Palette file.");
        let pretty = PrettyConfig::new()
            .with_depth_limit(3)
            .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME);
        let s = to_string_pretty(self, pretty)?;

        file.write_all(s.as_bytes())
            .map_err(FileError::from)
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
    
    /// Inserts the given color expression objects into the palette.
    pub fn insert_exprs<'name, S>(
        &mut self,
        insert_exprs: &[InsertExpr],
        name: Option<S>,
        positioning: Positioning,
        cursor_behavior: CursorBehavior)
        -> Result<(), PaletteError>
        where S: ToString
    {
        use Operation::*;
        // Get start index.
        let mut idx = self.inner
            .unoccupied_index_or_next(0)
            .expect("no free indices"); // TODO: Handle with an error.
        // Get start position.
        let mut next = match positioning {
            Positioning::Position(p) => p,
            Positioning::Open => Position::ZERO,
            Positioning::Cursor => self.inner.position_cursor(),
            Positioning::None => Position::ZERO,
        };
        if !positioning.is_none() {
            next = self.inner
                .unoccupied_position_or_next(next)
                .expect("no free positions"); // TODO: Handle with an error.
        }
        let start_position = next.clone();

        // Convert name into proper format.
        let name: Option<Cow<'static, str>> = name
            .map(|n| n.to_string().into());


        let mut name_used = false;
        let mut ops = Vec::with_capacity(insert_exprs.len() * 2);
        for insert_expr in insert_exprs {
            for exprs in insert_expr.exprs(&self.inner) {
                for expr in exprs {
                    // Insert Cell
                    ops.push(InsertCell {
                        idx,
                        cell: Cell::new_with_expr(expr),
                    });

                    // Assign position.
                    if !positioning.is_none() { 
                        ops.push(AssignPosition {
                            cell_ref: CellRef::Index(idx),
                            position: next.clone(),
                        });
                    }

                    // Assign group.
                    if let Some(name) = &name {
                        name_used = true;
                        ops.push(AssignGroup {
                            cell_ref: CellRef::Index(idx),
                            group: name.clone(),
                            idx: None,
                        });
                    }

                    // Shift to next index.
                    idx = self.inner
                        .unoccupied_index_or_next(idx.wrapping_add(1))
                        .expect("no free indices"); 

                    // Shift to next position.
                    if !positioning.is_none() {
                        next = self.inner
                            .unoccupied_position_or_next(next.wrapping_succ())
                            .expect("no free positions");
                    }
                }
            }
        }

        match name {
            Some(name) if !name_used => ops.push(AssignName {
                selector: PositionSelector::from(next),
                name,
            }),
            _ => (),
        }
        
        if !positioning.is_none() {
            match cursor_behavior {
                CursorBehavior::RemainInPlace => (),

                CursorBehavior::MoveToStart
                    => ops.push(SetPositionCursor { position: start_position }),
                
                CursorBehavior::MoveAfterEnd
                    => ops.push(SetPositionCursor { position: next.succ() }),
                
                CursorBehavior::MoveToOpen => {
                    let position = self.inner
                        .unoccupied_position_or_next(Position::ZERO)
                        .expect("no free positions");
                    ops.push(SetPositionCursor { position })
                },
            }
        }
        self.apply_operations(&ops[..])
    }

    /// Deletes the selected cells from the palette.
    pub fn delete_selection<'name>(
        &mut self,
        selection: CellSelection<'name>,
        cursor_behavior: CursorBehavior)
        -> Result<(), PaletteError>
    {
        use Operation::*;

        let index_selection = selection.resolve(self.inner());    
        let mut saved_position: Option<Position> = None;
        let mut ops = Vec::new();

        for idx in index_selection {
            let cell_ref = CellRef::Index(idx);
            saved_position = match cursor_behavior {
                CursorBehavior::MoveToStart => match (
                    self.inner().assigned_position(&cell_ref).cloned(), 
                    saved_position.take())
                {
                    (Some(a), None)             => Some(a),
                    (Some(a), Some(s)) if a < s => Some(a),
                    (_,       Some(s))          => Some(s),
                    (None,    None)             => None,
                },
                CursorBehavior::MoveAfterEnd => match (
                    self.inner().assigned_position(&cell_ref).cloned(), 
                    saved_position.take())
                {
                    (Some(a), None)             => Some(a),
                    (Some(a), Some(s)) if a > s => Some(a),
                    (_,       Some(s))          => Some(s),
                    (None,    None)             => None,
                },
                _ => saved_position,
            };

            ops.push(RemoveCell { cell_ref });
        }

        match cursor_behavior {
            CursorBehavior::RemainInPlace => (),

            CursorBehavior::MoveToStart |
            CursorBehavior::MoveAfterEnd
                => if let Some(position) = saved_position
            {
                ops.push(SetPositionCursor { position });
            }
            
            CursorBehavior::MoveToOpen => if let Some(position) = self.inner
                .unoccupied_position_or_next(Position::ZERO)
            {
                ops.push(SetPositionCursor { position })
            },
        }
        self.apply_operations(&ops[..])
    }


    /// Moves the selected cells within the palette.
    pub fn move_selection<'name>(
        &mut self,
        selection: CellSelection<'name>,
        positioning: Positioning,
        cursor_behavior: CursorBehavior)
        -> Result<(), PaletteError>
    {
        use Operation::*;
        // TODO: Fix this method so it doesn't misbehave for overlapping moves.
        // Maybe apply operations directly and build the undo ourselves?

        let index_selection = selection.resolve(self.inner());    
        let mut ops = Vec::new();
        let mut next = match positioning {
            Positioning::Position(p) => p,
            Positioning::Open => Position::ZERO,
            Positioning::Cursor => self.inner.position_cursor(),
            Positioning::None => Position::ZERO,
        };
        let start_position = next.clone();

        for idx in index_selection {
            if positioning.is_none() {
                ops.push(UnassignPosition {
                    cell_ref: CellRef::Index(idx)
                });
                continue;
            }

            match self.inner()
                .unoccupied_position_or_next(next)
            {
                Some(pos) => {
                    ops.push(AssignPosition {
                        cell_ref: CellRef::Index(idx),
                        position: pos,
                    });
                    next = pos.succ();
                },
                None => return Err(PaletteError::AllPositionsAssigned),
            }
        }

        if !positioning.is_none() {
            match cursor_behavior {
                CursorBehavior::RemainInPlace => (),

                CursorBehavior::MoveToStart
                    => ops.push(SetPositionCursor { position: start_position }),
                
                CursorBehavior::MoveAfterEnd
                    => ops.push(SetPositionCursor { position: next.succ() }),
                
                CursorBehavior::MoveToOpen => {
                    let position = self.inner
                        .unoccupied_position_or_next(Position::ZERO)
                        .unwrap_or(next.succ());
                    ops.push(SetPositionCursor { position })
                },
            }
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
        match self.history.as_mut() {
            Some(history) => self.inner.undo(history, count),
            _             => 0,
        }
    }

    /// Reapplies the latest set of undone operations.
    /// 
    /// Returns the number of redo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer redo operations
    /// recorded than requested.
    pub fn redo(&mut self, count: usize) -> usize {
        match self.history.as_mut() {
            Some(history) => self.inner.redo(history, count),
            _             => 0,
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
