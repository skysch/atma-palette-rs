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
use crate::cell::CellPackage;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

/// A palette modifying operation.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum Operation {
    /// An operation which does nothing.
    Null,
    /// Inserts a `Cell` into the palette.
    InsertCell(Cell),
    /// Inserts a `CellPackage` into the palette.
    InsertCellPackage(CellPackage),
    /// Removes a `Cell` from the palette.
    RemoveCell(CellRef),

}