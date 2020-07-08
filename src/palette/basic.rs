////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! BasicPalette data.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::Cell;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::color::Color;
use crate::error::PaletteError;
use crate::error::FileError;
use crate::error::FileErrorContext as _;
use crate::palette::Expr;
use crate::palette::History;
use crate::palette::Operation;
use crate::utility::Few;
use crate::utility::split_intersect;

// External library imports.
use bimap::BiBTreeMap;
use serde::Deserialize;
use serde::Serialize;
use ron::ser::PrettyConfig;
use ron::ser::to_string_pretty;

// Standard library imports.
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;



////////////////////////////////////////////////////////////////////////////////
// BasicPalette
////////////////////////////////////////////////////////////////////////////////
/// The Atma palette object.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize)]
pub struct BasicPalette {
    /// BasicPalette cells storage. Holds cells containing color expressions.
    cells: BTreeMap<u32, Cell>,
    /// The next free cell index.
    next_index: u32,
    /// A map of assigned names.
    names: BiBTreeMap<Cow<'static, str>, PositionSelector>,
    /// A map of positions assigned to cells.
    positions: BTreeMap<Position, u32>,
    /// A map of names assigned to groups of cells.
    groups: BTreeMap<Cow<'static, str>, Vec<u32>>,
}


impl BasicPalette {

    ////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////
    
    /// Constructs a new `BasicPalette`.
    pub fn new() -> Self {
        BasicPalette {
            cells: BTreeMap::new(),
            next_index: 0,
            names: BiBTreeMap::new(),
            positions: BTreeMap::new(),
            groups: BTreeMap::new(),
        }
    }

    /// Constructs a new `BasicPalette` by parsing data from the file at the given
    /// path.
    pub fn read_from_path<P>(path: &P) -> Result<Self, FileError>
        where P: AsRef<Path> + Debug
    {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .with_context(|| format!("Failed to open file {:?}", path))?;
        BasicPalette::read_from_file(&mut file)
    }

    /// Constructs a new `BasicPalette` by parsing data from the given file.
    pub fn read_from_file(file: &mut File) -> Result<Self, FileError> {
        BasicPalette::parse_ron_from_file(file)
    }

