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
}


