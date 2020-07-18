////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Script parsers.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::command::CommandOption;
use crate::parse::any_literal_map;
use crate::parse::bracket;
use crate::parse::char;
use crate::parse::char_matching;
use crate::parse::circumfix;
use crate::parse::intersperse_collect;
use crate::parse::escaped_string;
use crate::parse::Failure;
use crate::parse::literal;
use crate::parse::literal_once;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::QuoteType;
use crate::parse::repeat;
use crate::parse::Success;
use crate::parse::tokenize;
use crate::parse::whitespace;
use crate::error::ScriptError;

// External library imports.
use structopt::StructOpt as _;

// Standard library imports.
use std::borrow::Cow;





/// Parses a series of statements.
pub fn statements<'t>(text: &'t str) -> ParseResult<'t, Vec<CommandOption>> {
    intersperse_collect(1, None,
            statement,
            char(';'))
        (text)
}

/// Parses a statment.
pub fn statement<'t>(text: &'t str) -> ParseResult<'t, CommandOption> {
    command_option(text)
        .convert_value(|opt| {
            match &opt {
                CommandOption::New { .. } |
                CommandOption::List { .. } |
                CommandOption::Undo { .. } |
                CommandOption::Redo { .. } |
                CommandOption::Export { .. } |
                CommandOption::Import { .. } => return Err(ScriptError {
                    msg: "unsupported operation".into() 
                }),
                _ => (),
            }
            Ok(opt)
        })
}


/// Parses a CommandOption.
pub fn command_option<'t>(text: &'t str) -> ParseResult<'t, CommandOption> {
    let mut chunks = Vec::new();
    let mut suc = Success { value: (), token: "", rest: text };

    loop {
        match circumfix(
                chunk,
                maybe(whitespace))
            (suc.rest)
            .with_join_previous(suc, text)
        {
            Ok(chunk_suc) => {
                let (val, next_suc) = chunk_suc.take_value();
                chunks.push(val.to_string());
                suc = next_suc;
            },
            Err(_) => {
                return Ok(suc).convert_value(|_| {
                    CommandOption::from_iter_safe(chunks)
                });
            }
        }
    }
}

/// Parses a potentially quoted chunk of text.
pub fn chunk<'t>(text: &'t str) -> ParseResult<'t, Cow<'t, str>> {
    let escaped = escaped_string(
            script_string_open,
            script_string_close,
            script_string_escape)
        (text);
    if escaped.is_ok() {
        return escaped;
    }

    repeat(1, None,
            char_matching(|c| c != ';' && !c.is_whitespace()))
        (text)
        .tokenize_value()
        .map_value(Cow::from)
}



/// Parses a script string opening quote. For use with escaped_string.
pub fn script_string_open<'t>(text: &'t str)
    -> ParseResult<'t, (Cow<'static, str>, QuoteType)>
{
    let raw_hashed = bracket(
            tokenize(repeat(1, None, char('#'))),
            char('r'),
            char('"'))
        (text);
    if raw_hashed.is_ok() {
        return raw_hashed
            .map_value(|close| {
                let mut s = "\"".to_string();
                s.push_str(close);
                (s.into(), QuoteType::Raw)
            });
    }

    any_literal_map(
            literal,
            "string open quote",
            vec![
                ("r\"", ("\"".into(), QuoteType::Raw)),
                ("\"",  ("\"".into(), QuoteType::Double)),
                ("'",   ("'".into(),  QuoteType::Single)),
            ])
        (text)
}

/// Parses a script string closing quote. For use with escaped_string.
pub fn script_string_close<'t, 'o: 't>(text: &'t str, open: Cow<'o, str>)
    -> ParseResult<'t, &'t str>
{
    literal_once(open.as_ref())(text)
}

/// Parses a script string escape character. For use with escaped_string.
pub fn script_string_escape<'t>(text: &'t str, quote_type: QuoteType)
    -> ParseResult<'t, &'static str> 
{
    match quote_type {
        QuoteType::Double => any_literal_map(
                literal,
                "string escape",
                vec![
                    ("\\n",  "\n"),
                    ("\\t",  "\t"),
                    ("\"",   "\""),
                    ("\\\\", "\\"),
                ])
            (text),
        QuoteType::Single => any_literal_map(
                literal,
                "string escape",
                vec![
                    ("\\n",  "\n"),
                    ("\\t",  "\t"),
                    ("\\'",  "'"),
                    ("\\\\", "\\"),
                ])
            (text),
        QuoteType::Raw => Err(Failure {
            token: "",
            rest: text,
            expected: "".to_owned().into(),
            source: None,
        }),
    }
}
