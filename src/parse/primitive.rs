////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse primitives.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::parse::*;

// Standard library imports.
use std::borrow::Borrow;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::convert::Into;
use std::convert::TryFrom;
use std::convert::TryInto;

////////////////////////////////////////////////////////////////////////////////
// Char parsing.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which parses the specified `char`.
pub fn char<'t>(c: char) -> impl FnMut(&'t str) -> ParseResult<'t, char> {
    move |text| {
        if text.starts_with(c) {
            Ok(Success { 
                value: c,
                token: &text[..c.len_utf8()],
                rest: &text[c.len_utf8()..],
            })
        } else {
            Err(Failure {
                context: "",
                expected: c.to_string().into(),
                source: None,
                rest: text,
            })
        }
    }
}

/// Returns a parser which parses any single `char` in the given string.
pub fn char_in<'t, 'o: 't>(opts: &'o str)
    -> impl FnMut(&'t str) -> ParseResult<'t, char>
{
    move |text| {
        let mut opt_chars = opts.chars();
        while let Some(c) = opt_chars.next() {
            if let Ok(success) = char(c)(text) {
                 return Ok(success);
            }
        }

        Err(Failure {
            context: "",
            expected: format!("one of {}", opts).into(),
            source: None,
            rest: text,
        })
    }
}

/// Returns a parser which parses a `char` if it satisfies the given predicate.
pub fn char_matching<'t, F>(mut f: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, char>
    where F: FnMut(char) -> bool
{
    move |text| {
        if let Some(c) = text.chars().next() {
            if (f)(c) {
                return Ok(Success { 
                    value: c,
                    token: &text[..c.len_utf8()],
                    rest: &text[c.len_utf8()..],
                });
            }
        }

        Err(Failure {
            context: "",
            expected: "char satisfying predicate".into(),
            source: None,
            rest: text,
        })
    }
}

/// Parses a whitespace `char`.
pub fn char_whitespace<'t>(text: &'t str) -> ParseResult<'t, char> {
    char_matching(char::is_whitespace)(text)
        .with_parse_context("", text)
        .source_for("whitespace char")
}

////////////////////////////////////////////////////////////////////////////////
// String parsing.
////////////////////////////////////////////////////////////////////////////////

/// Parses any amount of whitespace.
pub fn whitespace<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    zero_or_more(char_whitespace)(text)
        .tokenize_value()
        .with_parse_context("", text)
        .source_for("whitespace")
}


////////////////////////////////////////////////////////////////////////////////
// Integer parseing.
////////////////////////////////////////////////////////////////////////////////

/// Parses an integer radix prefix.
pub fn radix_prefix<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    if  text.starts_with("0b") || 
        text.starts_with("0o") ||
        text.starts_with("0x") 
    {
        Ok(Success { 
            value: &text[0..2],
            token: &text[0..2],
            rest: &text[2..],
        })
    } else {
        Err(Failure { 
            context: "",
            expected: "0[box]".into(),
            source: None,
            rest: text,
        })
    }
}

/// Returns a parser which parses an unsigned integer with optional radix
/// prefix.
pub fn uint<'t, T>(int_type: &'static str)
    -> impl FnMut(&'t str) -> ParseResult<'t, T>
    where T: TryFrom<u32>
{
    move |text| {
        // TODO: Fix this to work for u64, u128 values.
        
        let radix_prefix = maybe(radix_prefix)(text).unwrap();
        let radix: u32 = match radix_prefix.value {
            Some("0b") => 2,
            Some("0o") => 8,
            Some("0x") => 16,
            None => 10,
            _ => unreachable!(),
        };

        let digit = char_matching(|c| c.is_digit(radix) || c == '_');
        let digits = one_or_more(digit)(radix_prefix.rest)
            .with_parse_context(radix_prefix.token, text)
            .source_for(
                format!("{} integer digits with radix {}", int_type, radix))?;
        
        let context_span = text.len() - digits.rest.len();
        let context = &text[0..context_span];

        let digits_span = radix_prefix.rest.len() - digits.rest.len();
        let digits_text = &radix_prefix.rest[0..digits_span];

        let mut res: u32 = 0;
        let mut chars = digits_text.chars();
        while let Some(c) = chars.next() {
            if c == '_' { continue; }

            // TODO: Consider parsing all hex digits and emitting an error if any
            // remain. This should make error handling nicer.
            let val = c.to_digit(radix).unwrap();
            
            match res.checked_mul(radix) {
                Some(x) => res = x,
                None => return Err(Failure {
                    context,
                    expected: "parse integer".into(),
                    source: Some(Box::new(ParseIntegerOverflow {
                        int_type: int_type.into(),
                        int_text: context.to_string().into(),
                    })),
                    rest: text,
                }),
            }
            match res.checked_add(val) {
                Some(x) => res = x,
                None => return Err(Failure {
                    context,
                    expected: "parse integer".into(),
                    source: Some(Box::new(ParseIntegerOverflow {
                        int_type: int_type.into(),
                        int_text: context.to_string().into(),
                    })),
                    rest: text,
                }),
            }
        }
        
        match res.try_into() {
            Ok(res) => Ok(Success {
                value: res,
                token: &text[..context_span],
                rest: &text[context_span..],
            }),
            Err(_) => Err(Failure {
                context,
                expected: "parse integer".into(),
                source: Some(Box::new(ParseIntegerOverflow {
                    int_type: int_type.into(),
                    int_text: context.to_string().into(),
                })),
                rest: text,
            }),
        }
    }
}
