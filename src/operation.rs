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

// External library imports.
use serde::Serialize;
use serde::Deserialize;


////////////////////////////////////////////////////////////////////////////////
// Operation
////////////////////////////////////////////////////////////////////////////////
/// A palette modifying operation.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum Operation {
    /// An operation which does nothing.
    Null,

    ////////////////////////////////////////////////////////////////////////////
    // Cell create operations
    ////////////////////////////////////////////////////////////////////////////
    

    /// Inserts a `Cell` into the palette.
    InsertCell {
        /// The index to insert the cell into, or None if a new one is to be
        /// allocated.
        idx: Option<u32>,
        /// The `Cell` to insert.
        cell: Cell,
    },

    /// Removes the referenced `Cell` from the palette.
    RemoveCell {
        /// A reference to the `Cell` to remove.
        cell_ref: CellRef
    },

    ////////////////////////////////////////////////////////////////////////////
    // Name operations
    ////////////////////////////////////////////////////////////////////////////

    /// Assigns a name to a cell.
    AssignName {
        /// A reference to the `Cell` to assign the name to.
        cell_ref: CellRef,
        /// The name to assign.
        name: String,
    },

    /// Unassigns a name for a cell.
    UnassignName {
        /// A reference to the `Cell` to unassign the name for.
        cell_ref: CellRef,
        /// The name to unassign.
        name: String,
    },

    /// Unassigns all name for a cell.
    ClearNames {
        /// A reference to the `Cell` to clear the names for.
        cell_ref: CellRef,
    },

    ////////////////////////////////////////////////////////////////////////////
    // Position operations
    ////////////////////////////////////////////////////////////////////////////
    
    /// Assigns a position to a cell.
    AssignPosition {
        /// A reference to the `Cell` to assign the position to.
        cell_ref: CellRef,
        /// The position to assign.
        position: Position,
    },

    /// Unassigns a position for a cell.
    UnassignPosition {
        /// A reference to the `Cell` to unassign the position for.
        cell_ref: CellRef,
        /// The position to unassign.
        position: Position,
    },

    /// Unassigns all position for a cell.
    ClearPositions {
        /// A reference to the `Cell` to clear the names for.
        cell_ref: CellRef,
    },
}


