////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! CellSelection parser test suite.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::parse::*;
use crate::cell::Position;
use crate::cell::CellSelector;
use crate::cell::PositionSelector;
use crate::cell::CellRef;


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::name`.
#[test]
fn name_match() {
    assert_eq!(
        name("'xyz'"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: "",
        }));

    assert_eq!(
        name("'xyz' abcd"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: " abcd",
        }));

    assert_eq!(
        name("'xyz'.abcd"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: ".abcd",
        }));

    assert_eq!(
        name("'xyz',abcd"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: ",abcd",
        }));

    assert_eq!(
        name("'xyz':abcd"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: ":abcd",
        }));

    assert_eq!(
        name("'xyz'-abcd"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: "-abcd",
        }));

    assert_eq!(
        name("'xyz'*abcd"),
        Ok(Success {
            value: "xyz".into(),
            token: "'xyz'",
            rest: "*abcd",
        }));
}

/// Tests `parse::name`.
#[test]
fn name_nonmatch() {
    assert_eq!(
        name(" 'xyz'"),
        Err(Failure {
            token: "",
            rest: " 'xyz'",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name(".'xyz'"),
        Err(Failure {
            token: "",
            rest: ".'xyz'",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name(",'xyz'"),
        Err(Failure {
            token: "",
            rest: ",'xyz'",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name(":'xyz'"),
        Err(Failure {
            token: "",
            rest: ":'xyz'",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name("-'xyz'"),
        Err(Failure {
            token: "",
            rest: "-'xyz'",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name("*'xyz'"),
        Err(Failure {
            token: "",
            rest: "*'xyz'",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


/// Tests `parse::index`.
#[test]
fn index_match() {
    assert_eq!(
        index(":0abcd"),
        Ok(Success {
            value: 0,
            token: ":0",
            rest: "abcd",
        }));

    assert_eq!(
        index(":0b10abcd"),
        Ok(Success {
            value: 0b10,
            token: ":0b10",
            rest: "abcd",
        }));
    
    assert_eq!(
        index(":0o70abcd"),
        Ok(Success {
            value: 0o70,
            token: ":0o70",
            rest: "abcd",
        }));
    
    assert_eq!(
        index(":0xF0 abcd"),
        Ok(Success {
            value: 0xF0,
            token: ":0xF0",
            rest: " abcd",
        }));
    
    assert_eq!(
        index(":0xFFFF abcd"),
        Ok(Success {
            value: 0xFFFF,
            token: ":0xFFFF",
            rest: " abcd",
        }));
}

/// Tests `parse::index`.
#[test]
fn index_nonmatch() {
    assert_eq!(
        index(":abcd"),
        Err(Failure {
            token: ":",
            rest: ":abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0b20abcd"),
        Err(Failure {
            token: ":0b",
            rest: ":0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0o80abcd"),
        Err(Failure {
            token: ":0o",
            rest: ":0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0xG0abcd"),
        Err(Failure {
            token: ":0x",
            rest: ":0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0x100000000 abcd"),
        Err(Failure {
            token: ":0x100000000",
            rest: ":0x100000000 abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::position`.
#[test]
fn position_match() {
    assert_eq!(
        position(":1.2.3abcd"),
        Ok(Success {
            value: Position { page: 1, line: 2, column: 3 },
            token: ":1.2.3",
            rest: "abcd",
        }));

    assert_eq!(
        position(":0b101.0x2F.0o17abcd"),
        Ok(Success {
            value: Position { page: 0b101, line: 0x2F, column: 0o17 },
            token: ":0b101.0x2F.0o17",
            rest: "abcd",
        }));
}

