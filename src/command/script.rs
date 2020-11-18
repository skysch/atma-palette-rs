////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line scripting.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::command::CommonOptions;
use crate::command::CursorBehavior;
use crate::command::Positioning;
use crate::error::FileError;
use crate::palette::InsertExpr;
use crate::palette::Palette;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::stmt;
use crate::parse::stmts;
use crate::setup::Config;
use crate::setup::Settings;

// External library imports.
use tephra::combinator::end_of_text;
use tephra::lexer::Lexer;
use tephra::position::Lf;
use tephra::result::FailureOwned;
use tephra::result::ParseResultExt as _;
use tracing::event;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::borrow::Cow;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Script
////////////////////////////////////////////////////////////////////////////////
/// An atma script.
#[derive(Debug)]
pub struct Script {
    /// The script's abstract syntax tree.
    stmts: Vec<Stmt>
}

impl Script {
    /// Executes the script on the given palette.
    pub fn execute(
        self,
        palette: &mut Palette,
        common: &CommonOptions,
        config: &Config,
        settings: &mut Settings)
        -> Result<(), anyhow::Error>
    {
        let span = span!(Level::TRACE, "Script::execute");
        let _enter = span.enter();

        if self.stmts.is_empty() {
            tracing::warn!("Executing empty script.");
        }
        for stmt in self.stmts {
            stmt.execute(palette, common, config, settings)?;
        }
        Ok(())
    }

    /// Constructs a new `Script` by parsing data from the file at the given
    /// path.
    pub fn read_from_path<P>(path: P) -> Result<Self, FileError>
        where P: AsRef<Path> + Debug
    {
        use crate::error::FileErrorContext as _;

        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .with_context(|| format!("Failed to open file {:?}", path))?;
        Script::read_from_file(&mut file)
    }

    /// Constructs a new `Script` by parsing data from the given file.
    pub fn read_from_file(file: &mut File) -> Result<Self, FileError> {
        let mut buf = String::new();

        let _ = file.read_to_string(&mut buf)?;

        <Script as std::str::FromStr>::from_str(&buf[..])
            .map_err(FileError::from)
    }
}

impl std::str::FromStr for Script {
    type Err = FailureOwned<Lf>;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let span = span!(Level::DEBUG, "Script::from_str");
        let _enter = span.enter();

        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        let (script, succ) = stmts
            (lexer)
            .map_value(|stmts| Script { stmts })?
            .take_value();

        // Try to parse end-of-text, and if it fails, return the error from
        // a stmt parse.
        let end = end_of_text
            (succ.lexer);
        if let Err(fail) = end {
            stmt
                (fail.lexer)
                .map_value(|_| script)
                .finish()
        } else {
            end.map_value(|_| script)
                .finish()
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Statements
////////////////////////////////////////////////////////////////////////////////
/// A script statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    PaletteHeader { 
        name: Option<Cow<'static, str>>,
    },
    PageHeader {
        name: Option<Cow<'static, str>>,
        number: Option<u16>,
    },
    LineHeader {
        name: Option<Cow<'static, str>>,
        number: Option<u16>,
    },
    Expr {
        expr: InsertExpr,
    },
}

impl Stmt {
    pub fn execute(
        self,
        palette: &mut Palette,
        common: &CommonOptions,
        config: &Config,
        settings: &mut Settings)
        -> Result<(), anyhow::Error>
    {
        let span = span!(Level::TRACE, "Stmt::execute");
        let _enter = span.enter();
        event!(Level::TRACE, "executing statement:\n{:?}", self);

        use Stmt::*;
        use anyhow::Context as _;
        
        match self {
            PaletteHeader { name }      => {
                if name.is_some() {
                    palette.set_name(name, PositionSelector::ALL)?;
                }
                let _ = palette.set_position_cursor(Position::MIN);
            },

            PageHeader { name, number } => {
                let page = number.unwrap_or_else(|| palette
                    .position_cursor().page);

                let _ = palette
                    .set_position_cursor(Position { page, line: 0, column: 0 });

                if name.is_some() {
                    palette.set_name(name, PositionSelector {
                        page: Some(page),
                        line: None,
                        column: None,
                    })?;
                }
            },
            
            LineHeader { name, number } => {
                let page = palette.position_cursor().page;
                let line = number.unwrap_or_else(|| palette
                    .position_cursor().line);

                let _ = palette
                    .set_position_cursor(Position { page, line, column: 0 });

                if name.is_some() {
                    palette.set_name(name, PositionSelector {
                        page: Some(page),
                        line: Some(line),
                        column: None,
                    })?;
                }
            },
            
            Expr { expr }               => {
                // TODO: implement expr naming.
                let name: Option<Cow<'static, str>> = None;
                let positioning = Positioning::Cursor;
                let cursor_behavior = CursorBehavior::MoveAfterEnd;

                palette
                    .insert_exprs(&[expr], name, positioning, cursor_behavior)
                    .context("expr insert failed.")?;
            },
        }

        Ok(())
    }
}
