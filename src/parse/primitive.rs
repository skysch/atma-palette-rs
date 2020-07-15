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
use crate::parse::atomic;
use crate::parse::Failure;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::repeat;
use crate::parse::circumfix;
use crate::parse::Success;

// Standard library imports.
use std::convert::TryFrom;
use std::convert::TryInto as _;
use std::borrow::Cow;
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////

/// Integer radix prefix for binary numbers.
pub const INT_RADIX_PREFIX_BIN: &'static str = "0b";

/// Integer radix prefix for octal numbers.
pub const INT_RADIX_PREFIX_OCT: &'static str = "0o";

/// Integer radix prefix for hexadecimal numbers.
pub const INT_RADIX_PREFIX_HEX: &'static str = "0x";

/// Floating point sign token.
pub const FLOAT_SIGN: &'static str = "+-";

/// Floating point infinity token.
pub const FLOAT_INF: &'static str = "inf";

/// Floating point NaN token.
pub const FLOAT_NAN: &'static str = "NaN";

/// Floating point exponent prefix token.
pub const FLOAT_EXP: &'static str = "eE";

/// Floating point decimal token.
pub const FLOAT_DECIMAL: char = '.';

/// Floating point decimal string. Used to ensure digits are parsed.
const FLOAT_DECIMAL_STR: &'static str = ".";

////////////////////////////////////////////////////////////////////////////////
// Char parsing.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which parses the specified `char`.
#[inline]
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
                token: "",
                expected: c.to_string().into(),
                source: None,
                rest: text,
            })
        }
    }
}

/// Returns a parser which parses any single `char` in the given string.
#[inline]
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
            token: "",
            expected: format!("one of {}", opts).into(),
            source: None,
            rest: text,
        })
    }
}

/// Returns a parser which parses a `char` if it satisfies the given predicate.
#[inline]
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
            token: "",
            expected: "char satisfying predicate".into(),
            source: None,
            rest: text,
        })
    }
}

/// Parses a whitespace `char`.
#[inline]
pub fn char_whitespace<'t>(text: &'t str) -> ParseResult<'t, char> {
    char_matching(char::is_whitespace)(text)
        .source_for("whitespace char")
}

////////////////////////////////////////////////////////////////////////////////
// String parsing.
////////////////////////////////////////////////////////////////////////////////

/// Parses any nonzero amount of whitespace.
#[inline]
pub fn whitespace<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    repeat(1, None, char_whitespace)(text)
        .tokenize_value()
        .source_for("whitespace")
}

