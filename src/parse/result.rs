////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse results.
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
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'t, V> = Result<Success<'t, V>, Failure<'t>>;

// Truncate context.
// Try other on failure.
// Try next on success.

/// Extension trait for parse results.
pub trait ParseResultExt<'t, V>: Sized {
    /// Converts the result into a source using the given expected parse
    /// description.
    fn source_for<E>(self, expected: E) -> Self
        where E: Into<Cow<'static, str>>;

    /// Sets the parse context and resume point for a failed parse.
    fn with_parse_context(self, context: &'t str, rest: &'t str) -> Self;

    /// Returns a refernce to the value produced by a successful parse, or None
    /// if the parse was not successful.
    fn value(&self) -> Option<&V>;

    /// Consumes the result, returning the value produced by a successful parse,
    /// or None if the parse was not successful.
    fn into_value(self) -> Option<V>;

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
    fn source_for<E>(self, expected: E) -> Self
        where E: Into<Cow<'static, str>>
    {
        self.map_err(|failure| {
            let context = failure.context;
            let rest = failure.rest;
            Failure {
                context,
                expected: expected.into(),
                source: Some(Box::new(failure.to_owned())),
                rest,
            }
        })
    }

    fn with_parse_context(self, context: &'t str, rest: &'t str) -> Self {
        self.map_err(|mut failure| {
            failure.context = context;
            failure.rest = rest;
            failure
        })
    }

    fn value(&self) -> Option<&V> {
        self.as_ref().ok().map(|success| &success.value)
    }

    fn into_value(self) -> Option<V> {
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
/// A struct representing a successful parse.
#[derive(Debug)]
pub struct Success<'t, V> {
    /// The parsed value.
    pub value: V,
    /// The parsed text.
    pub token: &'t str,
    /// The remainder of the parsable text.
    pub rest: &'t str,
}

impl<'t, V> Success<'t, V> {
    /// Applies the given closure to the parsed value and returns the result.
    fn map_value<F, U>(self, f: F) -> Success<'t, U>
        where F: FnOnce(V) -> U
    {
        Success {
            value: (f)(self.value),
            token: self.token,
            rest: self.rest,
        }
    }

    /// Discards the parsed value.
    pub fn discard_value(self) -> Success<'t, ()> {
        Success {
            value: (),
            token: self.token,
            rest: self.rest,
        }
    }

    /// Discards the parsed value, replacing it with the parsed text.
    pub fn tokenize_value(self) -> Success<'t, &'t str> {
        Success {
            value: self.token,
            token: self.token,
            rest: self.rest,
        }
    }

    /// Joins two sequential successful parse results together, discardin their
    /// values.
    pub fn join<U>(self, other: Success<'t, U>, base: &'t str)
        -> Success<'t, ()>
    {
        Success {
            value: (),
            token: &base[..self.token.len() + other.token.len()],
            rest: other.rest,
        }
    }

    /// Joins two sequential successful parse results together, tokenizing their
    /// values.
    pub fn join_tokenize<U>(self, other: Success<'t, U>, base: &'t str)
        -> Success<'t, &'t str>
    {
        let token = &base[..self.token.len() + other.token.len()];
        Success {
            value: token,
            token,
            rest: other.rest,
        }
    }

    /// Joins two sequential successful parse results together, combining values
    /// with the given function.
    pub fn join_with<F, U, T>(self, other: Success<'t, U>, base: &'t str, f: F)
        -> Success<'t, T>
        where F: FnOnce(V, U) -> T
    {
        Success {
            value: (f)(self.value, other.value),
            token: &base[..self.token.len() + other.token.len()],
            rest: other.rest,
        }
    }
}


/// A struct representing a failed parse with borrowed data.
#[derive(Debug)]
pub struct Failure<'t> {
    /// The parsable text. This is expected to contain any successfully parsed
    /// text, and optionally include any text which should be skipped if the
    /// parse is to recover from errors.
    pub context: &'t str,
    /// The expected result of the failing parse. Recommended to be a literal,
    /// function name, or description of the context.
    pub expected: Cow<'static, str>,
    /// The parse failure that caused this one.
    pub source: Option<Box<dyn std::error::Error + 'static>>,
    /// The remainder of the parsable text. Failed parses should return their
    /// exact input text.
    pub rest: &'t str,
}

impl<'t> Failure<'t> {
    pub fn rest_continuing(&self) -> &'t str {
        &self.rest[self.context.len()..]
    }

    /// Converts a borrowed `Failure` into a `FailureOwned`.
    fn to_owned(self) -> FailureOwned {
        FailureOwned {
            context: self.context.to_owned(),
            expected: self.expected,
            source: self.source,
        }
    }
}

impl<'t> std::fmt::Display for Failure<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: expected {}", self.expected)?;
        if !self.context.is_empty() {
            write!(f, ", found {}", self.context)?;
        }
        Ok(())
    }
}

impl<'t> std::error::Error for Failure<'t> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| src.as_ref())
    }
}

/// A struct representing a failed parse with owned data.
///
/// Similar to [`Failure`], except this  version owns all of its data, and can
/// thus  be used as an [`Error`] [`source`].
///
/// [`Failure`]: struct.Failure.html
/// [`Error`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html
/// [`source`]: https://doc.rust-lang.org/stable/std/error/trait.Error.html#method.source
#[derive(Debug)]
pub struct FailureOwned {
    /// The previously successful parse text. Usually non-empty for any parse
    /// function which employs sequential sub-parsers.
    pub context: String,
    /// The expected result of the failing parse. Recommended to be a literal,
    /// function name, or contextual description.
    pub expected: Cow<'static, str>,
    /// The parse failure that caused this one.
    pub source: Option<Box<dyn std::error::Error + 'static>>,
}

impl std::fmt::Display for FailureOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: expected {}", self.expected)?;
        if !self.context.is_empty() {
            write!(f, ", found {}", self.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for FailureOwned {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| src.as_ref())
    }
}
