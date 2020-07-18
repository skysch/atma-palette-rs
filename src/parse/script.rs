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
use crate::command::Script;
use crate::parse::any_literal_map;
use crate::parse::bracket;
use crate::parse::char;
use crate::parse::char_matching;
use crate::parse::circumfix;
use crate::parse::prefix;
use crate::parse::postfix;
use crate::parse::repeat_collect;
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
use log::*;

// Standard library imports.
use std::borrow::Cow;


/// Parses a script.
pub fn script<'t>(text: &'t str) -> ParseResult<'t, Script> {
    postfix(
        statements,
        maybe(whitespace))
        (text)
        .map_value(|statements| Script {
            statements,
        })
}

/// Parses a series of statements.
pub fn statements<'t>(text: &'t str) -> ParseResult<'t, Vec<CommandOption>> {
    repeat_collect(1, None,
            statement)
        (text)
        .source_for("statements")
        .map_value(|v| v.into_iter().filter_map(|c| c).collect())
}

/// Parses a statment.
pub fn statement<'t>(text: &'t str) -> ParseResult<'t, Option<CommandOption>> {
    trace!("Parsing statement from {:?}", text);
    let empty_stmt = circumfix(
            char(';'),
            maybe(whitespace))
        (text);
    if empty_stmt.is_ok() {
        return empty_stmt.map_value(|_| None);
    }

    command_option(text)
        .convert_value(|opt| {
            if opt.disallowed_in_scripts() {
                return Err(ScriptError {
                    msg: "unsupported operation".into() 
                })
            }
            Ok(Some(opt))
        })
        .source_for("statement")
}

/// Parses a CommandOption.
pub fn command_option<'t>(text: &'t str) -> ParseResult<'t, CommandOption> {
    trace!("Parsing command_option from {:?}", text);
    let mut chunks = vec!["script".into()];
    let mut suc = Success { value: (), token: "", rest: text };

    loop {
        match prefix(
                chunk,
                maybe(whitespace))
            (suc.rest)
            .with_join_previous(suc, text)
        {
            Ok(chunk_suc) if chunk_suc.token.trim().is_empty() => {
                trace!("    Invalid chunk (empty.)");
                return Ok(suc).convert_value(|_| {
                    CommandOption::from_iter_safe(chunks)
                });
            },
            Err(_) => {
                trace!("    Invalid chunk (failed.)");
                return Ok(suc).convert_value(|_| {
                    CommandOption::from_iter_safe(chunks)
                });
            }
            Ok(chunk_suc) => {
                let (val, next_suc) = chunk_suc.take_value();
                chunks.push(val.to_string());
                suc = next_suc;
            },
        }
    }
}

/// Parses a potentially quoted chunk of text.
pub fn chunk<'t>(text: &'t str) -> ParseResult<'t, Cow<'t, str>> {
    trace!("Parsing chunk from {:?}", text);
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
        .source_for("chunk")
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