/// Parses the given text literal.
#[inline]
pub fn literal<'t>(expect: &'t str)
    -> impl FnMut(&'t str) -> ParseResult<'t, &'t str>
{
    move |text| {
        if text.starts_with(expect) {
            Ok(Success { 
                value: &text[..expect.len()],
                token: &text[..expect.len()],
                rest: &text[expect.len()..],
            })
        } else {
            Err(Failure { 
                token: "",
                expected: expect.to_owned().into(),
                source: None,
                rest: text,
            })
        }
    }
}

/// Parses the given text literal, ignoring case.
#[inline]
pub fn literal_ignore_ascii_case<'t>(expect: &'t str)
    -> impl FnMut(&'t str) -> ParseResult<'t, &'t str>
{
    move |text| {
        let mut expect_chars = expect.chars();
        let mut text_chars = text.char_indices();
        let mut idx = 0;

        loop {
            match (expect_chars.next(), text_chars.next()) {
                (Some(e), Some((n, t))) if e.eq_ignore_ascii_case(&t) => {
                    idx = n + t.len_utf8();
                },

                (None, _) => return Ok(Success { 
                    value: &text[..idx],
                    token: &text[..idx],
                    rest: &text[idx..],
                }),

                (_, _) => return Err(Failure { 
                    token: &text[..idx],
                    expected: format!("ignore case literal {}", expect).into(),
                    source: None,
                    rest: text,
                }),
            }
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Integer parsing.
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which parses an unsigned integer with optional radix
/// prefix.
pub fn uint<'t, T>(int_type: &'static str)
    -> impl FnMut(&'t str) -> ParseResult<'t, T>
    where T: TryFrom<u64>
{
    move |text| {
        let radix_suc = maybe(prefix_radix_token)(text).unwrap();
        let radix: u32 = match radix_suc.value {
            Some(INT_RADIX_PREFIX_BIN) => 2,
            Some(INT_RADIX_PREFIX_OCT) => 8,
            Some(INT_RADIX_PREFIX_HEX) => 16,
            None => 10,
            // NOTE: This is safe as long as `prefix_radix_token` never succeeds
            // with another string.
            Some(_) => unsafe { std::hint::unreachable_unchecked() },
        };

        uint_value(int_type, radix)(radix_suc.rest)
            .source_for(format!("parse {} value", int_type))
            .with_join_previous(radix_suc, text)
    }
}

/// Parses an integer radix prefix.
pub fn prefix_radix_token<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    if text.starts_with(INT_RADIX_PREFIX_BIN) || 
       text.starts_with(INT_RADIX_PREFIX_OCT) ||
       text.starts_with(INT_RADIX_PREFIX_HEX) 
       // NOTE: Changes to these matches will result in UB in `uint` unless the
       // corresponding case is also handled there.
    {
        Ok(Success { 
            value: &text[..2],
            token: &text[..2],
            rest: &text[2..],
        })
    } else {
        Err(Failure { 
            token: "",
            expected: "0[box]".into(),
            source: None,
            rest: text,
        })
    }
}

/// Returns a parser which parses an unsigned integer with the given radix.
pub fn uint_value<'t, T>(int_type: &'static str, radix: u32)
    -> impl FnMut(&'t str) -> ParseResult<'t, T>
    where T: TryFrom<u64>
{
    uint_digits_value(int_type, 1, None, radix)
}

/// Returns a parser which parses an unsigned integer with the given radix and
/// number of digits.
pub fn uint_digits_value<'t, T>(
    int_type: &'static str,
    low: usize,
    high: Option<usize>,
    radix: u32)
    -> impl FnMut(&'t str) -> ParseResult<'t, T>
    where T: TryFrom<u64>
{
    move |text| {
        let digit = char_matching(|c| c.is_digit(radix) || c == '_');
        let digits_suc = repeat(low, high, digit)(text)
            .source_for(
                format!("{} integer digits with radix {}", int_type, radix))?;

        let mut res: u64 = 0;
        let mut chars = digits_suc.token.chars();
        while let Some(c) = chars.next() {
            if c == '_' { continue; }

            // TODO: Consider parsing all hex digits and emitting an error if
            // any remain. This should make error handling nicer.
            let val = u64::from(c.to_digit(radix).unwrap());
            
            res = res.checked_mul(u64::from(radix)).ok_or_else(|| Failure {
                token: digits_suc.token,
                expected: format!("overflow of {} value", int_type).into(),
                source: Some(Box::new(ParseIntegerOverflow {
                    int_type: int_type.into(),
                    int_text: digits_suc.token.to_string().into(),
                    value: u128::from(res),
                })),
                rest: text,
            })?;

            res = res.checked_add(val).ok_or_else(|| Failure {
                token: digits_suc.token,
                expected: format!("overflow of {} value", int_type).into(),
                source: Some(Box::new(ParseIntegerOverflow {
                    int_type: int_type.into(),
                    int_text: digits_suc.token.to_string().into(),
                    value: u128::from(res) + u128::from(val),
                })),
                rest: text,
            })?;
        }
        
        match res.try_into() {
            Ok(res) => Ok(digits_suc.map_value(|_| res)),
            Err(_) => Err(Failure {
                token: digits_suc.token,
                expected: format!("overflow of {} value", int_type).into(),
                source: Some(Box::new(ParseIntegerOverflow {
                    int_type: int_type.into(),
                    int_text: digits_suc.token.to_string().into(),
                    value: u128::from(res),
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
    pub value: u128,
}


impl std::fmt::Display for ParseIntegerOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "integer value '{}' ({}) does not fit in type {}",
            self.int_text, self.value, self.int_type)
    }
}

impl std::error::Error for ParseIntegerOverflow {}

////////////////////////////////////////////////////////////////////////////////
// Float parsing.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which parses a float value.
pub fn float<'t, T>(float_type: &'static str)
    -> impl FnMut(&'t str) -> ParseResult<'t, T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static
{
    move |text| {
        // Rust float syntax:
        // Float  ::= Sign? ( 'inf' | 'NaN' | Number )
        // Number ::= ( Digit+ |
        //              Digit+ '.' Digit* |
        //              Digit* '.' Digit+ ) Exp?
        // Exp    ::= [eE] Sign? Digit+
        // Sign   ::= [+-]
        // Digit  ::= [0-9]
        
        // Parse the sign.
        let suc = maybe(char_in(FLOAT_SIGN))
            (text)
            .expect("infallible maybe parse");

        // Parse INF literal.
        let float_inf = literal(FLOAT_INF)
            (suc.rest);
        if float_inf.is_ok() {
            return float_inf
                .with_join_previous(suc, text)
                .tokenize_value()
                .convert_value(
                    format!("parse inf {}", float_type),
                    T::from_str);
        }

        // Parse NAN literal.
        let float_nan = literal(FLOAT_NAN)
            (suc.rest);
        if float_nan.is_ok() {
            return float_nan
                .with_join_previous(suc, text)
                .tokenize_value()
                .convert_value(
                    format!("parse inf {}", float_type),
                    T::from_str);
        }

        // Parse the digits.
        let digits = circumfix(
                maybe(char(FLOAT_DECIMAL)),
                repeat(0, None, char_matching(|c| c.is_digit(10))))
            (suc.rest)
            .tokenize_value()
            .expect("infallible repeat parse");

        if digits.token == FLOAT_DECIMAL_STR || digits.token.is_empty() {
            let full = suc.join(digits, text);
            return Err(Failure {
                token: full.token,
                expected: format!("Valid {} value", float_type).into(),
                source: None,
                rest: full.rest,
            })
        }
        let full = suc.join(digits, text);

        // Parse the exponent.
        atomic(float_exp)
            (full.rest)
            .with_join_previous(full, text)
            .tokenize_value()
            .convert_value(
                format!("parse inf {}", float_type),
                T::from_str)
    }
}

/// Parses the floating point exponent.
#[inline]
fn float_exp<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    let suc = char_in(FLOAT_EXP)(text)
        .source_for("float exponent")?;

    let suc = maybe(char_in(FLOAT_SIGN))
        (suc.rest)
        .with_join_previous(suc, text)
        .expect("infallible maybe parse");

    repeat(1, None, char_matching(|c| c.is_digit(10)))
        (suc.rest)
        .source_for("float exponent digits")
        .with_join_previous(suc, text)
        .tokenize_value()
}


