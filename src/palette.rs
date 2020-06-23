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
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;



/// The Atma palette object.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize)]
pub struct Palette {
    /// Palette cells storage. Holds cells containing color expressions.
    cells: BTreeMap<u32, Cell>,
    /// The next free cell index.
    next_index: u32,
    /// A map of names assigned to cells.
    names: BTreeMap<Cow<'static, str>, u32>,
    /// A map of (page, line) positions assigned to cells.
    positions: BTreeMap<Position, u32>,
    /// A map of names assigned to groups of cells.
    groups: BTreeMap<Cow<'static, str>, Vec<u32>>,
}


impl Palette {

    ////////////////////////////////////////////////////////////////////////////
    // Constructors
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
    pub fn cell<'name>(&self, cell_ref: CellRef<'name>)
        -> Result<&Cell, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

        self.cells
            .get(&idx)
            .ok_or(Error::UndefinedCellReference { 
                cell_ref: cell_ref.into_static(),
            })
    }

    /// Retreives a mutable reference to the `Cell` associated with the given
    /// `CellRef`.
    pub fn cell_mut<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<&mut Cell, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

        self.cells
            .get_mut(&idx)
            .ok_or(Error::UndefinedCellReference { 
                cell_ref: cell_ref.into_static(),
            })
    }
    
    fn resolve_ref_to_index<'name>(
        names: &BTreeMap<Cow<'static, str>, u32>,
        positions: &BTreeMap<Position, u32>,
        groups: &BTreeMap<Cow<'static, str>, Vec<u32>>,
        cell_ref: &CellRef<'name>)
        -> Result<u32, Error>
    {
        use CellRef::*;
        match cell_ref {
            Index(idx) => Ok(*idx),

            Name(name) => names
                .get(&*name)
                .cloned()
                .ok_or(Error::UndefinedCellReference { 
                    cell_ref: cell_ref.clone().into_static(),
                }),

            Position(position) => positions
                .get(position)
                .cloned()
                .ok_or(Error::UndefinedCellReference { 
                    cell_ref: cell_ref.clone().into_static(),
                }),

            Group { group, idx } => groups
                .get(&*group)
                .and_then(|cells| cells.get(*idx as usize))
                .cloned()
                .ok_or(Error::UndefinedCellReference { 
                    cell_ref: cell_ref.clone().into_static(),
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
    // Composite operation interface
    ////////////////////////////////////////////////////////////////////////////
    
    /// Applies a sequence of `Operation`s to the palette.
    ///
    /// The applied operations' undo ops will be grouped together and inserted
    /// into the provided `History`.
    ///
    /// ### Parameters
    /// + `op`: The operation to apply.
    /// + `history`: The operation history.
    pub fn apply_operations(
        &mut self,
        ops: &[Operation],
        history: Option<&mut History>)
        -> Result<(), Error>
    {
        if let Some(history) = history {
            let mut undo_ops = Vec::with_capacity(ops.len());
            for op in ops {
                undo_ops.extend(self.apply_operation(op)?);
            }
            history.push_undo_ops(undo_ops);
        } else {
            for op in ops {
                let _ = self.apply_operation(op)?;
            }
        }
        Ok(())
    }

    /// Unapplies the latest set of operations recorded in the given `History`.
    ///
    /// Returns the number of undo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer undo operations
    /// recorded than requested.
    ///
    /// ### Panics
    /// 
    /// This method assumes that history and palette are synchronized. If they
    /// are not, (e.g., because the palette was modified without providing the
    /// exact history being used,) then the undo operation may panic if a
    /// previously recorded undo action cannot be applied to the palette in its
    /// current state.
    pub fn undo(&mut self, history: &mut History, count: usize) -> usize {
        let mut real_count = 0;
        for _ in 0..count {
            history.undo_with(|undo_ops| {
                let mut redo_ops = Vec::with_capacity(undo_ops.len());
                for op in undo_ops {
                    redo_ops.extend(self.apply_operation(op)
                        .expect("undo from valid state"));
                }
                real_count += 1;
                redo_ops
            });
        }
        real_count
    }

    /// Reapplies the latest set of undone operations, as recorded in the given
    /// `History`.
    ///
    /// Returns the number of redo operations successfully performed. This may
    /// be fewer than the number provided if there are fewer redo operations
    /// recorded than requested.
    ///
    /// ### Panics
    /// 
    /// This method assumes that history and palette are synchronized. If they
    /// are not, (e.g., because the palette was modified without providing the
    /// exact history being used,) then the redo operation may panic if a
    /// previously recorded redo action cannot be applied to the palette in its
    /// current state.
    pub fn redo(&mut self, history: &mut History, count: usize) -> usize {
        let mut real_count = 0;
        for _ in 0..count {
            history.redo_with(|redo_ops| {
                let mut undo_ops = Vec::with_capacity(redo_ops.len());
                for op in redo_ops {
                    undo_ops.extend(self.apply_operation(op)
                        .expect("redo from valid state"));
                }
                real_count += 1;
                undo_ops
            });
        }
        real_count
    }


    ////////////////////////////////////////////////////////////////////////////
    // Primitive operation interface
    ////////////////////////////////////////////////////////////////////////////

    /// Applies an `Operation` to the palette. Returns an `Operation` that will
    /// undo the applied changes.
    ///
    /// ### Parameters
    /// + `op`: The operation to apply.
    pub fn apply_operation(&mut self, op: &Operation) 
        -> Result<Vec<Operation>, Error>
    {
        use Operation::*;
        match op {
            InsertCell { idx, cell }
                => self.insert_cell(*idx, cell),
            RemoveCell { cell_ref }
                => self.remove_cell(cell_ref.clone()),

            AssignName { cell_ref, name } 
                => self.assign_name(cell_ref.clone(), name.clone()),
            UnassignName { cell_ref, name } 
                => self.unassign_name(cell_ref.clone(), name.clone()),
            ClearNames { cell_ref } 
                => self.clear_names(cell_ref.clone()),

            AssignPosition { cell_ref, position } 
                => self.assign_position(cell_ref.clone(), position.clone()),
            UnassignPosition { cell_ref, position } 
                => self.unassign_position(cell_ref.clone(), position.clone()),
            ClearPositions { cell_ref } 
                => self.clear_positions(cell_ref.clone()),

            AssignGroup { cell_ref, group, idx } 
                => self.assign_group(cell_ref.clone(), group.clone(), *idx),
            UnassignGroup { cell_ref, group } 
                => self.unassign_group(cell_ref.clone(), group.clone()),
            ClearGroups { cell_ref } 
                => self.clear_groups(cell_ref.clone()),

            SetExpr { cell_ref, expr }
                => self.set_expr(cell_ref.clone(), expr.clone()),
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
    pub fn remove_cell<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<Vec<Operation>, Error> 
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;
        
        match self.cells.remove(&idx) {
            // Cell was removed.
            Some(cell) => Ok(vec![
                Operation::InsertCell { idx: Some(idx), cell },
            ]),

            // Cell is already missing.
            None => Ok(Vec::new()),
        }
    }

    /// Assigns a name to a cell.
    pub fn assign_name<'name, T>(
        &mut self,
        cell_ref: CellRef<'name>,
        name: T)
        -> Result<Vec<Operation>, Error>
        where T: Into<Cow<'static, str>>
    {
        let name = name.into();
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

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
    pub fn unassign_name<'name, T>(
        &mut self,
        cell_ref: CellRef<'name>,
        name: T)
        -> Result<Vec<Operation>, Error>
        where T: Into<Cow<'static, str>>
    {
        let name = name.into();
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;
        
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
    pub fn clear_names<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

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
    pub fn assign_position<'name>(
        &mut self,
        cell_ref: CellRef<'name>,
        position: Position)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

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
    pub fn unassign_position<'name>(
        &mut self,
        cell_ref: CellRef<'name>,
        position: Position)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;
        
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
    pub fn clear_positions<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

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
    pub fn assign_group<'name, T>(
        &mut self,
        cell_ref: CellRef<'name>,
        group: T,
        group_idx: Option<u32>)
        -> Result<Vec<Operation>, Error>
        where T: Into<Cow<'static, str>>
    {
        let group = group.into();
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

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
                    cell_ref: CellRef::Index(idx),
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
    pub fn unassign_group<'name, T>(
        &mut self,
        cell_ref: CellRef<'name>,
        group: T)
        -> Result<Vec<Operation>, Error>
        where T: Into<Cow<'static, str>>
    {
        let group = group.into();
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;
        
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
    pub fn clear_groups<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

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

    /// Sets the color expression for a `Cell`.
    pub fn set_expr<'name>(&mut self, cell_ref: CellRef<'name>, expr: Expr)
        -> Result<Vec<Operation>, Error>
    {
        let idx = Palette::resolve_ref_to_index(
            &self.names,
            &self.positions,
            &self.groups,
            &cell_ref)?;

        let cell = self.cells.get_mut(&idx)
            .expect("retreive resolved cell");

        let old = std::mem::replace(cell.expr_mut(), expr);

        Ok(vec![
            Operation::SetExpr {
                cell_ref: CellRef::Index(idx),
                expr: old,
            }
        ])
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::new()
    }
}
