////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser test suite.
////////////////////////////////////////////////////////////////////////////////

use crate::parse::*;
// use crate::Palette;
// use crate::cell::Cell;
// use crate::cell::Position;
// use crate::selection::CellSelector;
// use crate::selection::PositionSelector;
// use crate::cell::CellRef;

// use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////
// Primitives
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::char`.
#[test]
fn parse_char_match() {
    let char_res = char('a')("abcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "bcd");
    assert_eq!(char_res.into_value(), Some('a'));
}

/// Tests `parse::char`.
#[test]
fn parse_char_nonmatch() {
    let char_res = char('b')("abcd");
    assert!(char_res.is_err());
    assert_eq!(char_res.rest(), "abcd");
}

/// Tests `parse::char_in`.
#[test]
fn parse_char_in_match() {
    let char_in_res = char_in("cab")("abcd");
    assert!(char_in_res.is_ok());
    assert_eq!(char_in_res.rest(), "bcd");
    assert_eq!(char_in_res.into_value(), Some('a'));
}

/// Tests `parse::char_in`.
#[test]
fn parse_char_in_nonmatch() {
    let char_in_res = char_in("bdcbd")("abcd");
    assert!(char_in_res.is_err());
    assert_eq!(char_in_res.rest(), "abcd");
}


/// Tests `parse::char_matching`.
#[test]
fn parse_char_matching_match() {
    let char_matching_res = char_matching(|c| c == 'a')("abcd");
    assert!(char_matching_res.is_ok());
    assert_eq!(char_matching_res.rest(), "bcd");
    assert_eq!(char_matching_res.into_value(), Some('a'));
}

/// Tests `parse::char_matching`.
#[test]
fn parse_char_matching_nonmatch() {
    let char_matching_res = char_matching(|c| c == 'b')("abcd");
    assert!(char_matching_res.is_err());
    assert_eq!(char_matching_res.rest(), "abcd");
}


/// Tests `parse::whitespace`.
#[test]
fn parse_char_whitespace_match() {
    let char_whitespace_res = char_whitespace("\tabcd");
    assert!(char_whitespace_res.is_ok());
    assert_eq!(char_whitespace_res.rest(), "abcd");
    assert_eq!(char_whitespace_res.into_value(), Some('\t'));

    let char_whitespace_res = char_whitespace(" abcd");
    assert!(char_whitespace_res.is_ok());
    assert_eq!(char_whitespace_res.rest(), "abcd");
    assert_eq!(char_whitespace_res.into_value(), Some(' '));
}

/// Tests `parse::char_whitespace`.
#[test]
fn parse_char_whitespace_nonmatch() {

    let char_whitespace_res = char_whitespace("abcd");
    assert!(char_whitespace_res.is_err());
    assert_eq!(char_whitespace_res.rest(), "abcd");
}

////////////////////////////////////////////////////////////////////////////////
// Combinators
////////////////////////////////////////////////////////////////////////////////

/// Tests `maybe` on `parse::char`.
#[test]
fn parse_maybe_char_match() {
    let char_res = maybe(char('a'))("abcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "bcd");
    assert_eq!(char_res.into_value(), Some(Some('a')));
}

/// Tests `maybe` on `parse::char`.
#[test]
fn parse_maybe_char_nonmatch() {
    let char_res = maybe(char('b'))("abcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "abcd");
}

/// Tests `zero_or_more` on `parse::char`.
#[test]
fn parse_zero_or_more_char_match() {
    let char_res = zero_or_more(char_in("ab"))("abcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "cd");
    assert_eq!(char_res.into_value(), Some(2));
}

/// Tests `zero_or_more` on `parse::char`.
#[test]
fn parse_zero_or_more_char_nonmatch() {
    let char_res = zero_or_more(char_in("bc"))("abcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "abcd");
    assert_eq!(char_res.into_value(), Some(0));
}

/// Tests `one_or_more` on `parse::char`.
#[test]
fn parse_one_or_more_char_match() {
    let char_res = one_or_more(char_in("ab"))("abcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "cd");
    assert_eq!(char_res.into_value(), Some(2));
}

/// Tests `one_or_more` on `parse::char`.
#[test]
fn parse_one_or_more_char_nonmatch() {
    let char_res = one_or_more(char_in("bc"))("abcd");
    assert!(char_res.is_err());
    assert_eq!(char_res.rest(), "abcd");
}

/// Tests `repeat` on `parse::char`.
#[test]
fn parse_repeat_char_match() {
    let char_res = repeat(3, Some(5), char('a'))("aaaabcd");
    assert!(char_res.is_ok());
    assert_eq!(char_res.rest(), "bcd");
    assert_eq!(char_res.into_value(), Some(4));
}

/// Tests `repeat` on `parse::char`.
#[test]
fn parse_repeat_char_nonmatch() {
    let char_res = repeat(3, Some(5), char('a'))("aabcd");
    assert!(char_res.is_err());
    assert_eq!(char_res.rest(), "aabcd");
}

////////////////////////////////////////////////////////////////////////////////
// integers
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::radix_prefix`.
#[test]
fn parse_radix_prefix_match() {
    let radix_prefix_res = radix_prefix("0babcd");
    assert!(radix_prefix_res.is_ok());
    assert_eq!(radix_prefix_res.rest(), "abcd");
    assert_eq!(radix_prefix_res.into_value(), Some("0b"));

    let radix_prefix_res = radix_prefix("0oabcd");
    assert!(radix_prefix_res.is_ok());
    assert_eq!(radix_prefix_res.rest(), "abcd");
    assert_eq!(radix_prefix_res.into_value(), Some("0o"));

    let radix_prefix_res = radix_prefix("0xabcd");
    assert!(radix_prefix_res.is_ok());
    assert_eq!(radix_prefix_res.rest(), "abcd");
    assert_eq!(radix_prefix_res.into_value(), Some("0x"));
}

