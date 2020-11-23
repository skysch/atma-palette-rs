////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Scanner definition for Atma commands.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// External library imports.
use tephra::combinator::any;
use tephra::combinator::both;
use tephra::combinator::bracket;
use tephra::combinator::bracket_dynamic;
use tephra::combinator::exact;
use tephra::combinator::one;
use tephra::combinator::right;
use tephra::combinator::text;
use tephra::lexer::Lexer;
use tephra::lexer::Scanner;
use tephra::position::ColumnMetrics;
use tephra::position::Pos;
use tephra::result::Failure;
use tephra::result::ParseError;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;
use tracing::event;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::convert::TryInto as _;
use std::borrow::Cow;


macro_rules! return_if_some {
    ($p:expr) => {
        if let Some(parse) = $p {
            return Some(parse);
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// AtmaToken
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AtmaToken {
    /// Any number of whitespace characters.
    Whitespace,

    /// An open line comment sequence '//'.
    OpenLineComment,
    /// An open block comment sequence '/*'.
    OpenBlockComment,
    /// A close block comment sequence '*/'.
    CloseBlockComment,
    /// Comment source.
    CommentText,
    
    /// An open parenthesis character '('.
    OpenParen,
    /// A close parenthesis character ')'.
    CloseParen,

    /// An open square bracket character '['.
    OpenBracket,
    /// A close square bracket character ']'.
    CloseBracket,

    /// An open curly bracket character '{'.
    OpenBrace,
    /// A close curly bracket character '}'.
    CloseBrace,

    /// A raw string open "r[#*]\"".
    RawStringOpen,
    /// A raw string close "\"[#*]", which must match the corresponding open
    /// source.
    RawStringClose,
    /// A raw string, ignoring escape characters.
    RawStringText,
    
    /// A single-quote string open character '''.
    StringOpenSingle,
    /// A single-quote string close character '''.
    StringCloseSingle,
    /// A double-quote string open character '"'.
    StringOpenDouble,
    /// A double-quote string close character '"'.
    StringCloseDouble,
    /// A string with potential escape characters.
    StringText,

    /// A semicolon character ';'.
    Semicolon,
    /// A colon character ':'.
    Colon,
    /// A comma character ','.
    Comma,
    /// An octothorpe character '#'.
    Hash,
    /// An asterisk character '*'.
    Mult,
    /// A plus character '+'.
    Plus,
    /// A minus or hyphen character '-'.
    Minus,

    /// A floating point number.
    Float,
    /// A decimal point character '.'.
    Decimal,
    
    /// Any number of uint digits or underscore characters.
    Uint,
    /// Any number of hex digits. Can only be parsed imediately following a
    /// Hash token.
    HexDigits,
    /// An identifier with the form "[_[alpha]][alphanumeric]+".
    Ident,

    /// An underscore character '_'.
    Underscore,
}

impl AtmaToken {
    pub fn is_whitespace_or_comment(&self) -> bool {
        use AtmaToken::*;
        match self {
            Whitespace        |
            OpenLineComment   |
            OpenBlockComment  |
            CloseBlockComment |
            CommentText       => true,
            _                 => false,
        }
    }
}

impl std::fmt::Display for AtmaToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AtmaToken::*;
        match self {
            Whitespace        => write!(f, "whitespace"),
            OpenLineComment   => write!(f, "'//'"),
            OpenBlockComment  => write!(f, "'/*'"),
            CloseBlockComment => write!(f, "'*/'"),
            CommentText       => write!(f, "comment source"),
            OpenParen         => write!(f, "'('"),
            CloseParen        => write!(f, "')'"),
            OpenBracket       => write!(f, "'['"),
            CloseBracket      => write!(f, "']'"),
            OpenBrace         => write!(f, "'{{'"),
            CloseBrace        => write!(f, "'}}'"),
            RawStringOpen     => write!(f, "raw string quote"),
            RawStringClose    => write!(f, "raw string quote"),
            RawStringText     => write!(f, "raw string source"),
            StringOpenSingle  => write!(f, "open quote '''"),
            StringCloseSingle => write!(f, "close qoute '''"),
            StringOpenDouble  => write!(f, "open quote '\"'"),
            StringCloseDouble => write!(f, "close quote '\"'"),
            StringText        => write!(f, "string source"),
            Semicolon         => write!(f, "';'"),
            Colon             => write!(f, "':'"),
            Comma             => write!(f, "','"),
            Hash              => write!(f, "'#'"),
            Mult              => write!(f, "'*'"),
            Plus              => write!(f, "'+'"),
            Minus             => write!(f, "'-'"),
            Float             => write!(f, "float"),
            Decimal           => write!(f, "'.'"),
            Uint              => write!(f, "integer"),
            HexDigits         => write!(f, "hex digits"),
            Ident             => write!(f, "identifier"),
            Underscore        => write!(f, "'_'"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// AtmaScanner
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtmaScanner {
    open: Option<AtmaToken>,
    depth: u64,
}

impl AtmaScanner {
    pub fn new() -> Self {
        AtmaScanner {
            open: None,
            depth: 0,
        }
    }


    /// Parses a string.
    fn parse_str<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm,
        pattern: &str,
        token: AtmaToken)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        metrics.position_after_str(source, base, pattern)
            .map(|end| (token, end))
    }

    /// Parses a Ident token.
    fn parse_ident<Cm>(&mut self, source: &str, base: Pos, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_ident");
        let _enter = span.enter();

        // Parse alpha or underscore.
        let end = metrics
            .next_position_after_chars_matching(source, base,
                |c|  c.is_alphabetic() || c == '_')?;

        // Parse alphanumerics or underscores.
        let end = metrics
            .position_after_chars_matching(source, end,
                |c|  c.is_alphanumeric() || c == '_')
            .unwrap_or(end);

        Some((AtmaToken::Ident, end))
    }

    /// Parses an Uint token.
    fn parse_uint<Cm>(&mut self, source: &str, base: Pos, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_uint");
        let _enter = span.enter();

        let mut end = base;

        // Unprefixed uints can't start with '_'.
        if source[end.byte..].starts_with('_') { return None; }

        // Identify radix.
        let radix = if source[end.byte..].starts_with("0b") {
            end += Pos::new(2, 0, 2);
            2
        } else if source[end.byte..].starts_with("0o") {
            end += Pos::new(2, 0, 2);
            8
        } else if source[end.byte..].starts_with("0x") {
            end += Pos::new(2, 0, 2);
            16
        } else {
            10
        };

        // Parse digits or underscore.
        metrics
            .position_after_chars_matching(source, end,
                |c| c.is_digit(radix) || c == '_')
            .map(|end| (AtmaToken::Uint, end))
    }

    /// Parses an HexDigits token.
    fn parse_hex_digits<Cm>(&mut self, source: &str, base: Pos, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_hex_digits");
        let _enter = span.enter();

        // Parse digits or underscore.
        metrics
            .position_after_chars_matching(source, base,
                |c| c.is_digit(16))
            .map(|end| (AtmaToken::HexDigits, end))
    }

    /// Parses a Float token.
    fn parse_float<Cm>(&mut self, source: &str, base: Pos, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_float");
        let _enter = span.enter();

        // Check for special float forms.
        return_if_some!(metrics.position_after_str(source, base, "inf")
            .map(|end| (AtmaToken::Float, end)));

        return_if_some!(metrics.position_after_str(source, base, "nan")
            .map(|end| (AtmaToken::Float, end)));

        // Parse digits, fail if not found.
        let end = metrics
            .position_after_chars_matching(source, base,
                |c| c.is_digit(10))?;

        // Parse decimal point, fail if not found.
        let end = metrics
            .position_after_str(source, end, ".")?;

        // Parse digits.
        let end = metrics
            .position_after_chars_matching(source, end,
                |c| c.is_digit(10))
            .unwrap_or(end);

        // Parse exponent.
        if let Some(end) = metrics
            .next_position_after_chars_matching(source, end,
                |c| c == 'e' || c == 'E')
        {
            // Parse exponent sign.
            let end = metrics
                .next_position_after_chars_matching(source, end,
                    |c| c == '+' || c == '-')
                .unwrap_or(end);

            // Parse exponent digits, fail if not found.
            let end = metrics
                .position_after_chars_matching(source, end,
                    |c| c.is_digit(10))?;

            Some((AtmaToken::Float, end))
        } else {
            Some((AtmaToken::Float, end))
        }
    }

    /// Parses a RawStringOpen token.
    fn parse_raw_string_open<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_raw_string_open");
        let _enter = span.enter();

        let mut end = metrics.position_after_str(source, base, "r")?;

        while let Some(adv) = metrics
            .position_after_chars_matching(source, end,
                |c| c == '#')
        {
            self.depth += 1;
            end = adv;
        }

        metrics.position_after_str(source, end, "\"")
            .map(|end| (AtmaToken::RawStringOpen, end))
    }

    /// Parses a RawStringClose token.
    fn parse_raw_string_close<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_raw_string_close");
        let _enter = span.enter();

        let mut end = metrics.position_after_str(source, base, "\"")?;

        while let Some(adv) = metrics
            .position_after_chars_matching(source, end,
                |c| c == '#')
        {
            if self.depth == 0 { return None; }
            self.depth -= 1;
            end = adv;
        }

        Some((AtmaToken::RawStringOpen, end))
    }

    /// Parses a RawStringText token.
    fn parse_raw_string_text<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_raw_string_text");
        let _enter = span.enter();

        let mut end = base;
        let mut col_iter = metrics.iter_columns(source, end);

        while let Some((_, adv)) = col_iter.next() {
            end = adv;
            if self
                .parse_raw_string_close(source, adv, metrics)
                .is_some()
            {
                self.open = Some(AtmaToken::RawStringText);
                break;
            }
        }

        Some((AtmaToken::RawStringText, end))
    }

    /// Parses a StringText token.
    fn parse_string_text<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm,
        open: AtmaToken)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_string_text");
        let _enter = span.enter();

        let mut end = base;
        let mut col_iter = metrics.iter_columns(source, end);

        while let Some((next, adv)) = col_iter.next() {
            event!(Level::TRACE, "next: {:?}, adv: {}", next, adv);
            match (next, open) {
                ("\\", _) => match col_iter.next() {
                    Some(("\\", adv2)) |
                    Some(("\"", adv2)) |
                    Some(("'",  adv2)) |
                    Some(("t",  adv2)) |
                    Some(("r",  adv2)) |
                    Some(("n",  adv2)) => end = adv2,
                    Some(("u",  adv2)) => unimplemented!("unicode escapes unsupported"),
                    _                  => return None,
                },
                
                ("'",  AtmaToken::StringOpenSingle) |
                ("\"", AtmaToken::StringOpenDouble) => {
                    return Some((AtmaToken::StringText, end));
                },

                _ => end = adv,
            }
        }

        Some((AtmaToken::StringText, end))
    }

    /// Parses a Whitespace token.
    fn parse_whitespace<Cm>(&mut self, source: &str, base: Pos, metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_whitespace");
        let _enter = span.enter();

        metrics
            .position_after_chars_matching(source, base, char::is_whitespace)
            .map(|end| (AtmaToken::Whitespace, end))
    }

    fn parse_line_comment_text<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_line_comment_text");
        let _enter = span.enter();

        let mut end = base;
        let mut col_iter = metrics.iter_columns(source, end);

        while let Some((_, adv)) = col_iter.next() {
            if adv.is_line_start() { break; }
            end = adv;
        }

        Some((AtmaToken::CommentText, end))
    }

    fn parse_block_comment_text<Cm>(
        &mut self,
        source: &str,
        base: Pos,
        metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_block_comment_text");
        let _enter = span.enter();

        let mut end = base;
        let mut col_iter = metrics.iter_columns(source, end);

        while let Some((next, adv)) = col_iter.next() {
            event!(Level::TRACE, "next: {:?}", next);
            event!(Level::TRACE, "adv: {:?}", adv);
            match next {
                "/" => match col_iter.next() {
                    Some(("*", adv2)) => {
                        self.depth += 1;
                        end = adv2;
                    },
                    Some((_, adv2)) => end = adv2,
                    _               => end = adv,
                },
                "*" => match col_iter.next() {
                    Some(("/", adv2)) => {
                        if self.depth == 1 { break; }
                        self.depth -= 1;
                    },
                    Some((_, adv2)) => end = adv2,
                    _               => end = adv,
                },
                
                _ => end = adv,
            }
        }

        Some((AtmaToken::CommentText, end))
    }

    fn parse_token<'text, Cm>(
        &mut self, 
        source: &'text str,
        base: Pos,
        metrics: Cm)
        -> Option<(AtmaToken, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::TRACE, "AtmaScanner::parse_token");
        let _enter = span.enter();

        event!(Level::TRACE, "state: {:?}", self);

        use AtmaToken::*;
        match self.open.take() {
            Some(OpenLineComment) => {
                // Line comment cannot fail, so no other parse could be returned
                // here.
                self.parse_line_comment_text(source, base, metrics)
            },
            Some(OpenBlockComment) => {
                if let Some(parse) = self
                    .parse_str(source, base, metrics, "*/", CloseBlockComment)
                {
                    self.depth = 0;
                    return Some(parse);
                }
                
                self.open = Some(OpenBlockComment);
                // Block comment cannot fail, so no other parse could be
                // returned here.
                self.parse_block_comment_text(source, base, metrics)
            },


            Some(RawStringText) => {
                // Because it is necessary to recognize the RawStringClose to
                // finish parsing RawStringText, we should never get here unless
                // we know the next part of the source is the appropriately sized
                // RawStringClose token. So instead of explicitely parsing it,
                // we can just jump forward.
                let byte: usize = (self.depth + 1)
                    .try_into()
                    .expect("Pos overflow");
                Some((RawStringClose, Pos::new(byte, 0, byte)))
            },
            Some(RawStringOpen) => {
                if let Some(parse) = self
                    .parse_raw_string_close(source, base, metrics)
                {
                    self.depth = 0;
                    return Some(parse);
                }
                return_if_some!(self
                    .parse_raw_string_text(source, base, metrics));
                None
            },

            Some(StringOpenSingle) => {
                return_if_some!(self
                    .parse_str(source, base, metrics, "\'", StringCloseSingle));
                if let Some(parse) = self
                    .parse_string_text(source, base, metrics, StringOpenSingle)
                {
                    // Keep this open to prioritize the close.
                    self.open = Some(StringOpenSingle);
                    return Some(parse);
                }
                panic!("invalid scanner state");
            },
            Some(StringOpenDouble) => {
                return_if_some!(self
                    .parse_str(source, base, metrics, "\"", StringCloseDouble));
                if let Some(parse) = self
                    .parse_string_text(source, base, metrics, StringOpenDouble)
                {
                    // Keep this open to prioritize the close.
                    self.open = Some(StringOpenDouble);
                    return Some(parse);
                }
                panic!("invalid scanner state");
            },

            Some(Hash) => {
                // HexDigits can only come after Hash.
                return_if_some!(self.parse_hex_digits(source, base, metrics));
                self.scan(source, base, metrics)
            },

            Some(Colon) => {
                // Colon will make Position parts a priority until something
                // else is seen. It is important to have uint before float to
                // avoid swallowing them up with decimals.
                self.open = Some(Colon);
                return_if_some!(self.parse_uint(source, base, metrics));

                return_if_some!(self
                    .parse_str(source, base, metrics, ".", Decimal));

                return_if_some!(self
                    .parse_str(source, base, metrics, "*", Mult));

                return_if_some!(self
                    .parse_str(source, base, metrics, "+", Plus));

                return_if_some!(self
                    .parse_str(source, base, metrics, "-", Minus));


                self.open = None;
                self.scan(source, base, metrics)
            },

            None => {
                return_if_some!(self.parse_whitespace(source, base, metrics));

                return_if_some!(self
                    .parse_str(source, base, metrics, "(", OpenParen));

                return_if_some!(self
                    .parse_str(source, base, metrics, ")", CloseParen));

                return_if_some!(self
                    .parse_str(source, base, metrics, "[", OpenBracket));

                return_if_some!(self
                    .parse_str(source, base, metrics, "]", CloseBracket));

                return_if_some!(self
                    .parse_str(source, base, metrics, "{", OpenBrace));

                return_if_some!(self
                    .parse_str(source, base, metrics, "}", CloseBrace));



                if let Some(parse) = self
                    .parse_str(source, base, metrics, "//", OpenLineComment)
                {
                    self.open = Some(OpenLineComment);
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_str(source, base, metrics, "/*", OpenBlockComment)
                {
                    self.open = Some(OpenBlockComment);
                    self.depth = 1;
                    return Some(parse);
                }

                // RawStringOpen must be parsed before Hash.
                if let Some(parse) = self
                    .parse_raw_string_open(source, base, metrics)
                {
                    self.open = Some(RawStringOpen);
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_str(source, base, metrics, "\'", StringOpenSingle)
                {
                    self.open = Some(StringOpenSingle);
                    return Some(parse);
                }
                if let Some(parse) = self
                    .parse_str(source, base, metrics, "\"", StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Some(parse);
                }

                return_if_some!(self
                    .parse_str(source, base, metrics, ";", Semicolon));
                
                if let Some(parse) = self
                    .parse_str(source, base, metrics, ":", Colon)
                {
                    self.open = Some(Colon);
                    return Some(parse);
                }

                return_if_some!(self
                    .parse_str(source, base, metrics, ",", Comma));
                
                if let Some(parse) = self
                    .parse_str(source, base, metrics, "#", Hash)
                {
                    self.open = Some(Hash);
                    return Some(parse);
                }

                return_if_some!(self
                    .parse_str(source, base, metrics, "*", Mult));
                return_if_some!(self
                    .parse_str(source, base, metrics, "+", Plus));
                return_if_some!(self
                    .parse_str(source, base, metrics, "-", Minus));
                
                // Float must be parsed before Uint and Decimal.
                return_if_some!(self.parse_float(source, base, metrics));

                return_if_some!(self
                    .parse_str(source, base, metrics, ".", Decimal));
                return_if_some!(self.parse_uint(source, base, metrics));

                // Ident must be parsed before Underscore.
                return_if_some!(self.parse_ident(source, base, metrics));
                
                return_if_some!(self
                    .parse_str(source, base, metrics, "_", Underscore));
                
                None
            },

            Some(s) => panic!(
                "invalid lexer state Some({:?}) continuing at {:?}", s, source),
        }
    }
}

impl Scanner for AtmaScanner {
    type Token = AtmaToken;

    fn scan<'text, Cm>(&mut self, source: &'text str, base: Pos, metrics: Cm)
        -> Option<(Self::Token, Pos)>
        where Cm: ColumnMetrics,
    {
        let span = span!(Level::DEBUG, "AtmaScanner::scan");
        let _enter = span.enter();
        
        let res = self.parse_token(source, base, metrics);

        event!(Level::DEBUG,
            "scan result: {:?}",
            res.map(|(tok, end)| (tok, &source[base.byte..end.byte])));
        res
    }
}