    /// Parses a `BasicPalette` from a file using the RON format.
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
        let palette = BasicPalette::deserialize(&mut d)
            .context("Failed parsing RON file")?;
        d.end()
            .context("Failed parsing RON file")?;
        Ok(palette)
    }

    /// Writes the `BasicPalette` to the file at the given path.
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

    /// Writes the `BasicPalette` to the given file.
    pub fn write_to_file(&self, file: &mut File) -> Result<(), FileError> {
        self.generate_ron_into_file(file)
    }

    /// Generates a RON formatted `BasicPalette` by serializing into the given file.
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
    /// Retreives a copy of the color associated with the given `CellRef`.
    pub fn color<'name>(&self, cell_ref: &CellRef<'name>)
        -> Result<Option<Color>, PaletteError>
    {
        let mut index_list = HashSet::new();
        self.cycle_detect_color(cell_ref, &mut index_list)
    }

    /// Retreives a copy of the color associated with the given `CellRef`.
    pub(in crate) fn cycle_detect_color<'name>(
        &self,
        cell_ref: &CellRef<'name>,
        index_list: &mut HashSet<u32>)
        -> Result<Option<Color>, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, cell_ref)?;
        if index_list.contains(&idx) {
            return Err(PaletteError::UndefinedColor {
                cell_ref: cell_ref.clone().into_static(),
                circular: true,
            });
        }
        let _ = index_list.insert(idx);

        self.cells
            .get(&idx)
            .ok_or(PaletteError::UndefinedColor { 
                cell_ref: cell_ref.clone().into_static(),
                circular: false,
            })
            .and_then(|cell| cell.color(self, index_list))
    }

    /// Retreives a reference to the `Cell` associated with the given `CellRef`.
    pub fn cell<'name>(&self, cell_ref: &CellRef<'name>)
        -> Result<&Cell, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, cell_ref)?;

        self.cells
            .get(&idx)
            .ok_or(PaletteError::UndefinedCellReference { 
                cell_ref: cell_ref.clone().into_static(),
            })
    }

    /// Retreives a mutable reference to the `Cell` associated with the given
    /// `CellRef`.
    pub fn cell_mut<'name>(&mut self, cell_ref: &CellRef<'name>)
        -> Result<&mut Cell, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, cell_ref)?;

        self.cells
            .get_mut(&idx)
            .ok_or(PaletteError::UndefinedCellReference { 
                cell_ref: cell_ref.clone().into_static(),
            })
    }
    
    /// Resolves a `CellRef` to its index in the palette.
    pub fn resolve_ref_to_index<'name>(&self, cell_ref: &CellRef<'name>)
        -> Result<u32, PaletteError>
    {
        BasicPalette::resolve_ref_to_index_using(
            &self.names,
            &self.positions,
            &self.groups,
            cell_ref)
    }

    fn resolve_ref_to_index_using<'name>(
        names: &BiBTreeMap<Cow<'static, str>, PositionSelector>,
        positions: &BTreeMap<Position, u32>,
        groups: &BTreeMap<Cow<'static, str>, Vec<u32>>,
        cell_ref: &CellRef<'name>)
        -> Result<u32, PaletteError>
    {
        match cell_ref {
            CellRef::Index(idx) => Ok(*idx),

            CellRef::Name(name) => names
                .get_by_left(&*name)
                .and_then(|pos_sel| {
                    match Position::try_from(pos_sel.clone()) {
                        Err(_) => None,
                        Ok(pos) => positions.get(&pos).cloned(),
                    }
                })
                .ok_or(PaletteError::UndefinedCellReference { 
                    cell_ref: cell_ref.clone().into_static(),
                }),


            CellRef::Position(position) => positions
                .get(position)
                .cloned()
                .ok_or(PaletteError::UndefinedCellReference { 
                    cell_ref: cell_ref.clone().into_static(),
                }),

            CellRef::Group { group, idx } => groups
                .get(&*group)
                .and_then(|cells| cells.get(*idx as usize))
                .cloned()
                .ok_or(PaletteError::UndefinedCellReference { 
                    cell_ref: cell_ref.clone().into_static(),
                }),
        }
    }

    /// Returns the given index if it is unoccupied, or the next unoccupied
    /// index after it.
    pub fn unoccupied_index_or_next(&mut self, from: u32) -> Option<u32> {
        let mut next = from;
        while self.is_occupied_index(&next) {
            next = next.wrapping_add(1);
            // Check if we've looped all the way around.
            if next == from { return None; }
        }
        Some(next)
    }

    /// Returns the given position if it is unoccupied, or the next unoccupied
    /// position after it.
    pub fn unoccupied_position_or_next(&self, from: Position)
        -> Option<Position>
    {
        let mut next = from;
        while self.is_occupied_position(&next) {
            next = next.wrapping_succ();
            // Check if we've looped all the way around.
            if next == from { return None; }
        }
        Some(next)
    }


    ////////////////////////////////////////////////////////////////////////////
    // Range and usability queries
    ////////////////////////////////////////////////////////////////////////////

    /// Returns true if the given index is occupied in the palette.
    pub fn is_occupied_index(&self, idx: &u32) -> bool {
        self.cells.get(idx).is_some()
    }

    /// Returns the full range of occupied indices in the palette, or None if
    /// the palette is empty.
    pub(in crate) fn occupied_index_range(&self) -> Few<u32> {
        let mut keys = self.cells.keys();
        match (keys.next(), keys.next_back()) {
            (Some(first), Some(last)) => Few::Two(*first, *last),
            (Some(first), None)       => Few::One(*first),
            (None, _)                 => Few::Zero,
        }
    }

    pub(in crate) fn occupied_index_subrange(&self, low: u32, high: u32)
        -> Few<u32>
    {
        if low > high { return Few::Zero }
        
        let mut range = self.cells.range(low..=high).map(|(k, _v)| k);
        match (range.next(), range.next_back()) {
            (Some(first), Some(last)) => Few::Two(*first, *last),
            (Some(first), None)       => Few::One(*first),
            (None, _)                 => Few::Zero,
        }
    }


    /// Returns true if the given name is assigned in the palette.
    pub fn is_assigned_name(&self, name: &str) -> bool {
        self.names
            .get_by_left(&Cow::Borrowed(name))
            .is_some()
    }

    /// Returns true if the given name is occupied in the palette.
    pub fn is_occupied_name(&self, name: &str) -> bool {
        self.names
            .get_by_left(&Cow::Borrowed(name))
            .and_then(|pos_sel| {
                match Position::try_from(pos_sel.clone()) {
                    Err(_) => None,
                    Ok(pos) => self.positions.get(&pos).cloned(),
                }
            })
            .and_then(|idx| self.cells.get(&idx))
            .is_some()
    }

    /// Returns the index associated with the given name if it is occupied.
    pub fn resolve_name_if_occupied(&self, name: &str) -> Option<u32> {
        self.names
            .get_by_left(&Cow::Borrowed(name))
            .and_then(|pos_sel| {
                match Position::try_from(pos_sel.clone()) {
                    Err(_) => None,
                    Ok(pos) => self.positions.get(&pos).cloned(),
                }
            })
            .and_then(|idx| if self.cells.contains_key(&idx) {
                Some(idx)
            } else {
                None
            })
    }


    /// Returns true if the given group index is assigned in the palette.
    pub fn is_assigned_group(&self, group: &str, idx: u32) -> bool {
        self.groups
            .get(group)
            .map(|elems| usize::try_from(idx).unwrap() < elems.len())
            .unwrap_or(false)
    }

    /// Returns true if the given group index is occupied in the palette.
    pub fn is_occupied_group(&self, group: &str, idx: u32) -> bool {
        self.groups
            .get(group)
            .and_then(|elems| elems
                .get(usize::try_from(idx).unwrap())
                .and_then(|cell_idx| self.cells.get(cell_idx)))
            .is_some()
    }

    /// Returns the full range of assigned indexes for a group in the palette,
    /// or None if the group is empty.
    pub(in crate) fn assigned_group_range(&self, group: &str) -> Few<u32> {
        match self.groups.get(group) {
            None                            => Few::Zero,
            Some(elems) if elems.is_empty() => Few::Zero,
            Some(elems) if elems.len() == 1 => Few::One(0),
            Some(elems)                     => Few::Two(0,
                (elems.len() - 1)
                    .try_into()
                    .expect("to many elements in group")),
        }
    }

    pub(in crate) fn assigned_group_subrange(
        &self,
        group: &str,
        low: u32,
        high: u32)
        -> Few<u32>
    {
        match self.groups.get(group) {
            None                            => Few::Zero,
            Some(elems) if elems.is_empty() => Few::Zero,
            Some(elems)                     => {
                let max: u32 = (elems.len() - 1)
                    .try_into()
                    .expect("to many elements in group");
                split_intersect((low, high), (0, max))
            },
        }
    }

    /// Returns the index associated with the given group if it is occupied.
    pub fn resolve_group_if_occupied(&self, group: &str, idx: u32)
        -> Option<u32>
    {
        self.groups
            .get(group)
            .and_then(|elems| elems.get(usize::try_from(idx).unwrap()))
            .and_then(|idx| if self.cells.contains_key(idx) {
                Some(*idx)
            } else {
                None
            })
    }

    /// Returns true if the given position is assigned in the palette.
    pub fn is_assigned_position(&self, pos: &Position) -> bool {
        self.positions
            .get(pos)
            .is_some()
    }

    /// Returns true if the given position is occupied in the palette.
    pub fn is_occupied_position(&self, pos: &Position) -> bool {
        self.positions
            .get(pos)
            .and_then(|idx| self.cells.get(idx))
            .is_some()
    }


    /// Returns the full range of assigned positions in the palette, or None if
    /// no positions are assigned is empty.
    #[allow(unused)]
    pub(in crate) fn assigned_position_range(&self) -> Few<Position> {
        let mut keys = self.positions.keys();
        match (keys.next(), keys.next_back()) {
            (Some(first), Some(last)) => Few::Two(*first, *last),
            (Some(first), None)       => Few::One(*first),
            (None, _)                 => Few::Zero,
        }
    }

    pub(in crate) fn assigned_position_subrange(&self,
        low: Position,
        high: Position)
        -> Few<Position>
    {
        if low > high { return Few::Zero }
        
        let mut range = self.positions.range(low..=high).map(|(k, _v)| k);
        match (range.next(), range.next_back()) {
            (Some(first), Some(last)) => Few::Two(*first, *last),
            (Some(first), None)       => Few::One(*first),
            (None, _)                 => Few::Zero,
        }
    }

    /// Returns the index associated with the given position if it is occupied.
    pub fn resolve_position_if_occupied(&self, position: &Position)
        -> Option<u32>
    {
        self.positions
            .get(position)
            .and_then(|idx| if self.cells.contains_key(idx) {
                Some(*idx)
            } else {
                None
            })
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
        -> Result<(), PaletteError>
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
        -> Result<Vec<Operation>, PaletteError>
    {
        use Operation::*;
        match op {
            InsertCell { idx, cell }
                => self.insert_cell(*idx, cell.clone()),
            RemoveCell { cell_ref }
                => self.remove_cell(cell_ref.clone()),

            AssignName { selector, name } 
                => self.assign_name(name.clone(), selector.clone()),
            UnassignName { selector, name } 
                => self.unassign_name(selector.clone(), name.clone()),

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
    pub fn insert_cell(&mut self, idx: u32, cell: Cell)
        -> Result<Vec<Operation>, PaletteError>
    {
        match self.cells.insert(idx, cell) {
            // No cell was replaced.
            None => Ok(vec![
                Operation::RemoveCell {
                    cell_ref: CellRef::Index(idx)
                },
            ]),
            // A cell was replaced.
            Some(old) => Ok(vec![
                Operation::InsertCell { idx, cell: old },
            ]),
        }
    }

    /// Removes a `Cell` from the palette.
    pub fn remove_cell<'name>(&mut self, cell_ref: CellRef<'name>)
        -> Result<Vec<Operation>, PaletteError> 
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;
        
        match self.cells.remove(&idx) {
            // Cell was removed.
            Some(cell) => Ok(vec![
                Operation::InsertCell { idx, cell },
            ]),

            // Cell is already missing.
            None => Ok(Vec::new()),
        }
    }

    /// Assigns a name to a position.
    pub fn assign_name<T>(
        &mut self,
        name: T,
        selector: PositionSelector)
        -> Result<Vec<Operation>, PaletteError>
        where T: Into<Cow<'static, str>>
    {
        let name = name.into();

        use bimap::Overwritten::*;
        match self.names.insert(name.clone(), selector) {
            Left(old_name, old_selector) |
            Right(old_name, old_selector) |
            Pair(old_name, old_selector) => Ok(vec![
                Operation::AssignName {
                    selector: old_selector,
                    name: old_name,
                },
            ]),
            Both(
                (old_name_a, old_selector_a),
                (old_name_b, old_selector_b)) => 
            {  
                Ok(vec![
                    Operation::AssignName {
                        selector: old_selector_a,
                        name: old_name_a,
                    },
                    Operation::AssignName {
                        selector: old_selector_b,
                        name: old_name_b,
                    },
                ])
            },
            Neither => Ok(vec![
                Operation::UnassignName {
                    selector,
                    name,
                },
            ]),
        }
    }

    /// Unassigns a name for a cell.
    pub fn unassign_name<'name, T>(
        &mut self,
        selector: PositionSelector,
        name: T)
        -> Result<Vec<Operation>, PaletteError>
        where T: Into<Cow<'static, str>>
    {
        let name = name.into();
        
        match self.names.get_by_left(&name) {
            Some(cur_selector) if *cur_selector == selector => {
                let _ = self.names.remove_by_left(&name);
                Ok(vec![
                    Operation::AssignName {
                        selector,
                        name,
                    },
                ])
            },
            _ => Ok(Vec::new()),
        }
    }


    /// Assigns a position to a cell.
    pub fn assign_position<'name>(
        &mut self,
        cell_ref: CellRef<'name>,
        position: Position)
        -> Result<Vec<Operation>, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;

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
        -> Result<Vec<Operation>, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;
        
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
        -> Result<Vec<Operation>, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;

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
        -> Result<Vec<Operation>, PaletteError>
        where T: Into<Cow<'static, str>>
    {
        let group = group.into();
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;

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
            Err(PaletteError::GroupIndexOutOfBounds {
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
        -> Result<Vec<Operation>, PaletteError>
        where T: Into<Cow<'static, str>>
    {
        let group = group.into();
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;
        
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
        -> Result<Vec<Operation>, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;

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
        -> Result<Vec<Operation>, PaletteError>
    {
        let idx = BasicPalette::resolve_ref_to_index(&self, &cell_ref)?;

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

impl Default for BasicPalette {
    fn default() -> Self {
        BasicPalette::new()
    }
}