/// Tests `parse::radix_prefix`.
#[test]
fn parse_radix_prefix_nonmatch() {
    let radix_prefix_res = radix_prefix("0abcd");
    assert!(radix_prefix_res.is_err());
    assert_eq!(radix_prefix_res.rest(), "0abcd");
}

/// Tests `parse::uint` for a `u8` value.
#[test]
fn parse_uint_u8_match() {
    let uint_u8_res = uint::<u8>("u8")("0abcd");
    assert!(uint_u8_res.is_ok());
    assert_eq!(uint_u8_res.rest(), "abcd");
    assert_eq!(uint_u8_res.into_value(), Some(0u8));

    let uint_u8_res = uint::<u8>("u8")("0b10abcd");
    assert!(uint_u8_res.is_ok());
    assert_eq!(uint_u8_res.rest(), "abcd");
    assert_eq!(uint_u8_res.into_value(), Some(0b10u8));

    let uint_u8_res = uint::<u8>("u8")("0o70abcd");
    assert!(uint_u8_res.is_ok());
    assert_eq!(uint_u8_res.rest(), "abcd");
    assert_eq!(uint_u8_res.into_value(), Some(0o70u8));

    let uint_u8_res = uint::<u8>("u8")("0xF0 abcd");
    assert!(uint_u8_res.is_ok());
    assert_eq!(uint_u8_res.rest(), " abcd");
    assert_eq!(uint_u8_res.into_value(), Some(0xF0u8));
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

    let name_res = name("   ab   cd   ");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), "");
    assert_eq!(name_res.into_value(), Some("ab   cd"));

    let name_res = name("xyz .abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), ".abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz :abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), ":abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz *abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), "*abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz -abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), "-abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));

    let name_res = name("xyz ,abcd");
    assert!(name_res.is_ok());
    assert_eq!(name_res.rest(), ",abcd");
    assert_eq!(name_res.into_value(), Some("xyz"));
}

/// Tests `parse::name`.
#[test]
fn parse_name_nonmatch() {
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
