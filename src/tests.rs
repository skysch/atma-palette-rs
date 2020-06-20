////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette operation test suite.
////////////////////////////////////////////////////////////////////////////////

use crate::Palette;
use crate::cell::Cell;
use crate::cell::CellRef;

/// Tests `Palette::insert_cell` followed by `Palette::remove_cell`.
#[test]
fn cell_insert_remove_method_inverse() {
    let mut pal = Palette::new();
    pal.insert_cell(None, &Cell::default());
    assert!(pal.cell(&CellRef::Index(0)).is_some());
    pal.remove_cell(&CellRef::Index(0));
    assert!(pal.cell(&CellRef::Index(0)).is_none());
}
