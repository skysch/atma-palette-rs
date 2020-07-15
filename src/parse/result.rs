////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser results.
////////////////////////////////////////////////////////////////////////////////

// Standard library imports.
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::convert::Into;



////////////////////////////////////////////////////////////////////////////////
// ParseResult
////////////////////////////////////////////////////////////////////////////////
/// The result of a parse attempt.
pub type ParseResult<'t, V> = Result<Success<'t, V>, Failure<'t>>;


////////////////////////////////////////////////////////////////////////////////
// ParseResultExt
////////////////////////////////////////////////////////////////////////////////
/// Extension trait for parse results.
pub trait ParseResultExt<'t, V>: Sized {
    /// Converts a parse success into a failre if the end of the text input has
    /// not been reached.
    fn end_of_text(self) -> ParseResult<'t, V>;

    /// Discards parse data and returns the parse result or owned error.
    fn finish(self) -> Result<V, FailureOwned>;

    /// Converts any Err result into an parse failure source.
    ///
    /// The given discription will become the new parse description.
    ///
    /// It is recommended to call this before any calls to `with_join_context`
    /// or `with_new_context` to ensure the subparse context is maintained in
    /// the failure source.
    fn source_for<E>(self, expected: E) -> Self
        where E: Into<Cow<'static, str>>;

    /// Sets the context for a failed parse by extending back to include a
    /// previously successful parse.
    ///
    /// This is typically used to set establish a recovery point before a
    /// previously successful parse.
    fn with_join_context<U>(self, success: &Success<'t, U>, text: &'t str)
        -> ParseResult<'t, V>;

    /// Sets a new context for a failed parse.
    ///
    /// This is typically used by a parser combinator to establish a recovery
    /// point before any subparser calls. The `context` argument sets the context directly, and should only be
    /// non-empty if there is a known recovery point for the parse.
    fn with_new_context(self, context: &'t str, text: &'t str)
        -> ParseResult<'t, V>;

    /// Sets the context for a parse by combining it with a previously
    /// successful parse.
    fn with_join_previous<U>(self, success: Success<'t, U>, text: &'t str)
        -> ParseResult<'t, V>;

    /// Applies the given closure to the parsed value, causing the parse to fail
    /// if the closure is Err. Will only be called if the parse was successful.
    fn convert_value<F, U, E, T>(self, expected: T, f: F) -> ParseResult<'t, U>
        where
            T: Into<Cow<'static, str>>,
            F: FnOnce(V) -> Result<U, E>,
            E: std::error::Error + Send + Sync + 'static;

    /// Applies the given closure to the parsed value. Will only be called if
    /// the parse was successful.
    fn map_value<F, U>(self, f: F) -> ParseResult<'t, U>
        where F: FnOnce(V) -> U;

    /// Discards the parsed value, replacing it with the parsed text.
    fn tokenize_value(self) -> ParseResult<'t, &'t str>;

    /// Discards the parsed value.
    fn discard_value(self) -> ParseResult<'t, ()> {
        self.map_value(|_| ())
    }
}

impl<'t, V> ParseResultExt<'t, V> for ParseResult<'t, V> {
    fn end_of_text(self) -> ParseResult<'t, V> {
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

    fn finish(self) -> Result<V, FailureOwned> {
        self.map(|suc| suc.value)
            .map_err(Failure::to_owned)
    }

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

    fn with_join_context<U>(self, success: &Success<'t, U>, text: &'t str)
        -> ParseResult<'t, V>
    {
        self.map_err(|failure| success.join_failure(failure, text))
    }

    fn with_new_context(self, context: &'t str, text: &'t str)
        -> ParseResult<'t, V>
    {
        self.map_err(|mut failure| {
            failure.context = context;
            failure.rest = text;
            failure
        })
    }

    fn with_join_previous<U>(self, success: Success<'t, U>, text: &'t str)
        -> ParseResult<'t, V>
    {
        self.map_err(|failure| success.join_failure(failure, text))
            .map(|curr| success.join_with(curr, text, |_, v| v))
    }

    fn convert_value<F, U, E, T>(self, expected: T, f: F) -> ParseResult<'t, U>
        where
            T: Into<Cow<'static, str>>,
            F: FnOnce(V) -> Result<U, E>,
            E: std::error::Error + Send + Sync + 'static
    {
        match self {
            Ok(success) => match (f)(success.value) {
                Ok(value) => Ok(Success {
                    value,
                    token: success.token,
                    rest: success.rest,
                }),

                Err(e) => Err(Failure {
                    context: success.token,
                    expected: expected.into(),
                    source: Some(Box::new(e)),
                    rest: success.rest,
                }),
            },
            Err(err) => Err(err),
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

    fn tokenize_value(self) -> ParseResult<'t, &'t str> {
        self.map(|success| {
            let token = success.token;
            success.map_value(|_| token)
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// Success and Failure
////////////////////////////////////////////////////////////////////////////////
/// A struct representing a successful parse.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
    pub fn map_value<F, U>(self, f: F) -> Success<'t, U>
        where F: FnOnce(V) -> U
    {
        Success {
            value: (f)(self.value),
            token: self.token,
            rest: self.rest,
        }
    }

    /// Splits the parsed value from the Success.
    pub fn take_value(self) -> (V, Success<'t, ()>) {
        (self.value, Success {
            value: (),
            token: self.token,
            rest: self.rest,
        })
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
    pub fn join<U>(self, other: Success<'t, U>, text: &'t str)
        -> Success<'t, ()>
    {
        Success {
            value: (),
            token: &text[..self.token.len() + other.token.len()],
            rest: other.rest,
        }
    }

    /// Joins two sequential successful parse results together, combining values
    /// with the given function.
    pub fn join_with<F, U, T>(self, other: Success<'t, U>, text: &'t str, f: F)
        -> Success<'t, T>
        where F: FnOnce(V, U) -> T
    {
        Success {
            value: (f)(self.value, other.value),
            token: &text[..self.token.len() + other.token.len()],
            rest: other.rest,
        }
    }

    /// Joins a failure result to a previously successful result, expanding the
    /// context of the failure.
    pub fn join_failure(&self, mut other: Failure<'t>, text: &'t str)
        -> Failure<'t>
    {
        other.context = &text[..self.token.len() + other.context.len()];
        other.rest = text;
        other
    }
}


////////////////////////////////////////////////////////////////////////////////
// Failure
////////////////////////////////////////////////////////////////////////////////
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
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    /// The remainder of the parsable text. Failed parses should return their
    /// exact input text.
    pub rest: &'t str,
}

impl<'t> Failure<'t> {
    /// Advances the parse text remainder past the failing token.
    pub fn rest_continuing(&self) -> &'t str {
        &self.rest[self.context.len()..]
    }

    /// Converts a borrowed `Failure` into a `FailureOwned`.
    pub fn to_owned(self) -> FailureOwned {
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
            write!(f, ", found \"{}\"", self.context)?;
        }
        Ok(())
    }
}

impl<'t> std::error::Error for Failure<'t> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}

#[cfg(test)]
impl<'t> PartialEq for Failure<'t> {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context &&
        self.rest == other.rest
    }
}


////////////////////////////////////////////////////////////////////////////////
// FailureOwned
////////////////////////////////////////////////////////////////////////////////
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
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl std::fmt::Display for FailureOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: expected {}", self.expected)?;
        if !self.context.is_empty() {
            write!(f, ", found \"{}\"", self.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for FailureOwned {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|src| {
            // Cast away Send + Sync bounds.
            let src: &(dyn std::error::Error + 'static) = src.as_ref();
            src
        })
    }
}
