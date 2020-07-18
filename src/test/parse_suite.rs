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
        token: &input[2..5],
        rest: &input[2..],
        // These fields are unchecked:
        expected: "".into(), source: None,
    };

    assert_eq!(
        l.join_failure(r, input),
        Failure {
            token: &input[..5],
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
            token: "",
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
            token: "",
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
            token: "",
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
            token: "",
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
            token: "",
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
            token: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}


/// Tests `parse::literal_ignore_ascii_case`.
#[test]
fn literal_ignore_ascii_case_match() {
    assert_eq!(
        literal_ignore_ascii_case("")("abcd"),
        Ok(Success {
            value: "",
            token: "",
            rest: "abcd",
        }));

    assert_eq!(
        literal_ignore_ascii_case("abc")("AbCd"),
        Ok(Success {
            value: "AbC",
            token: "AbC",
            rest: "d",
        }));
}


/// Tests `parse::escaped_string`.
#[test]
fn escaped_string_match() {
    assert_eq!(
        escaped_string(
                rust_string_open,
                rust_string_close,
                rust_string_escape)
            ("\"some text\"abcd"),
        Ok(Success {
            value: "some text".into(),
            token: "\"some text\"",
            rest: "abcd",
        }));

    assert_eq!(
        escaped_string(
                rust_string_open,
                rust_string_close,
                rust_string_escape)
            ("\"some \\nte\\\\xt\"abcd"),
        Ok(Success {
            value: "some \nte\\xt".into(),
            token: "\"some \\nte\\\\xt\"",
            rest: "abcd",
        }));

    assert_eq!(
        escaped_string(
                rust_string_open,
                rust_string_close,
                rust_string_escape)
            ("r###\"some text\"###abcd"),
        Ok(Success {
            value: "some text".into(),
            token: "r###\"some text\"###",
            rest: "abcd",
        }));

    assert_eq!(
        escaped_string(
                script_string_open,
                script_string_close,
                script_string_escape)
            ("'some \"text'abcd"),
        Ok(Success {
            value: "some \"text".into(),
            token: "'some \"text'",
            rest: "abcd",
        }));

    assert_eq!(
        escaped_string(
                script_string_open,
                script_string_close,
                script_string_escape)
            ("'some \\'text'abcd"),
        Ok(Success {
            value: "some 'text".into(),
            token: "'some \\'text'",
            rest: "abcd",
        }));

    assert_eq!(
        escaped_string(
                script_string_open,
                script_string_close,
                script_string_escape)
            ("\"some \\'text\"abcd"),
        Ok(Success {
            value: "some \\'text".into(),
            token: "\"some \\'text\"",
            rest: "abcd",
        }));
}


/// Tests `parse::escaped_string`.
#[test]
fn escaped_string_nonmatch() {
    assert_eq!(
        escaped_string(
                rust_string_open,
                rust_string_close,
                rust_string_escape)
            ("\"some textabcd"),
        Err(Failure {
            token: "\"some textabcd",
            rest: "\"some textabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        escaped_string(
                rust_string_open,
                rust_string_close,
                rust_string_escape)
            ("some text\"abcd"),
        Err(Failure {
            token: "",
            rest: "some text\"abcd",
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
            token: "aa",
            rest: "aabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `postfix` on `parse::char`.
#[test]
fn postfix_char_match() {
    assert_eq!(
        postfix(char('a'), char('b'))("abcd"),
        Ok(Success {
            value: 'a',
            token: "ab",
            rest: "cd",
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
            token: "",
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
            token: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0b20abcd"),
        Err(Failure {
            token: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0o80abcd"),
        Err(Failure {
            token: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0xG0abcd"),
        Err(Failure {
            token: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u8>("u8")("0x100Gabcd"),
        Err(Failure {
            token: "0x100",
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
            token: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0b20abcd"),
        Err(Failure {
            token: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0o80abcd"),
        Err(Failure {
            token: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0xG0abcd"),
        Err(Failure {
            token: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u16>("u16")("0x10000Gabcd"),
        Err(Failure {
            token: "0x10000",
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
            token: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0b20abcd"),
        Err(Failure {
            token: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0o80abcd"),
        Err(Failure {
            token: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0xG0abcd"),
        Err(Failure {
            token: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u32>("u32")("0x100000000Gabcd"),
        Err(Failure {
            token: "0x100000000",
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
            token: "",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0b20abcd"),
        Err(Failure {
            token: "0b",
            rest: "0b20abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0o80abcd"),
        Err(Failure {
            token: "0o",
            rest: "0o80abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0xG0abcd"),
        Err(Failure {
            token: "0x",
            rest: "0xG0abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        uint::<u64>("u64")("0x10000000000000000Gabcd"),
        Err(Failure {
            token: "0x10000000000000000",
            rest: "0x10000000000000000Gabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
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
        float::<f32>("f32")("-110.0123141e1abcd"),
        Ok(Success {
            value: -110.0123141e1f32,
            token: "-110.0123141e1",
            rest: "abcd",
        }));
}

/// Tests `parse::float` with `f32` values.
#[test]
fn float_f32_nonmatch() {
    assert_eq!(
        float::<f32>("f32")("-110.0123141eabcd"),
        Err(Failure {
            token: "-110.0123141e",
            rest: "-110.0123141eabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}
