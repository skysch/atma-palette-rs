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
use crate::cell::Position;
use crate::cell::CellRef;

use std::borrow::Cow;

/// Tests `Palette::insert_cell` followed by `Palette::remove_cell`.
#[test]
fn cell_insert_remove_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());
    assert!(pal.cell(CellRef::Index(0)).is_ok());
    let _ = pal.remove_cell(CellRef::Index(0));
    assert!(pal.cell(CellRef::Index(0)).is_err());
}


/// Tests `Palette::assign_name` followed by `Palette::unassign_name`.
#[test]
fn cell_assign_unassign_name_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());

    let name: Cow<'_, _> = "TestName".into();

    let _ = pal.assign_name(CellRef::Index(0), name.clone());
    assert!(pal.cell(CellRef::Name(name.clone())).is_ok());

    let _ = pal.unassign_name(CellRef::Index(0), name.clone());
    assert!(pal.cell(CellRef::Name(name.clone())).is_err());
}


/// Tests `Palette::assign_name` followed by `Palette::clear_names`.
#[test]
fn cell_assign_clear_name_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());

    let name1: Cow<'_, _> = "TestName1".into();
    let name2: Cow<'_, _> = "TestName2".into();

    let _ = pal.assign_name(CellRef::Index(0), name1.clone());
    let _ = pal.assign_name(CellRef::Index(0), name2.clone());
    assert!(pal.cell(CellRef::Name(name1.clone())).is_ok());
    assert!(pal.cell(CellRef::Name(name2.clone())).is_ok());

    let _ = pal.clear_names(CellRef::Index(0));
    assert!(pal.cell(CellRef::Name(name1.clone())).is_err());
    assert!(pal.cell(CellRef::Name(name2.clone())).is_err());
}


/// Tests `Palette::assign_position` followed by `Palette::unassign_position`.
#[test]
fn cell_assign_unassign_position_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());

    let position = Position { page: 0, line: 10 };

    let _ = pal.assign_position(CellRef::Index(0), position.clone());
    assert!(pal.cell(CellRef::Position(position.clone())).is_ok());

    let _ = pal.unassign_position(CellRef::Index(0), position.clone());
    assert!(pal.cell(CellRef::Position(position.clone())).is_err());
}


/// Tests `Palette::assign_position` followed by `Palette::clear_positions`.
#[test]
fn cell_assign_clear_position_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());

    let position1 = Position { page: 0, line: 10 };
    let position2 = Position { page: 1, line: 4 };

    let _ = pal.assign_position(CellRef::Index(0), position1.clone());
    let _ = pal.assign_position(CellRef::Index(0), position2.clone());
    assert!(pal.cell(CellRef::Position(position1.clone())).is_ok());
    assert!(pal.cell(CellRef::Position(position2.clone())).is_ok());

    let _ = pal.clear_positions(CellRef::Index(0));
    assert!(pal.cell(CellRef::Position(position1.clone())).is_err());
    assert!(pal.cell(CellRef::Position(position2.clone())).is_err());
}

/// Tests `Palette::assign_group` followed by `Palette::unassign_group`.
#[test]
fn cell_assign_unassign_group_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());

    let group: Cow<'_, _> = "TestGroup".into();

    let _ = pal.assign_group(CellRef::Index(0), group.clone(), None);
    assert!(pal.cell(CellRef::Group { group: group.clone(), idx: 0 })
        .is_ok());

    let _ = pal.unassign_group(CellRef::Index(0), group.clone());
    assert!(pal.cell(CellRef::Group { group: group.clone(), idx: 0 })
        .is_err());
}


/// Tests `Palette::assign_group` followed by `Palette::clear_groups`.
#[test]
fn cell_assign_clear_group_method_inverse() {
    let mut pal = Palette::new();
    let _ = pal.insert_cell(None, &Cell::default());

    let group1: Cow<'_, _> = "TestGroup1".into();
    let group2: Cow<'_, _> = "TestGroup2".into();

    let _ = pal.assign_group(CellRef::Index(0), group1.clone(), None);
    let _ = pal.assign_group(CellRef::Index(0), group2.clone(), None);
    assert!(pal.cell(CellRef::Group { group: group1.clone(), idx: 0 })
        .is_ok());
    assert!(pal.cell(CellRef::Group { group: group2.clone(), idx: 0 })
        .is_ok());

    let _ = pal.clear_groups(CellRef::Index(0));
    assert!(pal.cell(CellRef::Group { group: group1.clone(), idx: 0 })
        .is_err());
    assert!(pal.cell(CellRef::Group { group: group2.clone(), idx: 0 })
        .is_err());
}
