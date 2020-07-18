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
use crate::parse::literal_once;
use crate::parse::any_literal_map;
use crate::parse::literal;
use crate::parse::escaped_string;
use crate::parse::char;
use crate::parse::tokenize;
use crate::parse::repeat;
use crate::parse::bracket;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;

// Standard library imports.
use std::borrow::Cow;

/// Parses a statment.
pub fn statement<'t>(text: &'t str) -> ParseResult<'t, CommandOption> {
    command_option(text)
}

/// Parses a command option.
pub fn command_option<'t>(_text: &'t str) -> ParseResult<'t, CommandOption> {
    
    unimplemented!()
}

/// Parses a potentially quoted chunk of text.
pub fn chunk<'t>(text: &'t str) -> ParseResult<'t, Cow<'t, str>> {
    
    escaped_string(
            script_string_open,
            script_string_close,
            script_string_escape)
        (text)
}





/// Parses a script string opening quote. For use with escaped_string.
pub fn script_string_open<'t>(text: &'t str)
    -> ParseResult<'t, (Cow<'static, str>, bool)>
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
                (s.into(), false)
            });
    }

    any_literal_map(
            literal,
            "string open quote",
            vec![
                ("r\"", ("\"".into(), false)),
                ("\"",  ("\"".into(), true)),
                ("'",  ("'".into(), true)),
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
pub fn script_string_escape<'t>(text: &'t str) -> ParseResult<'t, &'static str>
{
    any_literal_map(
            literal,
            "string escape",
            vec![
                ("\\n",  "\n"),
                ("\\t",  "\t"),
                ("\"",   "\""),
                ("\\'",  "'"),
                ("\\\\", "\\"),
            ])
        (text)
}
