////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Reversable palette operations.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::Cell;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::expr::Expr;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// Operation
////////////////////////////////////////////////////////////////////////////////
/// A primitive, reversable palette operation.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum Operation {
    ////////////////////////////////////////////////////////////////////////////
    // Cell create operations
    ////////////////////////////////////////////////////////////////////////////

    /// Inserts a `Cell` into the palette.
    InsertCell {
        /// The index to insert the cell into.
        idx: u32,
        /// The `Cell` to insert.
        cell: Cell,
    },

    /// Removes the referenced `Cell` from the palette.
    RemoveCell {
        /// A reference to the `Cell` to remove.
        cell_ref: CellRef<'static>,
    },

    ////////////////////////////////////////////////////////////////////////////
    // Name operations
    ////////////////////////////////////////////////////////////////////////////

    /// Assigns a name to a position selector.
    AssignName {
        /// The PositionSelector to assign the name to.
        selector: PositionSelector,
        /// The name to assign.
        name: Cow<'static, str>,
    },

    /// Unassigns a name for a position selector.
    UnassignName {
        /// The PositionSelector to unassign the name from.
        selector: PositionSelector,
        /// The name to unassign.
        name: Cow<'static, str>,
    },

    ////////////////////////////////////////////////////////////////////////////
    // Position operations
    ////////////////////////////////////////////////////////////////////////////
    
    /// Assigns a position to a cell.
    AssignPosition {
        /// A reference to the `Cell` to assign the position to.
        cell_ref: CellRef<'static>,
        /// The position to assign.
        position: Position,
    },

    /// Unassigns a position for a cell.
    UnassignPosition {
        /// A reference to the `Cell` to unassign the position for.
        cell_ref: CellRef<'static>,
        /// The position to unassign.
        position: Position,
    },

    /// Unassigns all positions for a cell.
    ClearPositions {
        /// A reference to the `Cell` to clear the positions for.
        cell_ref: CellRef<'static>,
    },

    ////////////////////////////////////////////////////////////////////////////
    // Group operations
    ////////////////////////////////////////////////////////////////////////////

    /// Assigns a group to a cell.
    AssignGroup {
        /// A reference to the `Cell` to assign the group to.
        cell_ref: CellRef<'static>,
        /// The group to assign.
        group: Cow<'static, str>,
        /// The group index to assign.
        idx: Option<u32>,
    },

    /// Unassigns a group for a cell.
    UnassignGroup {
        /// A reference to the `Cell` to unassign the group for.
        cell_ref: CellRef<'static>,
        /// The group to unassign.
        group: Cow<'static, str>,
    },

    /// Unassigns all groups for a cell.
    ClearGroups {
        /// A reference to the `Cell` to clear the groups for.
        cell_ref: CellRef<'static>,
    },

    ////////////////////////////////////////////////////////////////////////////
    // Expr operations
    ////////////////////////////////////////////////////////////////////////////
    /// Sets the color expression for a cell.
    SetExpr {
        /// A reference to the `Cell` to set the `Expr` for.
        cell_ref: CellRef<'static>,
        /// The expression to set.
        expr: Expr,
    },
}


