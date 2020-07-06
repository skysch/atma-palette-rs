////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! BasicPalette primitive operations test suite.
////////////////////////////////////////////////////////////////////////////////
#![allow(unused_must_use)]

// Local imports.
use crate::basic::BasicPalette;
use crate::cell::Cell;
use crate::cell::Position;
use crate::cell::CellRef;

// Standard library imports.
use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////
// Basic cell operations
////////////////////////////////////////////////////////////////////////////////

/// Tests `BasicPalette::insert_cell` followed by `BasicPalette::remove_cell`.
#[test]
fn cell_insert_remove_method_inverse() {
    let mut pal = BasicPalette::new();
    pal.insert_cell(0, Cell::default());
    assert!(pal.cell(&CellRef::Index(0)).is_ok());
    pal.remove_cell(CellRef::Index(0));
    assert!(pal.cell(&CellRef::Index(0)).is_err());
}


// /// Tests `BasicPalette::assign_name` followed by `BasicPalette::unassign_name`.
// #[test]
// fn cell_assign_unassign_name_method_inverse() {
//     let mut pal = BasicPalette::new();
//     pal.insert_cell(0, Cell::default());

//     let name: Cow<'_, _> = "TestName".into();

//     pal.assign_name(CellRef::Index(0), name.clone());
//     assert!(pal.cell(&CellRef::Name(name.clone())).is_ok());

//     pal.unassign_name(CellRef::Index(0), name.clone());
//     assert!(pal.cell(&CellRef::Name(name.clone())).is_err());
// }




/// Tests `BasicPalette::assign_position` followed by
/// `BasicPalette::unassign_position`.
#[test]
fn cell_assign_unassign_position_method_inverse() {
    let mut pal = BasicPalette::new();
    pal.insert_cell(0, Cell::default());

    let position = Position { page: 0, line: 10, column: 0 };

    pal.assign_position(CellRef::Index(0), position.clone());
    assert!(pal.cell(&CellRef::Position(position.clone())).is_ok());

    pal.unassign_position(CellRef::Index(0), position.clone());
    assert!(pal.cell(&CellRef::Position(position.clone())).is_err());
}


/// Tests `BasicPalette::assign_position` followed by
/// `BasicPalette::clear_positions`.
#[test]
fn cell_assign_clear_position_method_inverse() {
    let mut pal = BasicPalette::new();
    pal.insert_cell(0, Cell::default());

    let position1 = Position { page: 0, line: 10, column: 0 };
    let position2 = Position { page: 1, line: 4, column: 3 };

    pal.assign_position(CellRef::Index(0), position1.clone());
    pal.assign_position(CellRef::Index(0), position2.clone());
    assert!(pal.cell(&CellRef::Position(position1.clone())).is_ok());
    assert!(pal.cell(&CellRef::Position(position2.clone())).is_ok());

    pal.clear_positions(CellRef::Index(0));
    assert!(pal.cell(&CellRef::Position(position1.clone())).is_err());
    assert!(pal.cell(&CellRef::Position(position2.clone())).is_err());
}

/// Tests `BasicPalette::assign_group` followed by
/// `BasicPalette::unassign_group`.
#[test]
fn cell_assign_unassign_group_method_inverse() {
    let mut pal = BasicPalette::new();
    pal.insert_cell(0, Cell::default());

    let group: Cow<'_, _> = "TestGroup".into();

    pal.assign_group(CellRef::Index(0), group.clone(), None);
    assert!(pal.cell(&CellRef::Group { group: group.clone(), idx: 0 })
        .is_ok());

    pal.unassign_group(CellRef::Index(0), group.clone());
    assert!(pal.cell(&CellRef::Group { group: group.clone(), idx: 0 })
        .is_err());
}


/// Tests `BasicPalette::assign_group` followed by `BasicPalette::clear_groups`.
#[test]
fn cell_assign_clear_group_method_inverse() {
    let mut pal = BasicPalette::new();
    pal.insert_cell(0, Cell::default());

    let group1: Cow<'_, _> = "TestGroup1".into();
    let group2: Cow<'_, _> = "TestGroup2".into();

    pal.assign_group(CellRef::Index(0), group1.clone(), None);
    pal.assign_group(CellRef::Index(0), group2.clone(), None);
    assert!(pal.cell(&CellRef::Group { group: group1.clone(), idx: 0 })
        .is_ok());
    assert!(pal.cell(&CellRef::Group { group: group2.clone(), idx: 0 })
        .is_ok());

    pal.clear_groups(CellRef::Index(0));
    assert!(pal.cell(&CellRef::Group { group: group1.clone(), idx: 0 })
        .is_err());
    assert!(pal.cell(&CellRef::Group { group: group2.clone(), idx: 0 })
        .is_err());
}

