////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser helpers.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(unused_imports)]
#![allow(missing_docs)]


// Standard library imports.
use std::convert::Into;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::borrow::Borrow;


////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
pub type ParseResult<'t, V> = Result<Success<'t, V>, Failure<'t>>;


pub trait ParseResultExt<'t, V>: Sized {
    /// Adds context and expectation detail to a failed parse.
    fn with_parse_context<E>(self, context: &'t str, expected: E) -> Self
        where E: Into<Cow<'static, str>>;

    /// Returns the value produced by a successful parse, or None if the parse
    /// was not successful.
    fn value(self) -> Option<V>;

    /// Returns the token associated with a successful parse, or None if the
    /// parse was not successful.
    fn token(&self) -> Option<&'t str>;

    /// Returns the remaining parse text.
    fn rest(&self) -> &'t str;

    /// Applies the given closure to the parsed value. Will only be called if
    /// the parse was successful.
    fn map_value<F, U>(self, f: F) -> ParseResult<'t, U>
        where F: FnOnce(V) -> U;

    /// Discards the parsed value, replacing it with the parsed text.
    fn tokenize_value(self) -> ParseResult<'t, &'t str> {
        let token = self.token();
        self.map_value(|_| token.unwrap())
    }

    /// Discards the parsed value.
    fn discard_value(self) -> ParseResult<'t, ()> {
        self.map_value(|_| ())
    }

    /// Converts a parse success into a failre if the end of the text input has
    /// not been reached.
    fn expect_end_of_text(self) -> ParseResult<'t, V>;

    /// Converts a parse failure into a success and vice versa.
    fn expect_failure(self) -> ParseResult<'t, &'t str>;
}

impl<'t, V> ParseResultExt<'t, V> for ParseResult<'t, V> {
    fn with_parse_context<E>(self, context: &'t str, expected: E) -> Self
        where E: Into<Cow<'static, str>>
    {
        self.map_err(|failure| {
            let rest = failure.rest;
            let found = &context[..0];
            Failure {
                context,
                expected: expected.into(),
                found,
                source: Some(Box::new(failure.to_owned())),
                rest,
            }
        })
    }

    fn value(self) -> Option<V> {
        self.ok().map(|success| success.value)
    }

    fn token(&self) -> Option<&'t str> {
        self.as_ref().map(|success| success.token).ok()
    }

    fn rest(&self) -> &'t str {
        match self {
            Ok(success) => success.rest,
            Err(failure) => failure.rest,
        }
    }

    fn map_value<F, U>(self, f: F) -> ParseResult<'t, U>
        where F: FnOnce(V) -> U
    {
        self.map(|success| Success {
            value: (f)(success.value),
            token: success.token,
            rest: success.rest,
        })
    }

    fn expect_end_of_text(self) -> ParseResult<'t, V> {
        match self {
            Ok(success) if success.rest.is_empty() => Ok(success),
            Ok(success) => Err(Failure {
                context: success.token,
                expected: "end-of-text".into(),
                found: success.rest,
                source: None,
                rest: success.rest,
            }),
            Err(failure) => Err(failure),
        }
    }

    fn expect_failure(self) -> ParseResult<'t, &'t str> {
        match self {
            Ok(success) => Err(Failure {
                context: success.token,
                expected: "parse failure".into(),
                found: success.token,
                source: None,
                rest: success.rest,
            }),
            Err(failure) => Ok(Success {
                value: failure.context,
                token: failure.context,
                rest: failure.rest,
            })
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Success and Failure
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Success<'t, V> {
    /// The parsed value.
    pub value: V,
    /// The parsed text.
    pub token: &'t str,
    /// The remainder of the parsable text.
    pub rest: &'t str,
}


#[derive(Debug)]
pub struct Failure<'t> {
    /// The previously successful parse text. Usually non-empty for any parse
    /// function which employs sequential sub-parsers.
    pub context: &'t str,
    /// The expected result of the failing parse. Recommended to be a literal,
    /// function name, or contextual description.
    pub expected: Cow<'static, str>,
    /// The text at the failed parse location.
    pub found: &'t str,
    /// The parse failure that caused this one.
    pub source: Option<Box<dyn std::error::Error + 'static>>,
    /// The full usable remainder of the parse text, used for error recovery.
    pub rest: &'t str,
}

impl<'t> Failure<'t> {
    fn to_owned(self) -> FailureOwned {
        FailureOwned {
            context: self.context.to_owned(),
            expected: self.expected,
            found: self.found.to_owned(),
            source: self.source,
        }
    }
}

impl<'t> std::fmt::Display for Failure<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: expected {}", self.expected)?;
        if !self.found.is_empty() {
            write!(f, ", found {}", self.found)?;
        }
        if !self.context.is_empty() {
            write!(f, ", after successful parse of {}", self.context)?;
        }
        Ok(())
    }
}

impl<'t> std::error::Error for Failure<'t> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| src.as_ref())
    }
}


#[derive(Debug)]
pub struct FailureOwned {
    /// The previously successful parse text. Usually non-empty for any parse
    /// function which employs sequential sub-parsers.
    pub context: String,
    /// The expected result of the failing parse. Recommended to be a literal,
    /// function name, or contextual description.
    pub expected: Cow<'static, str>,
    /// The text at the failed parse location.
    pub found: String,
    /// The parse failure that caused this one.
    pub source: Option<Box<dyn std::error::Error + 'static>>,
}

