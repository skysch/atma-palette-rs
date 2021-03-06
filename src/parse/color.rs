////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Color parsing.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// Local imports.
use crate::color::Color;
use crate::color::Rgb;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::float;
use crate::parse::uint;

// External library imports.
use tephra::combinator::any;
use tephra::combinator::both;
use tephra::combinator::bracket;
use tephra::combinator::bracket_dynamic;
use tephra::combinator::exact;
use tephra::combinator::intersperse_collect;
use tephra::combinator::one;
use tephra::combinator::right;
use tephra::combinator::section;
use tephra::combinator::seq;
use tephra::combinator::spanned;
use tephra::combinator::text;
use tephra::lexer::Lexer;
use tephra::lexer::Scanner;
use tephra::position::ColumnMetrics;
use tephra::result::Failure;
use tephra::result::ParseError;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;
use tephra::result::Spanned;
use tephra::result::Success;
use tephra::span::Span;
use tracing::event;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::str::FromStr as _;
use std::convert::TryFrom as _;



////////////////////////////////////////////////////////////////////////////////
// color
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which parses a `Color`.
pub fn color<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Color>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "color");
    let _enter = span.enter();

    match rgb_hex_code
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(Color::from),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    color_function(lexer).map_value(Color::from)
}


////////////////////////////////////////////////////////////////////////////////
// rgb_hex
////////////////////////////////////////////////////////////////////////////////

/// Returns a parser which parses a hex code with the given number of digits.
pub fn rgb_hex_code<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Rgb>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "rgb_hex_code");
    let _enter = span.enter();

    let (mut val, succ) = text(exact(
            seq(&[AtmaToken::Hash, AtmaToken::HexDigits])))
        (lexer)?
        .take_value();

    if val.len() == 4 || val.len() == 7 {
        let rgb = Rgb::from_hex_code(val).unwrap();
        Ok(Success {
            lexer: succ.lexer,
            value: rgb,
        })
    } else {
        Err(Failure {
            parse_error: ParseError::new("invalid color code")
                .with_span(
                    format!("3 or 6 digits required, {} provided",
                        val.len() - 1),
                    succ.lexer.token_span(),
                    succ.lexer.column_metrics()),
            lexer: succ.lexer,
            source: None,
        })
    }
}



////////////////////////////////////////////////////////////////////////////////
// color_function
////////////////////////////////////////////////////////////////////////////////

pub fn color_function<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Color>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "color_function");
    let _enter = span.enter();

    let (val, succ) = fn_call
            (lexer.sublexer())?
        .take_value();

    if val.name.eq_ignore_ascii_case("rgb") {
        return rgb_from_args(lexer.join(succ.lexer), val.args)
            .map_value(Color::from);
    }

    Err(Failure {
        parse_error: ParseError::new("invalid color")
            .with_span(
                "not a recognized color form",
                succ.lexer.parse_span(),
                succ.lexer.column_metrics()),
        lexer: lexer.join(succ.lexer),
        source: None,
    })
}


