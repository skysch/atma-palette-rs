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
use crate::command::CommandOption;
use crate::palette::Palette;
use crate::Config;
use crate::Settings;
use crate::error::FileError;
use crate::parse::script;
use crate::parse::FailureOwned;
use crate::parse::ParseResultExt as _;
use crate::error::FileErrorContext as _;

// External library imports.
use log::*;

// Standard library imports.
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
    /// The script's executable statements.
    pub statements: Vec<CommandOption>,
}

impl Script {
    /// Executes the script on the given palette.
    pub fn execute(
        self,
        _palette: &mut Palette,
        _config: &Config,
        _settings: &mut Settings)
        -> Result<(), anyhow::Error>
    {
        Ok(())
    }

    /// Constructs a new `Script` by parsing data from the file at the given
    /// path.
    pub fn read_from_path<P>(path: P) -> Result<Self, FileError>
        where P: AsRef<Path> + Debug
    {
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
        script(text)
            .end_of_text()
            .finish()
    }
}
