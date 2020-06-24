////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse primitives.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::parse::Failure;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::repeat;
use crate::parse::Success;

// Standard library imports.
use std::convert::TryFrom;
use std::convert::TryInto as _;
use std::borrow::Cow;

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
    repeat(0, None, char_whitespace)(text)
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
    where T: TryFrom<u64>
{
    move |text| {
        let radix_prefix = maybe(radix_prefix)(text).unwrap();
        let radix: u32 = match radix_prefix.value {
            Some("0b") => 2,
            Some("0o") => 8,
            Some("0x") => 16,
            None => 10,
            _ => unreachable!(),
        };

        let digit = char_matching(|c| c.is_digit(radix) || c == '_');
        let digits = repeat(1, None, digit)(radix_prefix.rest)
            .with_parse_context(radix_prefix.token, text)
            .source_for(
                format!("{} integer digits with radix {}", int_type, radix))?;
        
        let context_span = text.len() - digits.rest.len();
        let context = &text[0..context_span];

        let digits_span = radix_prefix.rest.len() - digits.rest.len();
        let digits_text = &radix_prefix.rest[0..digits_span];

        let mut res: u64 = 0;
        let mut chars = digits_text.chars();
        while let Some(c) = chars.next() {
            if c == '_' { continue; }

            // TODO: Consider parsing all hex digits and emitting an error if
            // any remain. This should make error handling nicer.
            let val = u64::from(c.to_digit(radix).unwrap());
            
            match res.checked_mul(u64::from(radix)) {
                Some(x) => res = x,
                None => return Err(Failure {
                    context,
                    expected: "parse integer".into(),
                    source: Some(Box::new(ParseIntegerOverflow {
                        int_type: int_type.into(),
                        int_text: context.to_string().into(),
                        value: u64::from(res),
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
                        value: u64::from(res) + u64::from(val),
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
                    value: u64::from(res),
                })),
                rest: text,
            }),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// ParseIntegerOverflow
////////////////////////////////////////////////////////////////////////////////
/// An overflow error occurred while parsing an integer.
#[derive(Debug, Clone)]
pub struct ParseIntegerOverflow {
    /// The integer type.
    pub int_type: Cow<'static, str>,
    /// The integer text.
    pub int_text: Cow<'static, str>,
    /// The parsed value.
    pub value: u64,
}


impl std::fmt::Display for ParseIntegerOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "integer value {} ('{}' does not fit in type {}",
            self.value, self.int_text, self.int_type)
    }
}

impl std::error::Error for ParseIntegerOverflow {}