impl std::fmt::Display for FailureOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: expected {}", self.expected)?;
        if !self.found.is_empty() {
            write!(f, ", found {}", self.found)?;
        }
        if !self.context.is_empty() {
            write!(f, ", after successful parse of {}", self.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for FailureOwned {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| src.as_ref())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parser combinators.
////////////////////////////////////////////////////////////////////////////////
/// Repeats a parse until it fails, then returns the last successfully parsed
/// value.
pub fn zero_or_more<'t, F, V>(text: &'t str, mut parser: F)
    -> ParseResult<'t, Option<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    let mut result = Ok(Success {
        value: None,
        token: &text[..0],
        rest: text,
    });
    let mut rest = text;
    loop {
        match (parser)(rest) {
            Ok(success) => {
                rest = success.rest;
                result = Ok(success).map_value(Some);
            }
            Err(failure) => break,
        }
    }
    result
}

/// Attempts a parse, returning its value if it succeeds.
pub fn maybe<'t, F, V>(text: &'t str, mut parser: F)
    -> ParseResult<'t, Option<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    match (parser)(text) {
        Ok(success) => Ok(success).map_value(Some),
        Err(failure) => Ok(Success {
            value: None,
            token: &text[..0],
            rest: text,
        }),
    }
}

/// Repeats a parse until it fails, then returns the last successfully parsed
/// value.
pub fn one_or_more<'t, F, V>(text: &'t str, mut parser: F)
    -> ParseResult<'t, V>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    let mut result = (parser)(text)
        .with_parse_context(&text[..0], "one or more")?;
    let mut rest = result.rest;
    loop {
        match (parser)(rest) {
            Ok(success) => {
                rest = success.rest;
                result.value = success.value;
            }
            Err(failure) => break,
        }
    }
    Ok(result)
}

/// Repeats a parse until it fails, chaining the successful parse results into
/// each other using the given function.
pub fn one_or_more_chained<'t, F, V, G>(
    text: &'t str,
    mut parser: F,
    mut chain: G)
    -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(Success<'t, V>, Success<'t, V>) -> Success<'t, V>,
{
    let mut result = (parser)(text)
        .with_parse_context(&text[..0], "one or more")?;
    let mut rest = result.rest;
    loop {
        match (parser)(rest) {
            Ok(success) => {
                rest = success.rest;
                result = (chain)(result, success);
            }
            Err(failure) => break,
        }
    }
    Ok(result)
}

// Truncate context.
// Repeat.
// Try other on failure.
// Try next on success.
// Invert success and failure.

////////////////////////////////////////////////////////////////////////////////
// Primitive parsers.
////////////////////////////////////////////////////////////////////////////////

// Parses the specified char from the beginning of the text.
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

// Parses any single char in the given string from the beginning of the text.
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

// Parses any single char in the given string from the beginning of the text.
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

// Parses the specified char from the beginning of the text.
pub fn whitespace<'t>(text: &'t str) -> ParseResult<'t, char> {
    char_matching(text, |c| c.is_whitespace())
        .with_parse_context(&text[..0], "whitespace char")
}


////////////////////////////////////////////////////////////////////////////////
// Advanced parsers.
////////////////////////////////////////////////////////////////////////////////

/// Parses an integer radix prefix.
fn radix_prefix<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
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
            found: &text[0..],
            source: None,
            rest: &text[0..],
        })
    }
}

/// Parses an unsigned integer with optional radix prefix.
pub fn parse_uint<'t, T>(text: &'t str, int_type: &'static str)
    -> ParseResult<'t, T>
    where T: TryFrom<u32>
{
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
        .with_parse_context(radix_prefix.token, "integer digits")?;
    

    let context_span = text.len() - digits.rest.len();
    let context = &text[0..context_span];

    let digits_span = radix_prefix.rest.len() - digits.rest.len();
    let digits_text = &radix_prefix.rest[0..digits_span];

    let mut res: u32 = 0;
    let mut chars = digits_text.chars();
    while let Some(c) = chars.next() {
        if c == '_' { continue; }

        let val = c.to_digit(radix).unwrap();
        
        match res.checked_mul(10) {
            Some(x) => res = x,
            None => return Err(Failure {
                context,
                expected: "parse integer".into(),
                found: &text[0..],
                source: Some(Box::new(ParseIntegerOverflow {
                    int_type: int_type.into(),
                    int_text: context.to_string().into(),
                })),
                rest: &text[context_span..],
            }),
        }
        match res.checked_add(val) {
            Some(x) => res = x,
            None => return Err(Failure {
                context,
                expected: "parse integer".into(),
                found: &text[0..],
                source: Some(Box::new(ParseIntegerOverflow {
                    int_type: int_type.into(),
                    int_text: context.to_string().into(),
                })),
                rest: &text[context_span..],
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
            found: &text[0..],
            source: Some(Box::new(ParseIntegerOverflow {
                int_type: int_type.into(),
                int_text: context.to_string().into(),
            })),
            rest: &text[context_span..],
        }),
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse errors.
////////////////////////////////////////////////////////////////////////////////

/// An overflow error occurred while parsing an integer.
#[derive(Debug, Clone)]
pub struct ParseIntegerOverflow {
    /// The integer type.
    pub int_type: Cow<'static, str>,
    /// The integer text.
    pub int_text: Cow<'static, str>,
}


impl std::fmt::Display for ParseIntegerOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the integer value '{}' does not fit in type {}",
            self.int_text, self.int_type)
        
    }
}

impl std::error::Error for ParseIntegerOverflow {}
