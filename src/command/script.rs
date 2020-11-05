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
use crate::command::CommonOptions;
use crate::error::FileError;
use crate::palette::Palette;
use crate::palette::InsertExpr;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::stmts;
use crate::setup::Config;
use crate::setup::Settings;

// External library imports.
use tephra::combinator::left;
use tephra::combinator::end_of_text;
use tephra::lexer::Lexer;
use tephra::position::Lf;
use tephra::result::FailureOwned;
use tephra::result::ParseResultExt as _;

// Standard library imports.
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::borrow::Cow;
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
        if self.stmts.is_empty() {
            log::warn!("Executing empty script.");
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
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        left(stmts, end_of_text)(lexer)
            .map_value(|stmts| Script { stmts })
            .finish()
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
        use anyhow::Context as _;
        
        log::debug!("Executing statement {:?}", self);
        match self {
            Stmt::PaletteHeader { name }              => (),
            Stmt::PageHeader { name, number }         => (),
            Stmt::LineHeader { name, number }         => (),
            Stmt::Expr { expr } => {
                // TODO: implement expr naming.
                let name: Option<Cow<'static, str>> = None;
                let positioning = unimplemented!();
                let cursor_behavior = unimplemented!();

                palette
                    .insert_exprs(&[expr], name, positioning, cursor_behavior)
                    .context("expr insert failed.")?;
            },
        }

        Ok(())
    }
}
