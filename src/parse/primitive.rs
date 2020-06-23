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
        .with_parse_context(&text[..0], "whitespace char")
}

