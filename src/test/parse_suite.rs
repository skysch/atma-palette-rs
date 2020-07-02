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
// ParseResult handling.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Success::join`.
#[test]
fn success_join() {
    let input = "abcdefg";

    let l = Success {
        value: 2u32,
        token: &input[..2],
        rest: &input[3..],
    };

    let r = Success {
        value: 3u32,
        token: &input[2..5],
        rest: &input[5..],
    };

    assert_eq!(
        l.join(r, input),
        Success {
            value: (),
            token: &input[..5],
            rest: &input[5..],
        })
}

/// Tests `Success::join_with`.
#[test]
fn success_join_with() {
    let input = "abcdefg";

    let l = Success {
        value: 2u32,
        token: &input[..2],
        rest: &input[3..],
    };

    let r = Success {
        value: 3u32,
        token: &input[2..5],
        rest: &input[5..],
    };

    assert_eq!(
        l.join_with(r, input, |l, r| l * r),
        Success {
            value: 6u32,
            token: &input[..5],
            rest: &input[5..],
        })

}

/// Tests `Success::join_failure`.
#[test]
fn failure_join_failure() {

    let input = "abcdefg";

    let l = Success {
        value: 2u32,
        token: &input[..2],
        rest: &input[3..],
    };

    let r = Failure {
        context: &input[2..5],
        rest: &input[2..],
        // These fields are unchecked:
        expected: "".into(), source: None,
    };

    assert_eq!(
        l.join_failure(r, input),
        Failure {
            context: &input[..5],
            rest: input,
            // These fields are unchecked:
            expected: "".into(), source: None,
        })
}


////////////////////////////////////////////////////////////////////////////////
// Char primitives
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::char`.
#[test]
fn char_match() {
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
fn char_nonmatch() {
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
fn char_in_match() {
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
fn char_in_nonmatch() {
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
fn char_matching_match() {
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
fn char_matching_nonmatch() {
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
fn char_whitespace_match() {
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
fn char_whitespace_nonmatch() {
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
fn whitespace_match() {
    assert_eq!(
        whitespace("\t \n \tabcd"),
        Ok(Success {
            value: "\t \n \t",
            token: "\t \n \t",
            rest: "abcd",
        }));

    assert_eq!(
        maybe(whitespace)("abcd"),
        Ok(Success {
            value: None,
            token: "",
            rest: "abcd",
        }));
}

/// Tests `parse::whitespace`.
#[test]
fn whitespace_nonmatch() {
    assert_eq!(
        whitespace("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::literal`.
#[test]
fn literal_match() {
    assert_eq!(
        literal("")("abcd"),
        Ok(Success {
            value: "",
            token: "",
            rest: "abcd",
        }));

    assert_eq!(
        literal("abc")("abcd"),
        Ok(Success {
            value: "abc",
            token: "abc",
            rest: "d",
        }));
}

