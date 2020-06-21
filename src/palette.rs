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
use std::convert::TryInto;



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
    // Accessors
    ////////////////////////////////////////////////////////////////////////////

    /// Retreives a reference to the `Cell` associated with the given `CellRef`.
    pub fn cell(&self, cell_ref: &CellRef) -> Result<&Cell, Error> {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        self.cells
            .get(&idx)
            .ok_or(Error::UnrecognizedCellReference { 
                cell_ref: cell_ref.clone(),
            })
    }

    /// Retreives a mutable reference to the `Cell` associated with the given
    /// `CellRef`.
    pub fn cell_mut(&mut self, cell_ref: &CellRef) -> Result<&mut Cell, Error> {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        self.cells
            .get_mut(&idx)
            .ok_or(Error::UnrecognizedCellReference { 
                cell_ref: cell_ref.clone(),
            })
    }
    
    fn resolve_ref_to_index(
        names: &BTreeMap<String, u32>,
        positions: &BTreeMap<Position, u32>,
        groups: &BTreeMap<String, Vec<u32>>,
        cell_ref: &CellRef)
        -> Result<u32, Error>
    {
        use CellRef::*;
        match cell_ref {
            Index(idx) => Ok(*idx),
            Name(name) => names
                .get(name)
                .cloned()
                .ok_or(Error::UnrecognizedCellReference { 
                    cell_ref: cell_ref.clone(),
                }),
            Position(position) => positions
                .get(position)
                .cloned()
                .ok_or(Error::UnrecognizedCellReference { 
                    cell_ref: cell_ref.clone(),
                }),
            Group { group, idx } => groups
                .get(group)
                .and_then(|cells| cells.get(*idx as usize))
                .cloned()
                .ok_or(Error::UnrecognizedCellReference { 
                    cell_ref: cell_ref.clone(),
                }),
        }
    }

    fn allocate_index(&mut self) -> u32 {
        let idx = self.next_index;
        // TODO: On wrap, do index compression.
        self.next_index += 1;
        idx
    }

    ////////////////////////////////////////////////////////////////////////////
    // Operation-level interface
    ////////////////////////////////////////////////////////////////////////////
    /// Applies an `Operation` to the palette. Returns an `Operation` that will
    /// undo the applied changes.
    pub fn apply_operation(&mut self, op: &Operation) 
        -> Result<Vec<Operation>, Error>
    {
        use Operation::*;
        match op {
            Null => Ok(Vec::new()),
            InsertCell { idx, cell } => self.insert_cell(*idx, cell),
            RemoveCell { cell_ref } => self.remove_cell(cell_ref),

            _ => unimplemented!(),
        }
    }

    /// Inserts a `Cell` into the palette at the given index.
    pub fn insert_cell(&mut self, idx: Option<u32>, cell: &Cell)
        -> Result<Vec<Operation>, Error>
    {
        let idx = idx.unwrap_or_else(|| self.allocate_index());
        match self.cells.insert(idx, *cell) {
            // No cell was replaced.
            None => Ok(vec![
                Operation::RemoveCell {
                    cell_ref: CellRef::Index(idx)
                },
            ]),
            // A cell was replaced.
            Some(old) => Ok(vec![
                Operation::InsertCell { 
                    idx: Some(idx),
                    cell: old,
                },
            ]),
        }
    }

    /// Removes a `Cell` from the palette.
    pub fn remove_cell(&mut self, cell_ref: &CellRef)
        -> Result<Vec<Operation>, Error> 
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;
        
        match self.cells.remove(&idx) {
            // Cell was removed.
            Some(cell) => Ok(vec![
                Operation::InsertCell { idx: Some(idx), cell }
            ]),

            // Cell is already missing.
            None => Ok(Vec::new()),
        }
    }

    /// Assigns a name to a cell.
    pub fn assign_name(&mut self, cell_ref: &CellRef, name: String)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        match self.names.insert(name.clone(), idx) {
            Some(old_idx) => Ok(vec![
                Operation::AssignName {
                    cell_ref: CellRef::Index(old_idx),
                    name: name,
                },
            ]),
            None => Ok(vec![
                Operation::UnassignName {
                    cell_ref: CellRef::Index(idx),
                    name: name,
                },
            ]),
        }
    }

    /// Unassigns a name for a cell.
    pub fn unassign_name(&mut self, cell_ref: &CellRef, name: String)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;
        
        match self.names.get(&name) {
            Some(cur_idx) if *cur_idx == idx => {
                let _ = self.names.remove(&name);
                Ok(vec![
                    Operation::AssignName {
                        cell_ref: CellRef::Index(idx),
                        name,
                    },
                ])
            },
            _ => Ok(Vec::new()),
        }
    }

    /// Unassigns a name for a cell.
    pub fn clear_names(&mut self, cell_ref: &CellRef)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        // TODO: Use BTreeMap::drain_filter when it becomes stable.
        let mut to_remove = Vec::with_capacity(1);

        for (name, cur_idx) in self.names.iter() {
            if *cur_idx == idx { to_remove.push(name.clone()); }
        }

        let mut ops = Vec::with_capacity(to_remove.len());
        for name in to_remove.into_iter() {
            let _ = self.names.remove(&name);
            ops.push(Operation::AssignName {
                cell_ref: CellRef::Index(idx),
                name,
            });
        }      

        Ok(ops)
    }

    /// Assigns a position to a cell.
    pub fn assign_position(&mut self, cell_ref: &CellRef, position: Position)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        match self.positions.insert(position.clone(), idx) {
            Some(old_idx) => Ok(vec![
                Operation::AssignPosition {
                    cell_ref: CellRef::Index(old_idx),
                    position: position,
                },
            ]),
            None => Ok(vec![
                Operation::UnassignPosition {
                    cell_ref: CellRef::Index(idx),
                    position: position,
                },
            ]),
        }
    }

    /// Unassigns a position for a cell.
    pub fn unassign_position(&mut self, cell_ref: &CellRef, position: Position)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;
        
        match self.positions.get(&position) {
            Some(cur_idx) if *cur_idx == idx => {
                let _ = self.positions.remove(&position);
                Ok(vec![
                    Operation::AssignPosition {
                        cell_ref: CellRef::Index(idx),
                        position,
                    },
                ])
            },
            _ => Ok(Vec::new()),
        }
    }

    /// Unassigns a position for a cell.
    pub fn clear_positions(&mut self, cell_ref: &CellRef)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        // TODO: Use BTreeMap::drain_filter when it becomes stable.
        let mut to_remove = Vec::with_capacity(1);

        for (position, cur_idx) in self.positions.iter() {
            if *cur_idx == idx { to_remove.push(position.clone()); }
        }

        let mut ops = Vec::with_capacity(to_remove.len());
        for position in to_remove.into_iter() {
            let _ = self.positions.remove(&position);
            ops.push(Operation::AssignPosition {
                cell_ref: CellRef::Index(idx),
                position,
            });
        }      

        Ok(ops)
    }

    /// Assigns a group to a cell.
    pub fn assign_group(
        &mut self,
        cell_ref: &CellRef,
        group: String,
        group_idx: Option<u32>)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        let members = self.groups.entry(group.clone()).or_default();
        let members_len: u32 = members.len()
            .try_into()
            .expect("convert usize to u32");
        let group_idx = group_idx.unwrap_or(members_len);
        
        if group_idx <= members_len {    
            let group_idx_usize: usize = group_idx.try_into()
                .expect("convert u32 to usize");
            
            members.insert(group_idx_usize, idx);
            Ok(vec![
                Operation::UnassignGroup { 
                    cell_ref: cell_ref.clone(),
                    group,
                },
            ])
        } else {
            if members_len == 0 {
            // Remove the empty group that we probably just added.
            let _ = self.groups.remove(&group);
            }
            Err(Error::GroupIndexOutOfBounds {
                group,
                index: group_idx,
                max: members_len,
            })
        }
    }

    /// Unassigns a group for a cell.
    pub fn unassign_group(
        &mut self,
        cell_ref: &CellRef,
        group: String)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;
        
        let res = match self.groups.get_mut(&group) {
            Some(members) => match members.iter().position(|x| *x == idx) {
                Some(group_idx) => {
                    let _ = members.remove(group_idx);
                    Ok(vec![
                        Operation::AssignGroup {
                            cell_ref: CellRef::Index(idx),
                            group: group.clone(),
                            idx: Some(group_idx
                                .try_into()
                                .expect("convert usize to u32")),
                        },
                    ])

                }
                None => Ok(Vec::new()),
            },
            None => Ok(Vec::new()),
        };

        if self.groups.get(&group).map(Vec::is_empty).unwrap_or(false) {
            let _ = self.groups.remove(&group);
        }

        res
    }

    /// Removes the cell from all groups.
    pub fn clear_groups(&mut self, cell_ref: &CellRef)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)?;

        // TODO: Consider using BTreeMap::drain_filter when it becomes stable.
        let mut empty_groups = Vec::new();
        let mut ops = Vec::new();
        for (group, members) in self.groups.iter_mut() {
            if let Some(group_idx) = members.iter().position(|x| *x == idx) {
                let _ = members.remove(group_idx);
                ops.push(Operation::AssignGroup {
                    cell_ref: CellRef::Index(idx),
                    group: group.clone(),
                    idx: Some(group_idx
                        .try_into()
                        .expect("convert usize to u32")),
                });
            }

            if members.is_empty() {
                empty_groups.push(group.clone())
            }
        }

        for group in empty_groups.into_iter() {
            let _ = self.groups.remove(&group);
        }

        Ok(ops)
    }

}

impl Default for Palette {
    fn default() -> Self {
        Palette::new()
    }
}