/// Tests `parse::position`.
#[test]
fn position_nonmatch() {
    assert_eq!(
        position(":1 .2.3abcd"),
        Err(Failure {
            token: ":1",
            rest: ":1 .2.3abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position(":1.*.3abcd"),
        Err(Failure {
            token: ":1.",
            rest: ":1.*.3abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position(":1.2.0x10000 abcd"),
        Err(Failure {
            token: ":1.2.0x10000",
            rest: ":1.2.0x10000 abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::cell_ref`.
#[test]
fn cell_ref_index_match() {
    assert_eq!(
        cell_ref(":0abcd"),
        Ok(Success {
            value: CellRef::Index(0u32),
            token: ":0",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_ref`.
#[test]
fn cell_ref_position_match() {
    assert_eq!(
        cell_ref(":1.2.3abcd"),
        Ok(Success {
            value: CellRef::Position(Position { page: 1, line: 2, column: 3 }),
            token: ":1.2.3",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_ref`.
#[test]
fn cell_ref_name_match() {
    assert_eq!(
        cell_ref("'xyz' abcd"),
        Ok(Success {
            value: CellRef::Name("xyz".into()),
            token: "'xyz'",
            rest: " abcd",
        }));
}

/// Tests `parse::cell_ref`.
#[test]
fn cell_ref_group_match() {
    assert_eq!(
        cell_ref("'xyz':12 abcd"),
        Ok(Success {
            value: CellRef::Group { group: "xyz".into(), idx: 12 },
            token: "'xyz':12",
            rest: " abcd",
        }));
}

////////////////////////////////////////////////////////////////////////////////
// CellSelector
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::position`.
#[test]
fn position_selector_match() {
    assert_eq!(
        position_selector(":1.2.3abcd"),
        Ok(Success {
            value: PositionSelector { 
                page: Some(1),
                line: Some(2),
                column: Some(3),
            },
            token: ":1.2.3",
            rest: "abcd",
        }));

    assert_eq!(
        position_selector(":0b1111111111111111.0o177777.0xFFFF.abcd"),
        Ok(Success {
            value: PositionSelector { 
                page: Some(0b1111111111111111),
                line: Some(0o177777),
                column: Some(0xFFFF),
            },
            token: ":0b1111111111111111.0o177777.0xFFFF",
            rest: ".abcd",
        }));

    assert_eq!(
        position_selector(":*.2.3abcd"),
        Ok(Success {
            value: PositionSelector { 
                page: None,
                line: Some(2),
                column: Some(3),
            },
            token: ":*.2.3",
            rest: "abcd",
        }));

    assert_eq!(
        position_selector(":*.*.3abcd"),
        Ok(Success {
            value: PositionSelector { 
                page: None,
                line: None,
                column: Some(3),
            },
            token: ":*.*.3",
            rest: "abcd",
        }));

    assert_eq!(
        position_selector(":1.2.*abcd"),
        Ok(Success {
            value: PositionSelector { 
                page: Some(1),
                line: Some(2),
                column: None,
            },
            token: ":1.2.*",
            rest: "abcd",
        }));
}

/// Tests `parse::position`.
#[test]
fn position_selector_nonmatch() {
    assert_eq!(
        position_selector(":1.2.abcd"),
        Err(Failure {
            token: ":1.2.",
            rest: ":1.2.abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position_selector(":1.2.0x1FFFF abcd"),
        Err(Failure {
            token: ":1.2.0x1FFFF",
            rest: ":1.2.0x1FFFF abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position_selector(":1.**.3abcd"),
        Err(Failure {
            token: ":1.*",
            rest: ":1.**.3abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_index_match() {
    assert_eq!(
        range_suffix(index)("-:10abcd"),
        Ok(Success {
            value: 10,
            token: "-:10",
            rest: "abcd",
        }));

    assert_eq!(
        range_suffix(index)(" - :10abcd"),
        Ok(Success {
            value: 10,
            token: " - :10",
            rest: "abcd",
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_index_nonmatch() {
    assert_eq!(
        range_suffix(index)("-:0x1FFFFFFFF abcd"),
        Err(Failure {
            token: "-:0x1FFFFFFFF",
            rest: "-:0x1FFFFFFFF abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(index)("-: 10abcd"),
        Err(Failure {
            token: "-:",
            rest: "-: 10abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(index)("--: 10abcd"),
        Err(Failure {
            token: "-",
            rest: "--: 10abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_position_match() {
    assert_eq!(
        range_suffix(position)("-:1.2.3abcd"),
        Ok(Success {
            value: Position { page: 1, line: 2, column: 3 },
            token: "-:1.2.3",
            rest: "abcd",
        }));

    assert_eq!(
        range_suffix(position)("\t-  :1.2.3abcd"),
        Ok(Success {
            value: Position { page: 1, line: 2, column: 3 },
            token: "\t-  :1.2.3",
            rest: "abcd",
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_position_nonmatch() {
    assert_eq!(
        range_suffix(position)("- -: 10abcd"),
        Err(Failure {
            token: "- ",
            rest: "- -: 10abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(position)(" -:0xFFFF.0xFFFF.0x1FFFF abcd"),
        Err(Failure {
            token: " -:0xFFFF.0xFFFF.0x1FFFF",
            rest: " -:0xFFFF.0xFFFF.0x1FFFF abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_group_match() {
    assert_eq!(
        range_suffix(group)("- 'xyz':0abcd"),
        Ok(Success {
            value: ("xyz".into(), 0),
            token: "- 'xyz':0",
            rest: "abcd",
        }));

    assert_eq!(
        range_suffix(group)("- 'xyz':0xFFFFFFFF abcd"),
        Ok(Success {
            value: ("xyz".into(), 0xFFFFFFFF),
            token: "- 'xyz':0xFFFFFFFF",
            rest: " abcd",
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_group_nonmatch() {
    assert_eq!(
        range_suffix(group)("- -'xyz':0abcd"),
        Err(Failure {
            token: "- ",
            rest: "- -'xyz':0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(group)(" -'xyz':0xHabcd"),
        Err(Failure {
            token: " -'xyz':0x",
            rest: " -'xyz':0xHabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_all_match() {
    assert_eq!(
        cell_selector("*abcd"),
        Ok(Success {
            value: CellSelector::All,
            token: "*",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_index_match() {
    assert_eq!(
        cell_selector(":1abcd"),
        Ok(Success {
            value: CellSelector::Index(1),
            token: ":1",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_index_range_match() {
    assert_eq!(
        cell_selector(":1-:2abcd"),
        Ok(Success {
            value: CellSelector::IndexRange { low: 1, high: 2 },
            token: ":1-:2",
            rest: "abcd",
        }));

    assert_eq!(
        cell_selector(":1-:1abcd"),
        Ok(Success {
            value: CellSelector::Index(1),
            token: ":1-:1",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_index_range_nonmatch() {
    assert_eq!(
        cell_selector(":1-:0abcd"),
        Err(Failure {
            token: ":1-:0",
            rest: ":1-:0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_position_match() {
    assert_eq!(
        cell_selector(":1.2.3abcd"),
        Ok(Success {
            value: CellSelector::PositionSelector(PositionSelector {
                page: Some(1),
                line: Some(2),
                column: Some(3),
            }),
            token: ":1.2.3",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_position_range_match() {
    assert_eq!(
        cell_selector(":1.2.3-:4.5.6abcd"),
        Ok(Success {
            value: CellSelector::PositionRange {
                low: Position { page: 1, line: 2, column: 3 },
                high: Position { page: 4, line: 5, column: 6 }
            },
            token: ":1.2.3-:4.5.6",
            rest: "abcd",
        }));

    assert_eq!(
        cell_selector(":1.2.3-:1.2.3abcd"),
        Ok(Success {
            value: CellSelector::PositionSelector(PositionSelector {
                page: Some(1),
                line: Some(2),
                column: Some(3),
            }),
            token: ":1.2.3-:1.2.3",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_position_range_nonmatch() {
    assert_eq!(
        cell_selector(":1.2.3-:0.5.6abcd"),
        Err(Failure {
            token: ":1.2.3-:0.5.6",
            rest: ":1.2.3-:0.5.6abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_position_selector_match() {
    assert_eq!(
        cell_selector(":1.2.*abcd"),
        Ok(Success {
            value: CellSelector::PositionSelector(PositionSelector {
                page: Some(1),
                line: Some(2),
                column: None,
            }),
            token: ":1.2.*",
            rest: "abcd",
        }));

    assert_eq!(
        cell_selector(":*.*.*abcd"),
        Ok(Success {
            value: CellSelector::PositionSelector(PositionSelector {
                page: None,
                line: None,
                column: None,
            }),
            token: ":*.*.*",
            rest: "abcd",
        }));

}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_match() {
    assert_eq!(
        cell_selector("'xyz':2abcd"),
        Ok(Success {
            value: CellSelector::Group { group: "xyz".into(), idx: 2 },
            token: "'xyz':2",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_range_match() {
    assert_eq!(
        cell_selector("'xyz':2-'xyz':4abcd"),
        Ok(Success {
            value: CellSelector::GroupRange {
                group: "xyz".into(),
                low: 2,
                high: 4,
            },
            token: "'xyz':2-'xyz':4",
            rest: "abcd",
        }));

    assert_eq!(
        cell_selector("'xyz':2-'xyz':2abcd"),
        Ok(Success {
            value: CellSelector::Group { group: "xyz".into(), idx: 2 },
            token: "'xyz':2-'xyz':2",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_range_nonmatch() {
    assert_eq!(
        cell_selector("'xyz':2-'xyzt':4abcd"),
        Err(Failure {
            token: "'xyz':2-'xyzt':4",
            rest: "'xyz':2-'xyzt':4abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        cell_selector("'xyz':2-'xyz':1abcd"),
        Err(Failure {
            token: "'xyz':2-'xyz':1",
            rest: "'xyz':2-'xyz':1abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_all_match() {
    assert_eq!(
        cell_selector("'xyz':*abcd"),
        Ok(Success {
            value: CellSelector::GroupAll("xyz".into()),
            token: "'xyz':*",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_name_match() {
    assert_eq!(
        cell_selector("'xyz' abcd"),
        Ok(Success {
            value: CellSelector::Name("xyz".into()),
            token: "'xyz'",
            rest: " abcd",
        }));
}

/// Tests `parse::cell_selection`.
#[test]
fn cell_selection_match() {
    assert_eq!(
        cell_selection("*, :0 , :3-:4, 'xyz'*abcd"),
        Ok(Success {
            value: vec![
                CellSelector::All,
                CellSelector::Index(0),
                CellSelector::IndexRange { low: 3, high: 4 },
                CellSelector::Name("xyz".into()),
            ].into(),
            token: "*, :0 , :3-:4, 'xyz'",
            rest: "*abcd",
        }));
}