/// Tests `parse::literal`.
#[test]
fn literal_nonmatch() {
    assert_eq!(
        literal("xyz")("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

////////////////////////////////////////////////////////////////////////////////
// Combinators
////////////////////////////////////////////////////////////////////////////////

/// Tests `maybe` on `parse::char`.
#[test]
fn maybe_char_match() {
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
fn maybe_char_nonmatch() {
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
fn repeat_char_match() {
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
fn repeat_char_nonmatch() {
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

/// Tests `parse::prefix_radix_token`.
#[test]
fn prefix_radix_token_match() {
    assert_eq!(
        prefix_radix_token("0b1234abcd"),
        Ok(Success {
            value: "0b",
            token: "0b",
            rest: "1234abcd",
        }));

    assert_eq!(
        prefix_radix_token("0o1234abcd"),
        Ok(Success {
            value: "0o",
            token: "0o",
            rest: "1234abcd",
        }));

    assert_eq!(
        prefix_radix_token("0x1234abcd"),
        Ok(Success {
            value: "0x",
            token: "0x",
            rest: "1234abcd",
        }));
}

/// Tests `parse::prefix_radix_token`.
#[test]
fn prefix_radix_token_nonmatch() {
    assert_eq!(
        prefix_radix_token("1234abcd"),
        Err(Failure {
            context: "",
            rest: "1234abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


/// Tests `parse::value_radix` for a `u8` value.
#[test]
fn uint_value_u8_match() {
    assert_eq!(
        uint_value::<u8>("u8", 2)("1010abcd"),
        Ok(Success {
            value: 0b1010u8,
            token: "1010",
            rest: "abcd",
        }));

    assert_eq!(
        uint_value::<u8>("u8", 8)("170abcd"),
        Ok(Success {
            value: 0o170u8,
            token: "170",
            rest: "abcd",
        }));

    assert_eq!(
        uint_value::<u8>("u8", 10)("190abcd"),
        Ok(Success {
            value: 190u8,
            token: "190",
            rest: "abcd",
        }));

    assert_eq!(
        uint_value::<u8>("u8", 16)("AF abcd"),
        Ok(Success {
            value: 0xAFu8,
            token: "AF",
            rest: " abcd",
        }));
}


/// Tests `parse::uint` for a `u8` value.
#[test]
fn uint_u8_match() {
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
            value: 0xFFu8,
            token: "0xFf",
            rest: " abcd",
        }));
}

/// Tests `parse::uint` for a `u8` value.
#[test]
fn uint_u8_nonmatch() {
    assert_eq!(
        uint::<u8>("u8")("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0b20abcd"),
        Err(Failure {
            context: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0o80abcd"),
        Err(Failure {
            context: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0xG0abcd"),
        Err(Failure {
            context: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0x100Gabcd"),
        Err(Failure {
            context: "0x100",
            rest: "0x100Gabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::uint` for a `u16` value.
#[test]
fn uint_u16_match() {
    assert_eq!(
        uint::<u16>("u16")("0abcd"),
        Ok(Success {
            value: 0u16,
            token: "0",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u16>("u16")("0b10abcd"),
        Ok(Success {
            value: 0b10u16,
            token: "0b10",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u16>("u16")("0o70abcd"),
        Ok(Success {
            value: 0o70u16,
            token: "0o70",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u16>("u16")("0xFfFf abcd"),
        Ok(Success {
            value: 0xFFFFu16,
            token: "0xFfFf",
            rest: " abcd",
        }));
}

/// Tests `parse::uint` for a `u16` value.
#[test]
fn uint_u16_nonmatch() {
    assert_eq!(
        uint::<u16>("u16")("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0b20abcd"),
        Err(Failure {
            context: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0o80abcd"),
        Err(Failure {
            context: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0xG0abcd"),
        Err(Failure {
            context: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0x10000Gabcd"),
        Err(Failure {
            context: "0x10000",
            rest: "0x10000Gabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::uint` for a `u32` value.
#[test]
fn uint_u32_match() {
    assert_eq!(
        uint::<u32>("u32")("0abcd"),
        Ok(Success {
            value: 0u32,
            token: "0",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u32>("u32")("0b10abcd"),
        Ok(Success {
            value: 0b10u32,
            token: "0b10",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u32>("u32")("0o70abcd"),
        Ok(Success {
            value: 0o70u32,
            token: "0o70",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u32>("u32")("0xFfFfFfFf abcd"),
        Ok(Success {
            value: 0xFFFFFFFFu32,
            token: "0xFfFfFfFf",
            rest: " abcd",
        }));
}

/// Tests `parse::uint` for a `u32` value.
#[test]
fn uint_u32_nonmatch() {
    assert_eq!(
        uint::<u32>("u32")("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0b20abcd"),
        Err(Failure {
            context: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0o80abcd"),
        Err(Failure {
            context: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0xG0abcd"),
        Err(Failure {
            context: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0x100000000Gabcd"),
        Err(Failure {
            context: "0x100000000",
            rest: "0x100000000Gabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::uint` for a `u64` value.
#[test]
fn uint_u64_match() {
    assert_eq!(
        uint::<u64>("u64")("0abcd"),
        Ok(Success {
            value: 0u64,
            token: "0",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u64>("u64")("0b10abcd"),
        Ok(Success {
            value: 0b10u64,
            token: "0b10",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u64>("u64")("0o70abcd"),
        Ok(Success {
            value: 0o70u64,
            token: "0o70",
            rest: "abcd",
        }));

    assert_eq!(
        uint::<u64>("u64")("0xFfFfFfFfFfFfFfFf abcd"),
        Ok(Success {
            value: 0xFFFFFFFFFFFFFFFFu64,
            token: "0xFfFfFfFfFfFfFfFf",
            rest: " abcd",
        }));
}

/// Tests `parse::uint` for a `u64` value.
#[test]
fn uint_u64_nonmatch() {
    assert_eq!(
        uint::<u64>("u64")("abcd"),
        Err(Failure {
            context: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0b20abcd"),
        Err(Failure {
            context: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0o80abcd"),
        Err(Failure {
            context: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0xG0abcd"),
        Err(Failure {
            context: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0x10000000000000000Gabcd"),
        Err(Failure {
            context: "0x10000000000000000",
            rest: "0x10000000000000000Gabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::name`.
#[test]
fn name_match() {
    assert_eq!(
        name("xyz"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: "",
        }));

    assert_eq!(
        name("xyz abcd"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: " abcd",
        }));

    assert_eq!(
        name("xyz.abcd"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: ".abcd",
        }));

    assert_eq!(
        name("xyz,abcd"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: ",abcd",
        }));

    assert_eq!(
        name("xyz:abcd"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: ":abcd",
        }));

    assert_eq!(
        name("xyz-abcd"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: "-abcd",
        }));

    assert_eq!(
        name("xyz*abcd"),
        Ok(Success {
            value: "xyz",
            token: "xyz",
            rest: "*abcd",
        }));
}

/// Tests `parse::name`.
#[test]
fn name_nonmatch() {
    assert_eq!(
        name(" xyz"),
        Err(Failure {
            context: "",
            rest: " xyz",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name(".xyz"),
        Err(Failure {
            context: "",
            rest: ".xyz",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name(",xyz"),
        Err(Failure {
            context: "",
            rest: ",xyz",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name(":xyz"),
        Err(Failure {
            context: "",
            rest: ":xyz",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name("-xyz"),
        Err(Failure {
            context: "",
            rest: "-xyz",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        name("*xyz"),
        Err(Failure {
            context: "",
            rest: "*xyz",
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
            context: ":",
            rest: ":abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0b20abcd"),
        Err(Failure {
            context: ":0b",
            rest: ":0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0o80abcd"),
        Err(Failure {
            context: ":0o",
            rest: ":0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0xG0abcd"),
        Err(Failure {
            context: ":0x",
            rest: ":0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        index(":0x100000000 abcd"),
        Err(Failure {
            context: ":0x100000000",
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
            context: ":1",
            rest: ":1 .2.3abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position(":1.*.3abcd"),
        Err(Failure {
            context: ":1.",
            rest: ":1.*.3abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position(":1.2.0x10000 abcd"),
        Err(Failure {
            context: ":1.2.0x10000",
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
        cell_ref("xyz abcd"),
        Ok(Success {
            value: CellRef::Name("xyz".into()),
            token: "xyz",
            rest: " abcd",
        }));
}

/// Tests `parse::cell_ref`.
#[test]
fn cell_ref_group_match() {
    assert_eq!(
        cell_ref("xyz:12 abcd"),
        Ok(Success {
            value: CellRef::Group { group: "xyz".into(), idx: 12 },
            token: "xyz:12",
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
            context: ":1.2.",
            rest: ":1.2.abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position_selector(":1.2.0x1FFFF abcd"),
        Err(Failure {
            context: ":1.2.0x1FFFF",
            rest: ":1.2.0x1FFFF abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        position_selector(":1.**.3abcd"),
        Err(Failure {
            context: ":1.*",
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
            context: "-:0x1FFFFFFFF",
            rest: "-:0x1FFFFFFFF abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(index)("-: 10abcd"),
        Err(Failure {
            context: "-:",
            rest: "-: 10abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(index)("--: 10abcd"),
        Err(Failure {
            context: "-",
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
            context: "- ",
            rest: "- -: 10abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(position)(" -:0xFFFF.0xFFFF.0x1FFFF abcd"),
        Err(Failure {
            context: " -:0xFFFF.0xFFFF.0x1FFFF",
            rest: " -:0xFFFF.0xFFFF.0x1FFFF abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_group_match() {
    assert_eq!(
        range_suffix(group)("- xyz:0abcd"),
        Ok(Success {
            value: ("xyz", 0),
            token: "- xyz:0",
            rest: "abcd",
        }));

    assert_eq!(
        range_suffix(group)("- xyz:0xFFFFFFFF abcd"),
        Ok(Success {
            value: ("xyz", 0xFFFFFFFF),
            token: "- xyz:0xFFFFFFFF",
            rest: " abcd",
        }));
}

/// Tests `parse::range_suffix`.
#[test]
fn range_suffix_group_nonmatch() {
    assert_eq!(
        range_suffix(group)("- -xyz:0abcd"),
        Err(Failure {
            context: "- ",
            rest: "- -xyz:0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        range_suffix(group)(" -xyz:0xHabcd"),
        Err(Failure {
            context: " -xyz:0x",
            rest: " -xyz:0xHabcd",
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
            context: ":1-:0",
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
            context: ":1.2.3-:0.5.6",
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
        cell_selector("xyz:2abcd"),
        Ok(Success {
            value: CellSelector::Group { group: "xyz".into(), idx: 2 },
            token: "xyz:2",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_range_match() {
    assert_eq!(
        cell_selector("xyz:2-xyz:4abcd"),
        Ok(Success {
            value: CellSelector::GroupRange {
                group: "xyz".into(),
                low: 2,
                high: 4,
            },
            token: "xyz:2-xyz:4",
            rest: "abcd",
        }));

    assert_eq!(
        cell_selector("xyz:2-xyz:2abcd"),
        Ok(Success {
            value: CellSelector::Group { group: "xyz".into(), idx: 2 },
            token: "xyz:2-xyz:2",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_range_nonmatch() {
    assert_eq!(
        cell_selector("xyz:2-xyzt:4abcd"),
        Err(Failure {
            context: "xyz:2-xyzt:4",
            rest: "xyz:2-xyzt:4abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        cell_selector("xyz:2-xyz:1abcd"),
        Err(Failure {
            context: "xyz:2-xyz:1",
            rest: "xyz:2-xyz:1abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_group_all_match() {
    assert_eq!(
        cell_selector("xyz:*abcd"),
        Ok(Success {
            value: CellSelector::GroupAll("xyz".into()),
            token: "xyz:*",
            rest: "abcd",
        }));
}

/// Tests `parse::cell_selector`.
#[test]
fn cell_selector_name_match() {
    assert_eq!(
        cell_selector("xyz abcd"),
        Ok(Success {
            value: CellSelector::Name("xyz".into()),
            token: "xyz",
            rest: " abcd",
        }));
}

/// Tests `parse::cell_selection`.
#[test]
fn cell_selection_match() {
    assert_eq!(
        cell_selection("*, :0 , :3-:4, xyz*abcd"),
        Ok(Success {
            value: vec![
                CellSelector::All,
                CellSelector::Index(0),
                CellSelector::IndexRange { low: 3, high: 4 },
                CellSelector::Name("xyz".into()),
            ].into(),
            token: "*, :0 , :3-:4, xyz",
            rest: "*abcd",
        }));
}


////////////////////////////////////////////////////////////////////////////////
// Float parsing.
////////////////////////////////////////////////////////////////////////////////


/// Tests `parse::float` with `f32` values.
#[test]
fn float_f32_match() {
    assert_eq!(
        float::<f32>("f32")("0.0abcd"),
        Ok(Success {
            value: 0.0f32,
            token: "0.0",
            rest: "abcd",
        }));

    assert_eq!(
        float::<f32>("f32")("-110.0123141abcd"),
        Ok(Success {
            value: -110.0123141f32,
            token: "-110.0123141",
            rest: "abcd",
        }));

    assert_eq!(
        float::<f32>("f32")("-110.0123141eabcd"),
        Ok(Success {
            value: -110.0123141f32,
            token: "-110.0123141",
            rest: "eabcd",
        }));

    assert_eq!(
        float::<f32>("f32")("-110.0123141e1abcd"),
        Ok(Success {
            value: -110.0123141e1f32,
            token: "-110.0123141e1",
            rest: "abcd",
        }));
}
