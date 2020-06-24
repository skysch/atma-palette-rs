////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! CellSelection parser test suite.
////////////////////////////////////////////////////////////////////////////////

use crate::parse::*;
use crate::cell::Position;
use crate::selection::CellSelector;
use crate::selection::PositionSelector;
use crate::cell::CellRef;


////////////////////////////////////////////////////////////////////////////////
// Char primitives
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::char`.
#[test]
fn parse_char_match() {
    assert_eq!(
        char('a')("abcd"),
        Ok(Success {
            value: 'a',
            token: "a",
            rest: "bcd",
        }));
}

/// Tests `parse::char`.
#[test]
fn parse_char_nonmatch() {
    assert_eq!(
        char('b')("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::char_in`.
#[test]
fn parse_char_in_match() {
    assert_eq!(
        char_in("cab")("abcd"),
        Ok(Success {
            value: 'a',
            token: "a",
            rest: "bcd",
        }));
}

/// Tests `parse::char_in`.
#[test]
fn parse_char_in_nonmatch() {
    assert_eq!(
        char_in("bdcbd")("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


/// Tests `parse::char_matching`.
#[test]
fn parse_char_matching_match() {
    assert_eq!(
        char_matching(|c| c == 'a')("abcd"),
        Ok(Success {
            value: 'a',
            token: "a",
            rest: "bcd",
        }));
}

/// Tests `parse::char_matching`.
#[test]
fn parse_char_matching_nonmatch() {
    assert_eq!(
        char_matching(|c| c == 'b')("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


/// Tests `parse::char_whitespace`.
#[test]
fn parse_char_whitespace_match() {
    assert_eq!(
        char_whitespace("\tabcd"),
        Ok(Success {
            value: '\t',
            token: "\t",
            rest: "abcd",
        }));

    assert_eq!(char_whitespace(" abcd"),
        Ok(Success {
            value: ' ',
            token: " ",
            rest: "abcd",
        }));
}

/// Tests `parse::char_whitespace`.
#[test]
fn parse_char_whitespace_nonmatch() {
    assert_eq!(
        char_whitespace("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

////////////////////////////////////////////////////////////////////////////////
// String primitives
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::whitespace`.
#[test]
fn parse_whitespace() {
    assert_eq!(
        whitespace("\t \n \tabcd"),
        Ok(Success {
            value: "\t \n \t",
            token: "\t \n \t",
            rest: "abcd",
        }));

    assert_eq!(
        whitespace("abcd"),
        Ok(Success {
            value: "",
            token: "",
            rest: "abcd",
        }));
}


////////////////////////////////////////////////////////////////////////////////
// Combinators
////////////////////////////////////////////////////////////////////////////////

/// Tests `maybe` on `parse::char`.
#[test]
fn parse_maybe_char_match() {
    assert_eq!(
        maybe(char('a'))("abcd"),
        Ok(Success {
            value: Some('a'),
            token: "a",
            rest: "bcd",
        }));
}

/// Tests `maybe` on `parse::char`.
#[test]
fn parse_maybe_char_nonmatch() {
    assert_eq!(
        maybe(char('b'))("abcd"),
        Ok(Success {
            value: None,
            token: "",
            rest: "abcd",
        }));
}

/// Tests `repeat` on `parse::char`.
#[test]
fn parse_repeat_char_match() {
    assert_eq!(
        repeat(3, Some(5), char('a'))("aaaabcd"),
        Ok(Success {
            value: 4,
            token: "aaaa",
            rest: "bcd",
        }));
}

/// Tests `repeat` on `parse::char`.
#[test]
fn parse_repeat_char_nonmatch() {
    assert_eq!(
        repeat(3, Some(5), char('a'))("aabcd"),
        Err(Failure {
            context: "aa",
            rest: "aabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

////////////////////////////////////////////////////////////////////////////////
// integers
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::radix_prefix`.
#[test]
fn parse_radix_prefix_match() {
    assert_eq!(
        radix_prefix("0b1234abcd"),
        Ok(Success {
            value: "0b",
            token: "0b",
            rest: "1234abcd",
        }));

    assert_eq!(
        radix_prefix("0o1234abcd"),
        Ok(Success {
            value: "0o",
            token: "0o",
            rest: "1234abcd",
        }));

    assert_eq!(
        radix_prefix("0x1234abcd"),
        Ok(Success {
            value: "0x",
            token: "0x",
            rest: "1234abcd",
        }));
}

/// Tests `parse::radix_prefix`.
#[test]
fn parse_radix_prefix_nonmatch() {
    assert_eq!(
        radix_prefix("1234abcd"),
        Err(Failure {
            context: "",
            rest: "1234abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::uint` for a `u8` value.
#[test]
fn parse_uint_u8_match() {
    assert_eq!(
        uint::<u8>("u8")("0abcd"),
        Ok(Success {
            value: 0u8,
            token: "0",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u8>("u8")("0b10abcd"),
        Ok(Success {
            value: 0b10u8,
            token: "0b10",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u8>("u8")("0o70abcd"),
        Ok(Success {
            value: 0o70u8,
            token: "0o70",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u8>("u8")("0xFf abcd"),
        Ok(Success {
            value: 0xFfu8,
            token: "0xFf",
            rest: " abcd",
        }));
}

/// Tests `parse::uint` for a `u8` value.
#[test]
fn parse_uint_u8_nonmatch() {
    let uint_u8_res = uint::<u8>("u8")("abcd");
    assert!(uint_u8_res.is_err());
    assert_eq!(uint_u8_res.rest(), "abcd");

    let uint_u8_res = uint::<u8>("u8")("0b20abcd");
    assert!(uint_u8_res.is_err());
    assert_eq!(uint_u8_res.rest(), "0b20abcd");

    let uint_u8_res = uint::<u8>("u8")("0o80abcd");
    assert!(uint_u8_res.is_err());
    assert_eq!(uint_u8_res.rest(), "0o80abcd");

    let uint_u8_res = uint::<u8>("u8")("0xG0abcd");
    assert!(uint_u8_res.is_err());
    assert_eq!(uint_u8_res.rest(), "0xG0abcd");

    let uint_u8_res = uint::<u8>("u8")("0xFF0abcd");
    assert!(uint_u8_res.is_err());
    assert_eq!(uint_u8_res.rest(), "0xFF0abcd");
}

/// Tests `parse::uint` for a `u16` value.
#[test]
fn parse_uint_u16_match() {
    let uint_u16_res = uint::<u16>("u16")("0abcd");
    assert!(uint_u16_res.is_ok());
    assert_eq!(uint_u16_res.rest(), "abcd");
    assert_eq!(uint_u16_res.into_value(), Some(0u16));

    let uint_u16_res = uint::<u16>("u16")("0b10abcd");
    assert!(uint_u16_res.is_ok());
    assert_eq!(uint_u16_res.rest(), "abcd");
    assert_eq!(uint_u16_res.into_value(), Some(0b10u16));

    let uint_u16_res = uint::<u16>("u16")("0o70abcd");
    assert!(uint_u16_res.is_ok());
    assert_eq!(uint_u16_res.rest(), "abcd");
    assert_eq!(uint_u16_res.into_value(), Some(0o70u16));

    let uint_u16_res = uint::<u16>("u16")("0xF0 abcd");
    assert!(uint_u16_res.is_ok());
    assert_eq!(uint_u16_res.rest(), " abcd");
    assert_eq!(uint_u16_res.into_value(), Some(0xF0u16));
}

/// Tests `parse::uint` for a `u16` value.
#[test]
fn parse_uint_u16_nonmatch() {
    let uint_u16_res = uint::<u16>("u16")("abcd");
    assert!(uint_u16_res.is_err());
    assert_eq!(uint_u16_res.rest(), "abcd");

    let uint_u16_res = uint::<u16>("u16")("0b20abcd");
    assert!(uint_u16_res.is_err());
    assert_eq!(uint_u16_res.rest(), "0b20abcd");

    let uint_u16_res = uint::<u16>("u16")("0o80abcd");
    assert!(uint_u16_res.is_err());
    assert_eq!(uint_u16_res.rest(), "0o80abcd");

    let uint_u16_res = uint::<u16>("u16")("0xG0abcd");
    assert!(uint_u16_res.is_err());
    assert_eq!(uint_u16_res.rest(), "0xG0abcd");

    let uint_u16_res = uint::<u16>("u16")("0xFF0abcd");
    assert!(uint_u16_res.is_err());
    assert_eq!(uint_u16_res.rest(), "0xFF0abcd");
}

/// Tests `parse::uint` for a `u32` value.
#[test]
fn parse_uint_u32_match() {
    let uint_u32_res = uint::<u32>("u32")("0abcd");
    assert!(uint_u32_res.is_ok());
    assert_eq!(uint_u32_res.rest(), "abcd");
    assert_eq!(uint_u32_res.into_value(), Some(0u32));

    let uint_u32_res = uint::<u32>("u32")("0b10abcd");
    assert!(uint_u32_res.is_ok());
    assert_eq!(uint_u32_res.rest(), "abcd");
    assert_eq!(uint_u32_res.into_value(), Some(0b10u32));

    let uint_u32_res = uint::<u32>("u32")("0o70abcd");
    assert!(uint_u32_res.is_ok());
    assert_eq!(uint_u32_res.rest(), "abcd");
    assert_eq!(uint_u32_res.into_value(), Some(0o70u32));

    let uint_u32_res = uint::<u32>("u32")("0xF0 abcd");
    assert!(uint_u32_res.is_ok());
    assert_eq!(uint_u32_res.rest(), " abcd");
    assert_eq!(uint_u32_res.into_value(), Some(0xF0u32));
}

/// Tests `parse::uint` for a `u32` value.
#[test]
fn parse_uint_u32_nonmatch() {
    let uint_u32_res = uint::<u32>("u32")("abcd");
    assert!(uint_u32_res.is_err());
    assert_eq!(uint_u32_res.rest(), "abcd");

    let uint_u32_res = uint::<u32>("u32")("0b20abcd");
    assert!(uint_u32_res.is_err());
    assert_eq!(uint_u32_res.rest(), "0b20abcd");

    let uint_u32_res = uint::<u32>("u32")("0o80abcd");
    assert!(uint_u32_res.is_err());
    assert_eq!(uint_u32_res.rest(), "0o80abcd");

    let uint_u32_res = uint::<u32>("u32")("0xG0abcd");
    assert!(uint_u32_res.is_err());
    assert_eq!(uint_u32_res.rest(), "0xG0abcd");

    let uint_u32_res = uint::<u32>("u32")("0xFFFFFF0abcd");
    assert!(uint_u32_res.is_err());
    assert_eq!(uint_u32_res.rest(), "0xFFFFFF0abcd");
}

////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::name`.
#[test]
fn parse_name_match() {
    let name_res = name("abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), "");
    assert_eq!(name_res.into_value(), Some("abcd"));

    let name_res = name("xyz .abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), " .abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz.abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), ".abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz:abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), ":abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz*abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), "*abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz-abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), "-abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz,abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), ",abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));
}

/// Tests `parse::name`.
#[test]
fn parse_name_nonmatch() {
    let name_res = name(" abcd");
    assert!(name_res.is_err());
    assert_eq!(name_res.rest(), " abcd");

    let name_res = name(".abcd");
    assert!(name_res.is_err());
    assert_eq!(name_res.rest(), ".abcd");

    let name_res = name(":abcd");
    assert!(name_res.is_err());
    assert_eq!(name_res.rest(), ":abcd");

    let name_res = name("*abcd");
    assert!(name_res.is_err());
    assert_eq!(name_res.rest(), "*abcd");

    let name_res = name("-abcd");
    assert!(name_res.is_err());
    assert_eq!(name_res.rest(), "-abcd");

    let name_res = name(",abcd");
    assert!(name_res.is_err());
    assert_eq!(name_res.rest(), ",abcd");
}


/// Tests `parse::index`.
#[test]
fn parse_index_match() {
    let index_res = index(":0abcd");
    assert!(index_res.is_ok());
    assert_eq!(index_res.rest(), "abcd");
    assert_eq!(index_res.into_value(), Some(0u32));

    let index_res = index(":0b10abcd");
    assert!(index_res.is_ok());
    assert_eq!(index_res.rest(), "abcd");
    assert_eq!(index_res.into_value(), Some(0b10u32));

    let index_res = index(":0o70abcd");
    assert!(index_res.is_ok());
    assert_eq!(index_res.rest(), "abcd");
    assert_eq!(index_res.into_value(), Some(0o70u32));

    let index_res = index(":0xF0 abcd");
    assert!(index_res.is_ok());
    assert_eq!(index_res.rest(), " abcd");
    assert_eq!(index_res.into_value(), Some(0xF0u32));
}

/// Tests `parse::index`.
#[test]
fn parse_index_nonmatch() {
    let index_res = index(":abcd");
    assert!(index_res.is_err());
    assert_eq!(index_res.rest(), ":abcd");

    let index_res = index(":0b20abcd");
    assert!(index_res.is_err());
    assert_eq!(index_res.rest(), ":0b20abcd");

    let index_res = index(":0o80abcd");
    assert!(index_res.is_err());
    assert_eq!(index_res.rest(), ":0o80abcd");

    let index_res = index(":0xG0abcd");
    assert!(index_res.is_err());
    assert_eq!(index_res.rest(), ":0xG0abcd");

    let index_res = index(":0xFFFFFF0abcd");
    assert!(index_res.is_err());
    assert_eq!(index_res.rest(), ":0xFFFFFF0abcd");
}

/// Tests `parse::position`.
#[test]
fn parse_position_match() {
    let pos_res = position(":1.2.3abcd");
    assert!(pos_res.is_ok());
    assert_eq!(pos_res.rest(), "abcd");
    assert_eq!(pos_res.into_value(), Some(Position {
        page: 1,
        line: 2,
        column: 3,
    }));

    let pos_res = position(":0b101.0x2F.0o17abcd");
    assert!(pos_res.is_ok());
    assert_eq!(pos_res.rest(), "abcd");
    assert_eq!(pos_res.into_value(), Some(Position {
        page: 0b101,
        line: 0x2F,
        column: 0o17,
    }));
}

/// Tests `parse::position`.
#[test]
fn parse_position_nonmatch() {
    let pos_res = position(":0xFFFFF1.2.3abcd");
    assert!(pos_res.is_err());
    assert_eq!(pos_res.rest(), ":0xFFFFF1.2.3abcd");

    let pos_res = position(":0xF1 .2.3abcd");
    assert!(pos_res.is_err());
    assert_eq!(pos_res.rest(), ":0xF1 .2.3abcd");

    let pos_res = position(":0xF1.*.3abcd");
    assert!(pos_res.is_err());
    assert_eq!(pos_res.rest(), ":0xF1.*.3abcd");
}

/// Tests `parse::cell_ref`.
#[test]
fn parse_cell_ref_index_match() {
    let cell_ref_res = cell_ref(":0abcd");
    assert!(cell_ref_res.is_ok());
    assert_eq!(cell_ref_res.rest(), "abcd");
    assert_eq!(cell_ref_res.into_value(), Some(CellRef::Index(0u32)));
}

/// Tests `parse::cell_ref`.
#[test]
fn parse_cell_ref_position_match() {
    let cell_ref_res = cell_ref(":1.2.3abcd");
    assert!(cell_ref_res.is_ok());
    assert_eq!(cell_ref_res.rest(), "abcd");
    assert_eq!(cell_ref_res.into_value(), Some(CellRef::Position(Position {
        page: 1,
        line: 2,
        column: 3,
    })));
}

/// Tests `parse::cell_ref`.
#[test]
fn parse_cell_ref_name_match() {
    let cell_ref_res = cell_ref("0abcd.abcd");
    assert!(cell_ref_res.is_ok());
    assert_eq!(cell_ref_res.rest(), ".abcd");
    assert_eq!(cell_ref_res.into_value(), Some(CellRef::Name("0abcd".into())));
}

/// Tests `parse::cell_ref`.
#[test]
fn parse_cell_ref_group_match() {
    let cell_ref_res = cell_ref("0abcd:123abcd");
    assert!(cell_ref_res.is_ok());
    assert_eq!(cell_ref_res.rest(), "abcd");
    assert_eq!(cell_ref_res.into_value(), Some(CellRef::Group {
        group: "0abcd".into(),
        idx: 123,
    }));
}

////////////////////////////////////////////////////////////////////////////////
// CellSelector
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::position`.
#[test]
fn parse_position_selector_match() {
    let pos_sel_res = position_selector(":1.2.3abcd");
    assert!(pos_sel_res.is_ok());
    assert_eq!(pos_sel_res.rest(), "abcd");
    assert_eq!(pos_sel_res.into_value(), Some(PositionSelector {
        page: Some(1),
        line: Some(2),
        column: Some(3),
    }));

    let pos_sel_res = position_selector(":0b101.0x2F.0o17abcd");
    assert!(pos_sel_res.is_ok());
    assert_eq!(pos_sel_res.rest(), "abcd");
    assert_eq!(pos_sel_res.into_value(), Some(PositionSelector {
        page: Some(0b101),
        line: Some(0x2F),
        column: Some(0o17),
    }));

    let pos_sel_res = position_selector(":*.2.3abcd");
    assert!(pos_sel_res.is_ok());
    assert_eq!(pos_sel_res.rest(), "abcd");
    assert_eq!(pos_sel_res.into_value(), Some(PositionSelector {
        page: None,
        line: Some(2),
        column: Some(3),
    }));

    let pos_sel_res = position_selector(":1.*.3abcd");
    assert!(pos_sel_res.is_ok());
    assert_eq!(pos_sel_res.rest(), "abcd");
    assert_eq!(pos_sel_res.into_value(), Some(PositionSelector {
        page: Some(1),
        line: None,
        column: Some(3),
    }));

    let pos_sel_res = position_selector(":*.2.*abcd");
    assert!(pos_sel_res.is_ok());
    assert_eq!(pos_sel_res.rest(), "abcd");
    assert_eq!(pos_sel_res.into_value(), Some(PositionSelector {
        page: None,
        line: Some(2),
        column: None,
    }));

    let pos_sel_res = position_selector(":*.*.*abcd");
    assert!(pos_sel_res.is_ok());
    assert_eq!(pos_sel_res.rest(), "abcd");
    assert_eq!(pos_sel_res.into_value(), Some(PositionSelector {
        page: None,
        line: None,
        column: None,
    }));
}

/// Tests `parse::position`.
#[test]
fn parse_position_selector_nonmatch() {
    let pos_sel_res = position_selector(":0xFFFFF1.2.3abcd");
    assert!(pos_sel_res.is_err());
    assert_eq!(pos_sel_res.rest(), ":0xFFFFF1.2.3abcd");

    let pos_sel_res = position_selector(":0xF1 .2.3abcd");
    assert!(pos_sel_res.is_err());
    assert_eq!(pos_sel_res.rest(), ":0xF1 .2.3abcd");

    let pos_sel_res = position_selector(":0xF1.**.3abcd");
    assert!(pos_sel_res.is_err());
    assert_eq!(pos_sel_res.rest(), ":0xF1.**.3abcd");
}

/// Tests `parse::range_suffix`.
#[test]
fn parse_range_suffix_index_match() {
    let range_suffix_res = range_suffix(index)("-:10abcd");
    assert!(range_suffix_res.is_ok());
    assert_eq!(range_suffix_res.rest(), "abcd");
    assert_eq!(range_suffix_res.into_value(), Some(10u32));

    let range_suffix_res = range_suffix(index)("  -  :0b10abcd");
    assert!(range_suffix_res.is_ok());
    assert_eq!(range_suffix_res.rest(), "abcd");
    assert_eq!(range_suffix_res.into_value(), Some(0b10u32));
}

/// Tests `parse::range_suffix`.
#[test]
fn parse_range_suffix_index_nonmatch() {
    let range_suffix_res = range_suffix(index)("-::10abcd");
    assert!(range_suffix_res.is_err());
    assert_eq!(range_suffix_res.rest(), "-::10abcd");

    let range_suffix_res = range_suffix(index)(":  -:0xH0abcd");
    assert!(range_suffix_res.is_err());
    assert_eq!(range_suffix_res.rest(), ":  -:0xH0abcd");
}

/// Tests `parse::range_suffix`.
#[test]
fn parse_range_suffix_position_match() {
    let range_suffix_res = range_suffix(position)("-:10.2.4abcd");
    assert!(range_suffix_res.is_ok());
    assert_eq!(range_suffix_res.rest(), "abcd");
    assert_eq!(range_suffix_res.into_value(), Some(Position {
        page: 10,
        line: 2,
        column: 4
    }));

    let range_suffix_res = range_suffix(position)("  - \t :0b10.2.0o4abcd");
    assert!(range_suffix_res.is_ok());
    assert_eq!(range_suffix_res.rest(), "abcd");
    assert_eq!(range_suffix_res.into_value(), Some(Position {
        page: 0b10,
        line: 2,
        column: 0o4
    }));
}

/// Tests `parse::range_suffix`.
#[test]
fn parse_range_suffix_position_nonmatch() {
    let range_suffix_res = range_suffix(position)("-::10abcd");
    assert!(range_suffix_res.is_err());
    assert_eq!(range_suffix_res.rest(), "-::10abcd");

    let range_suffix_res = range_suffix(position)(":  -:0xH0abcd");
    assert!(range_suffix_res.is_err());
    assert_eq!(range_suffix_res.rest(), ":  -:0xH0abcd");
}

/// Tests `parse::range_suffix`.
#[test]
fn parse_range_suffix_group_match() {
    let range_suffix_res = range_suffix(group)("-xyz:0abcd");
    assert!(range_suffix_res.is_ok());
    assert_eq!(range_suffix_res.rest(), "abcd");
    assert_eq!(range_suffix_res.into_value(), Some(("xyz", 0)));

    let range_suffix_res = range_suffix(group)(" - \t\nxyz:0o077abcd");
    assert!(range_suffix_res.is_ok());
    assert_eq!(range_suffix_res.rest(), "abcd");
    assert_eq!(range_suffix_res.into_value(), Some(("xyz", 0o077)));
}

/// Tests `parse::range_suffix`.
#[test]
fn parse_range_suffix_group_nonmatch() {
    let range_suffix_res = range_suffix(group)("--:10abcd");
    assert!(range_suffix_res.is_err());
    assert_eq!(range_suffix_res.rest(), "--:10abcd");

    let range_suffix_res = range_suffix(group)(":  -:0xH0abcd");
    assert!(range_suffix_res.is_err());
    assert_eq!(range_suffix_res.rest(), ":  -:0xH0abcd");
}


/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_all_match() {
    let cel_sel_res = cell_selector("*abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::All));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_index_match() {
    let cel_sel_res = cell_selector(":1abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::Index(1)));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_index_range_match() {
    let cel_sel_res = cell_selector(":1-:2abcd");
    assert_eq!(cel_sel_res, Ok(Success {
        value: CellSelector::IndexRange { 
            low: 1,
            high: 2,
        },
        token: ":1-:2",
        rest: "abcd",
    }));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_position_match() {
    let cel_sel_res = cell_selector(":1.2.3abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::PositionSelector(
        PositionSelector {
            page: Some(1),
            line: Some(2),
            column: Some(3),
        }
    )));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_position_range_match() {
    let cel_sel_res = cell_selector(":1.2.3-:4.5.6abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::PositionRange {
        low: Position { page: 1, line: 2, column: 3 },
        high: Position { page: 4, line: 5, column: 6 },
    }));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_position_selector_match() {
    let cel_sel_res = cell_selector(":*.2.3abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::PositionSelector(
        PositionSelector {
            page: None,
            line: Some(2),
            column: Some(3),
        }
    )));

    let cel_sel_res = cell_selector(":*.*.3abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::PositionSelector(
        PositionSelector {
            page: None,
            line: None,
            column: Some(3),
        }
    )));

    let cel_sel_res = cell_selector(":*.2.*abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::PositionSelector(
        PositionSelector {
            page: None,
            line: Some(2),
            column: None,
        }
    )));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_group_match() {
    let cel_sel_res = cell_selector("xyz:2abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::Group {
        group: "xyz".into(),
        idx: 2,
    }));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_group_range_match() {
    let cel_sel_res = cell_selector("xyz:2-xyz:4abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(), Some(CellSelector::GroupRange {
        group: "xyz".into(),
        low: 2,
        high: 4,
    }));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_group_all_match() {
    let cel_sel_res = cell_selector("xyz:*abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "abcd");
    assert_eq!(cel_sel_res.into_value(),
        Some(CellSelector::GroupAll("xyz".into())));
}

/// Tests `parse::cell_selector`.
#[test]
fn parse_cell_selector_name_match() {
    let cel_sel_res = cell_selector("xyz:abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), ":abcd");
    assert_eq!(cel_sel_res.into_value(),
        Some(CellSelector::Name("xyz".into())));
}

/// Tests `parse::cell_selection`.
#[test]
fn parse_cell_selection_match() {
    let cel_sel_res = cell_selection("*, :0 , :3-:4, xyz*abcd");
    assert!(cel_sel_res.is_ok());
    assert_eq!(cel_sel_res.rest(), "*abcd");
    assert_eq!(cel_sel_res.into_value(), Some(vec![
        CellSelector::All,
        CellSelector::Index(0),
        CellSelector::IndexRange { low: 3, high: 4 },
        CellSelector::Name("xyz".into()),
    ].into()));
}
