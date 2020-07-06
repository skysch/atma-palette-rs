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
use crate::palette::BasicPalette;
use crate::color::Rgb;
use crate::palette::Expr;
use crate::cell::Cell;
use crate::cell::Position;
use crate::cell::CellRef;
use crate::cell::CellSelector;
use crate::cell::CellSelection;
use crate::cell::PositionSelector;

// Standard library imports.
use std::str::FromStr;


fn test_palette() -> BasicPalette {
    let mut palette = BasicPalette::new();

    for i in 100u8..200u8 {
        let r: u8 = u8::MAX - i;
        let g: u8 = i;
        let b: u8 = u8::MAX - i;
        palette.insert_cell(i as u32, Cell::new_with_expr(
            Expr::Color(Rgb { r, g, b }.into())));
    }
    
    palette.assign_name(PositionSelector::new(1, 0, 0), "a");
    palette.assign_name(PositionSelector::new(1, 1, 0), "b");
    palette.assign_name(PositionSelector::new(2, 0, 0), "c");
    palette.assign_name(PositionSelector::new(2, 1, 0), "d");
    palette.assign_name(PositionSelector::new(3, 0, 0), "e");
    palette.assign_name(PositionSelector::new(3, 1, 0), "f");
    palette.assign_name(PositionSelector::new(4, 0, 0), "g");
    palette.assign_name(PositionSelector::new(4, 1, 0), "h");
    palette.assign_name(PositionSelector::new(5, 0, 0), "i");
    palette.assign_name(PositionSelector::new(5, 1, 0), "j");

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
// CellSelector resolution
////////////////////////////////////////////////////////////////////////////////
#[test]
fn cell_selector_resolve_all() {
    let pal = test_palette();
    let selector = CellSelector::All;

    let res: Vec<_> = selector.resolve(&pal).collect();

    // assert_eq!(res.len(), 100);
    assert_eq!(res[0], 100u32);
    assert_eq!(res[99], 199u32);
}

#[test]
fn cell_selector_resolve_index() {
    let pal = test_palette();
    let selector = CellSelector::Index(123u32);

    let res: Vec<_> = selector.resolve(&pal).collect();

    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 123u32);
}

#[test]
fn cell_selector_resolve_index_range() {
    let pal = test_palette();
    let selector = CellSelector::IndexRange { low: 180, high: 210 };

    let res: Vec<_> = selector.resolve(&pal).collect();

    assert_eq!(res.len(), 20);
    assert_eq!(res[0], 180u32);
    assert_eq!(res[19], 199u32);
}

#[test]
fn cell_selector_resolve_name() {
    let pal = test_palette();

    let selector = CellSelector::Name("a".into());
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 100u32);

    let selector = CellSelector::Name("b".into());
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 110u32);

    let selector = CellSelector::Name("z".into());
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 0);
}

#[test]
fn cell_selector_resolve_group() {
    let pal = test_palette();

    let selector = CellSelector::Group { group: "GroupA".into(), idx: 9};
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 109u32);


    let selector = CellSelector::Group { group: "GroupA".into(), idx: 10};
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 0);
}

#[test]
fn cell_selector_resolve_group_all() {
    let pal = test_palette();

    let selector = CellSelector::GroupAll("GroupA".into());
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 10);
    assert_eq!(res[0], 100u32);
    assert_eq!(res[9], 109u32);

    let selector = CellSelector::GroupAll("GroupB".into());
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 15);
    assert_eq!(res[0], 120u32);
    assert_eq!(res[14], 134u32);

    let selector = CellSelector::GroupAll("GroupC".into());
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 20);
    assert_eq!(res[0], 150u32);
    assert_eq!(res[19], 169u32);
}

#[test]
fn cell_selector_resolve_group_range() {
    let pal = test_palette();

    let selector = CellSelector::GroupRange {
        group: "GroupA".into(),
        low: 3,
        high: 6,
    };
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 4);
    assert_eq!(res[0], 103u32);
    assert_eq!(res[3], 106u32);

    let selector = CellSelector::GroupRange {
        group: "GroupB".into(),
        low: 10,
        high: 20,
    };
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 5);
    assert_eq!(res[0], 130u32);
    assert_eq!(res[4], 134u32);
}

#[test]
fn cell_selector_resolve_position_range() {
    let pal = test_palette();

    let selector = CellSelector::PositionRange {
        low: Position { page: 0, line: 0, column: 0 },
        high: Position { page: 2, line: 0, column: 0 },
    };
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 100);
    assert_eq!(res[0], 100u32);
    assert_eq!(res[99], 199u32);


    let selector = CellSelector::PositionRange {
        low: Position { page: 1, line: 1, column: 1 },
        high: Position { page: 1, line: 1, column: 1 },
    };
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 111u32);
}

#[test]
fn cell_selector_resolve_position_selector() {
    let pal = test_palette();

    let selector = CellSelector::PositionSelector(PositionSelector {
        page: Some(1),
        line: Some(1),
        column: Some(1),
    });
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 111u32);

    let selector = CellSelector::PositionSelector(PositionSelector {
        page: Some(1),
        line: Some(1),
        column: None,
    });
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 10);
    assert_eq!(res[0], 110u32);
    assert_eq!(res[9], 119u32);


    let selector = CellSelector::PositionSelector(PositionSelector {
        page: Some(1),
        line: None,
        column: Some(4),
    });
    let res: Vec<_> = selector.resolve(&pal).collect();
    assert_eq!(res.len(), 10);
    assert_eq!(res[0], 104u32);
    assert_eq!(res[1], 114u32);
    assert_eq!(res[2], 124u32);
    assert_eq!(res[9], 194u32);
}

////////////////////////////////////////////////////////////////////////////////
// CellSelection resolution
////////////////////////////////////////////////////////////////////////////////
#[test]
fn cell_selection_resolve() {
    let pal = test_palette();

    let selection = CellSelection::from_str(":100-:105, :1.6.6, GroupD:*")
        .unwrap();
    let res: Vec<_> = selection.resolve(&pal).into_iter().collect();
    assert_eq!(res.len(), 8);
    
}
