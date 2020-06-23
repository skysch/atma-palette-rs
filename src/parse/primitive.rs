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
// Primitive parsers.
////////////////////////////////////////////////////////////////////////////////

/// Parses the specified `char`.
pub fn char<'t>(text: &'t str, c: char) -> ParseResult<'t, char> {
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
            found: {
                let len = text.chars().next().map(char::len_utf8).unwrap_or(0);
                &text[..len]
            },
            source: None,
            rest: text,
        })
    }
}

/// Parses any single `char` in the given string.
pub fn char_in<'t>(text: &'t str, opts: &str) -> ParseResult<'t, char> {
    let mut opt_chars = opts.chars();
    while let Some(c) = opt_chars.next() {
        if let Ok(success) = char(text, c) {
             return Ok(success);
        }
    }

    Err(Failure {
        context: "",
        expected: format!("One of {}", opts).into(),
        found: {
            let len = text.chars().next().map(char::len_utf8).unwrap_or(0);
            &text[..len]
        },
        source: None,
        rest: text,
    })
}

/// Parses a `char` if it satisfies the given predicate.
pub fn char_matching<'t, F>(text: &'t str, mut f: F)
    -> ParseResult<'t, char>
    where F: FnMut(char) -> bool
{
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
        found: {
            let len = text.chars().next().map(char::len_utf8).unwrap_or(0);
            &text[..len]
        },
        source: None,
        rest: text,
    })
}

/// Parses a whitespace `char`.
pub fn whitespace<'t>(text: &'t str) -> ParseResult<'t, char> {
    char_matching(text, |c| c.is_whitespace())
        .with_parse_context(&text[..0], text)
        .source_for("whitespace char")
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
            found: text,
            source: None,
            rest: text,
        })
    }
}

/// Parses an unsigned integer with optional radix prefix.
pub fn uint<'t, T>(text: &'t str, int_type: &'static str)
    -> ParseResult<'t, T>
    where T: TryFrom<u32>
{
    // TODO: Fix this to work for u64, u128 values.
    
    let radix_prefix = maybe(text, |t| radix_prefix(t)).unwrap();
    let radix: u32 = match radix_prefix.value {
        Some("0b") => 2,
        Some("0o") => 8,
        Some("0x") => 16,
        None => 10,
        _ => unreachable!(),
    };

    let digits = one_or_more(
            radix_prefix.rest,
            |t| char_matching(t, |c| c.is_digit(radix) || c == '_'))
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
                found: text,
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
                found: text,
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
            found: text,
            source: Some(Box::new(ParseIntegerOverflow {
                int_type: int_type.into(),
                int_text: context.to_string().into(),
            })),
            rest: text,
        }),
    }
}
