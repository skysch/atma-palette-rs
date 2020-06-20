////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette data.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::Cell;
use crate::cell::CellRef;
use crate::cell::CellPackage;
use crate::cell::Position;
use crate::error::Error;
// use crate::expr::Expr;
use crate::operation::Operation;

// External library imports.
// use color::Color;

use serde::Deserialize;
use serde::Serialize;

use ron::ser::PrettyConfig;
use ron::ser::to_string_pretty;

// Standard library imports.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;



/// The Atma palette object.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Palette {
    /// Palette cells storage. Holds cells containing color expressions.
    cells: BTreeMap<u32, Cell>,
    /// The next free cell index.
    next_index: u32,
    /// A map of names assigned to cells.
    names: BTreeMap<String, u32>,
    /// A map of (page, line) positions assigned to cells.
    positions: BTreeMap<Position, u32>,
    /// A map of names assigned to groups of cells.
    groups: BTreeMap<String, Vec<u32>>,
}


impl Palette {

    ////////////////////////////////////////////////////////////////////////////
    // Command-level interface
    ////////////////////////////////////////////////////////////////////////////
    /// Constructs a new `Palette`.
    pub fn new() -> Self {
        Palette {
            cells: BTreeMap::new(),
            next_index: 0,
            names: BTreeMap::new(),
            positions: BTreeMap::new(),
            groups: BTreeMap::new(),
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
    // Operation-level interface
    ////////////////////////////////////////////////////////////////////////////
    /// Applies an `Operation` to the palette. Returns an `Operation` that will
    /// undo the applied changes.
    pub fn apply_operation(&mut self, op: &Operation) -> Operation {
        use Operation::*;
        match op {
            Null => Null,
            InsertCell(cell) => self.insert_cell(cell),
            InsertCellPackage(cell_pack) => self.insert_cell_package(cell_pack),
            RemoveCell(cell_ref) => self.remove_cell(cell_ref),

            // _ => unimplemented!(),
        }
    }

    /// Inserts a `Cell` into the palette.
    pub fn insert_cell(&mut self, cell: &Cell) -> Operation {
        let idx = self.allocate_index();
        match self.cells.insert(idx, *cell) {
            None => Operation::RemoveCell(CellRef::Index(idx)),
            Some(old) => {
                let cell_pack = self.extract_cell_package(idx, Some(old));
                Operation::InsertCellPackage(cell_pack)
            },
        }   
    }

    /// Inserts a `CellPackage` into the palette.
    pub fn insert_cell_package(&mut self, cell_pack: &CellPackage) -> Operation
    {
        //
        unimplemented!()

    }

    /// Removes a `Cell` from the palette.
    pub fn remove_cell(&mut self, cell_ref: &CellRef) -> Operation {
        match Palette::resolve_ref_to_idx(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)
        {
            None => Operation::Null,
            Some(idx) => {
                let removed = self.cells
                    .remove(&idx);
                let cell_pack = self.extract_cell_package(idx, removed);
                Operation::InsertCellPackage(cell_pack)
            },
        }
    }


    ////////////////////////////////////////////////////////////////////////////
    // Direct interface
    ////////////////////////////////////////////////////////////////////////////

    /// Retreives a reference to the `Cell` associated with the given `CellRef`.
    pub fn cell(&self, cell_ref: &CellRef) -> Option<&Cell> {
        Palette::resolve_ref_to_idx(
                &self.names,
                &self.positions,
                &self.groups,
                cell_ref)
            .and_then(|idx| self.cells.get(&idx))
    }

    /// Retreives a mutable reference to the `Cell` associated with the given
    /// `CellRef`.
    pub fn cell_mut(&mut self, cell_ref: &CellRef) -> Option<&mut Cell> {
        Palette::resolve_ref_to_idx(
                &self.names,
                &self.positions,
                &self.groups,
                cell_ref)
            .and_then(move |idx| self.cells.get_mut(&idx))
    }

    fn resolve_ref_to_idx(
        names: &BTreeMap<String, u32>,
        positions: &BTreeMap<Position, u32>,
        groups: &BTreeMap<String, Vec<u32>>,
        cell_ref: &CellRef)
        -> Option<u32>
    {
        use CellRef::*;
        match cell_ref {
            Index(idx) => Some(*idx),
            Name(name) => names
                .get(name)
                .cloned(),
            Position(position) => positions
                .get(position)
                .cloned(),
            Group { name, idx } => groups
                .get(name)
                .and_then(|cells| cells.get(*idx as usize))
                .cloned(),
        }
    }

    fn allocate_index(&mut self) -> u32 {
        let idx = self.next_index;
        // TODO: On wrap, do index compression.
        self.next_index += 1;
        idx
    }

    fn extract_cell_package(&mut self, idx: u32, cell: Option<Cell>)
        -> CellPackage
    {

        let mut refs = Vec::with_capacity(3);
        
        // Gather and remove cell names.
        let mut names = Vec::with_capacity(2);
        for (name, i) in self.names.iter() {
            if *i == idx { names.push(name.clone()) }
        }
        for name in names.into_iter() {
            let _ = self.names.remove(&name);
            refs.push(CellRef::Name(name));
        }

        // Gather and remove cell positions.
        let mut positions = Vec::with_capacity(2);
        for (position, i) in self.positions.iter() {
            if *i == idx { positions.push(*position) }
        }
        for position in positions.into_iter() {
            let _ = self.positions.remove(&position);
            refs.push(CellRef::Position(position));
        }

        // Gather and remove cell groups.
        let mut groups = Vec::with_capacity(2);
        for (name, cells) in self.groups.iter() {
            if cells.contains(&idx) { groups.push(name.clone()) }
        }
        for name in groups.into_iter() {
            if self.groups
                .get(&name)
                .expect("find index in owning group")
                .len() > 1 
            {
                // TODO: Simplify this if Vec::remove_item becomes stable.
                // let _ = self.groups
                //     .get_mut(&name)
                //     .expect("remove index from owning group")
                //     .remove_item(&idx);
                let _ = self.groups
                    .get_mut(&name)
                    .map(|cells| {
                        let pos = cells
                            .iter()
                            .position(|x| *x == idx)
                            .expect("find index in owning group");
                        let _ = cells.remove(pos);
                    });
            } else {
                let _ = self.groups.remove(&name);
            }

            refs.push(CellRef::Group { name, idx });
        }

        CellPackage { cell, idx: Some(idx), refs }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::new()
    }
}


