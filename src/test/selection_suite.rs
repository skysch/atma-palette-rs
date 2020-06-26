////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Selection and Selector function test suite.
////////////////////////////////////////////////////////////////////////////////
#![allow(unused_must_use)]

// Local imports.
use crate::basic::BasicPalette;
use crate::color::Color;
use crate::color::Rgb;
use crate::expr::Expr;
use crate::cell::Cell;
use crate::cell::Position;
use crate::cell::CellRef;
use crate::selection::CellSelector;
use crate::parse::cell_selection;


fn test_palette() -> BasicPalette {
    let mut palette = BasicPalette::new();

    for i in 100u8..200u8 {
        let r: u8 = u8::MAX - i;
        let g: u8 = i;
        let b: u8 = u8::MAX - i;
        palette.insert_cell(Some(i as u32), &Cell::new_with_expr(
            Expr::Color(Rgb { r, g, b }.into())));
    }
    
    palette.assign_name(CellRef::Index(100), "a");
    palette.assign_name(CellRef::Index(110), "b");
    palette.assign_name(CellRef::Index(120), "c");
    palette.assign_name(CellRef::Index(130), "d");
    palette.assign_name(CellRef::Index(140), "e");
    palette.assign_name(CellRef::Index(150), "f");
    palette.assign_name(CellRef::Index(160), "g");
    palette.assign_name(CellRef::Index(170), "h");
    palette.assign_name(CellRef::Index(180), "i");
    palette.assign_name(CellRef::Index(190), "j");

    for i in 0u32..10u32 {
        palette.assign_group(CellRef::Index(100 + i), "GroupA", None);
    }
    for i in 0u32..15u32 {
        palette.assign_group(CellRef::Index(120 + i), "GroupB", None);
    }
    for i in 0u32..20u32 {
        palette.assign_group(CellRef::Index(150 + i), "GroupC", None);
    }

    palette.assign_group(CellRef::Index(197), "GroupD", None);

    let mut i: u32 = 0;
    for page in 1..5 {
        for line in 0..10 {
            for column in 0u16..10 {
                palette.assign_position(CellRef::Index(100 + i), Position {
                    page, line, column });
                i += 1;
            }
        }
    }

    palette
}

////////////////////////////////////////////////////////////////////////////////
// Palette data bounds
////////////////////////////////////////////////////////////////////////////////

#[test]
fn palette_index_bounds_a() {
    let pal = test_palette();
    assert_eq!(
        pal.occupied_index_range(),
        Some((100, 199)));

    assert_eq!(
        pal.next_occupied_index_after(&5),
        Some(&100));
    assert_eq!(
        pal.next_occupied_index_after(&99),
        Some(&100));
    assert_eq!(
        pal.next_occupied_index_after(&150),
        Some(&151));
    assert_eq!(
        pal.next_occupied_index_after(&198),
        Some(&199));
    assert_eq!(
        pal.next_occupied_index_after(&199),
        None);
}

#[test]
fn palette_index_bounds_fresh() {
    let mut pal = BasicPalette::new();

    assert_eq!(
        pal.occupied_index_range(),
        None);

    pal.insert_cell(Some(88), &Cell::new());
    assert_eq!(
        pal.occupied_index_range(),
        Some((88, 88)));    
}

#[test]
fn palette_position_bounds() {
    let pal = test_palette();

    assert_eq!(
        pal.assigned_position_range(),
        Some((
            Position { page: 1, line: 0, column: 0 },
            Position { page: 4, line: 9, column: 9 } )));


    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 0, line: 0, column: 0 }),
        Some((&Position { page: 1, line: 0, column: 0 }, &100)));

    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 0, line: 0xFFFF, column: 0xFFFF }),
        Some((&Position { page: 1, line: 0, column: 0 }, &100)));

    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 1, line: 0, column: 0 }),
        Some((&Position { page: 1, line: 0, column: 1 }, &101)));

    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 1, line: 0, column: 20 }),
        Some((&Position { page: 1, line: 1, column: 0 }, &110)));

    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 1, line: 20, column: 20 }),
        Some((&Position { page: 2, line: 0, column: 0 }, &200)));

    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 4, line: 9, column: 8 }),
        Some((&Position { page: 4, line: 9, column: 9 }, &499)));

    assert_eq!(
        pal.next_assigned_position_after(
            &Position { page: 4, line: 9, column: 9 }),
        None);
}
#[test]
fn palette_position_bounds_fresh() {
    let mut pal = BasicPalette::new();

    assert_eq!(
        pal.assigned_position_range(),
        None);

    pal.insert_cell(Some(88), &Cell::new());
    pal.assign_position(CellRef::Index(88), Position {
                    page: 100, line: 15, column: 4 });
    assert_eq!(
        pal.assigned_position_range(),
        Some((
            Position { page: 100, line: 15, column: 4 }, 
            Position { page: 100, line: 15, column: 4 })));    
}

#[test]
fn palette_group_bounds() {
    let pal = test_palette();

    assert_eq!(
        pal.assigned_group_range("GroupA"),
        Some((0, 9)));

    assert_eq!(
        pal.assigned_group_range("GroupB"),
        Some((0, 14)));

    assert_eq!(
        pal.assigned_group_range("GroupC"),
        Some((0, 19)));

    assert_eq!(
        pal.assigned_group_range("GroupD"),
        Some((0, 0)));

    assert_eq!(
        pal.assigned_group_range("GroupE"),
        None);
}

////////////////////////////////////////////////////////////////////////////////
// CellSelector resolution
////////////////////////////////////////////////////////////////////////////////
#[test]
fn cell_selector_index_iter_all() {
    let pal = test_palette();
    let selector = CellSelector::All;

    let res: Vec<_> = selector.index_iter(&pal).collect();

    assert_eq!(res.len(), 100);
}

#[test]
fn cell_selector_index_iter_index() {
    let pal = test_palette();
    let selector = CellSelector::Index(123u32);

    let res: Vec<_> = selector.index_iter(&pal).collect();

    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 123u32);
}

#[test]
fn cell_selector_index_iter_index_range() {
    let pal = test_palette();
    let selector = CellSelector::IndexRange { low: 180, high: 210 };

    let res: Vec<_> = selector.index_iter(&pal).collect();

    assert_eq!(res.len(), 20);
    assert_eq!(res[0], 180u32);
    assert_eq!(res[19], 199u32);
}