fn rgb_from_args<'text, Cm>(
    lexer: Lexer<'text, AtmaScanner, Cm>,
    mut args: Vec<Spanned<'text, FnArg>>)
    -> ParseResult<'text, AtmaScanner, Cm, Rgb>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "rgb_from_args");
    let _enter = span.enter();

    if args.len() != 3 {
        return Err(Failure {
            parse_error: ParseError::new("invalid RGB color")
                .with_span(
                    format!("RGB color requires 3 arguments, {} provided",
                        args.len()),
                    lexer.token_span(),
                    lexer.column_metrics()),
            lexer,
            source: None,
        });
    }

    use FnArg::*;
    let arg = args.pop().expect("pop fn arg from capacity > 2");
    let b_span = (arg.value, arg.span);
    let arg = args.pop().expect("pop fn arg from capacity > 2");
    let g_span = (arg.value, arg.span);
    let arg = args.pop().expect("pop fn arg from capacity > 2");
    let r_span = (arg.value, arg.span);

    match (r_span, g_span, b_span) {
        ((F32(r), rs), (F32(g), gs), (F32(b), bs)) => {
            if r < 0.0 || r > 1.0 {
                Err(Failure {
                    parse_error: ParseError::new("invalid RGB color")
                        .with_span(
                            "red value out of allowed range [0.0, 1.0]",
                            rs,
                            lexer.column_metrics()),
                    lexer,
                    source: None,
                })
            } else if g < 0.0 || g > 1.0 {
                Err(Failure {
                    parse_error: ParseError::new("invalid RGB color")
                        .with_span(
                            "green value out of allowed range [0.0, 1.0]",
                            gs,
                            lexer.column_metrics()),
                    lexer,
                    source: None,
                })
            } else if b < 0.0 || b > 1.0 {
                Err(Failure {
                    parse_error: ParseError::new("invalid RGB color")
                        .with_span(
                            "blue value out of allowed range [0.0, 1.0]",
                            bs,
                            lexer.column_metrics()),
                    lexer,
                    source: None,
                })

            } else {
                Ok(Success {
                    value: Rgb::from([r, g, b]),
                    lexer,
                })
            }
        }
        
        ((U32(r), rs), (U32(g), gs), (U32(b), bs)) => match (
            u8::try_from(r),
            u8::try_from(g),
            u8::try_from(b)) 
        {
            (Ok(r), Ok(g), Ok(b)) => Ok(Success {
                value: Rgb::from([r, g, b]),
                lexer,
            }),

            (Ok(r), Ok(g), Err(e)) => Err(Failure {
                parse_error: ParseError::new("invalid RGB color")
                    .with_span(
                        "blue octet out of range [0-255]",
                        bs,
                        lexer.column_metrics()),
                lexer,
                source: Some(Box::new(e)),
            }),
            (Ok(r), Err(e),     _) => Err(Failure {
                parse_error: ParseError::new("invalid RGB color")
                    .with_span(
                        "green octet out of range [0-255]",
                        gs,
                        lexer.column_metrics()),
                lexer,
                source: Some(Box::new(e)),
            }),
            (Err(e),     _,     _) => Err(Failure {
                parse_error: ParseError::new("invalid RGB color")
                    .with_span(
                        "red octet out of range [0-255]",
                        rs,
                        lexer.column_metrics()),
                lexer,
                source: Some(Box::new(e)),
            }),
        }

        ((U32(_), _), (U32(_), _), (F32(_), s)) |
        ((U32(_), _), (F32(_), s), _          ) => Err(Failure {
            parse_error: ParseError::new("invalid RGB color")
                .with_span(
                    "expected u8 value here",
                    s,
                    lexer.column_metrics()),
            lexer,
            source: None,
        }),

        ((F32(_), _), (F32(_), _), (U32(_), s)) |
        ((F32(_), _), (U32(_), s), _          ) => Err(Failure {
            parse_error: ParseError::new("invalid RGB color")
                .with_span(
                    "expected f32 value here",
                    s,
                    lexer.column_metrics()),
            lexer,
            source: None,
        }),
    }
}




////////////////////////////////////////////////////////////////////////////////
// FnCall
////////////////////////////////////////////////////////////////////////////////
/// An AST matcher for parsing a function call.
#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<'text> {
    /// The name of the function.
    pub name: &'text str,
    /// The function arguments.
    pub args: Vec<Spanned<'text, FnArg>>,
}

////////////////////////////////////////////////////////////////////////////////
// FnArg
////////////////////////////////////////////////////////////////////////////////
/// And AST matcher for parsing a function call argument.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnArg {
    /// A u32 argument.
    U32(u32),
    /// An f32 argument.
    F32(f32),
}


////////////////////////////////////////////////////////////////////////////////
// Parsers
////////////////////////////////////////////////////////////////////////////////
/// Parses a simple function call.
pub fn fn_call<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, FnCall<'text>>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "fn_call");
    let _enter = span.enter();

    both(
        text(one(AtmaToken::Ident)),
        bracket(
            one(AtmaToken::OpenParen),
            intersperse_collect(0, None,
                section(spanned(fn_arg)),
                one(AtmaToken::Comma)),
            one(AtmaToken::CloseParen)))
        (lexer)
        .map_value(|(name, args)| FnCall { name, args })
}

/// Parses a simple function call arg.
pub fn fn_arg<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, FnArg>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "fn_arg");
    let _enter = span.enter();

    match float::<_, f32>
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(FnArg::F32),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }
    
    uint::<_, u32>
        (lexer)
        .map_value(FnArg::U32)
}
