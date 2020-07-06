////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! The application configuration file.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// Local imports.
use anyhow::Error;
use anyhow::Context;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

use log::*;

// Standard library imports.
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// DEFAULT_SETTINGS_PATH
////////////////////////////////////////////////////////////////////////////////
/// The default path to look for the [`Settings`] file, relative to the
/// application root.
///
/// [`Settings`]: struct.Settings.html
pub const DEFAULT_SETTINGS_PATH: &'static str = ".atma-settings";

////////////////////////////////////////////////////////////////////////////////
// Settings
////////////////////////////////////////////////////////////////////////////////
/// Application settings. Configures the application behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// The name of the palette to open when no palette is specified.
    #[serde(default)]
    pub active_palette: Option<PathBuf>,
}


impl Settings {
    /// Constructs a new `Settings` with the default options.
    pub fn new() -> Self {
        Settings::default()
    }

    /// Constructs a new `Settings` with options read from the given file path.
    pub fn from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let file = File::open(path)
            .with_context(|| "Failed to open settings file.")?;
        Settings::from_file(file)
    }

    /// Constructs a new `Settings` with options parsed from the given file.
    fn from_file(mut file: File) -> Result<Self, Error>  {
        Settings::parse_ron_file(&mut file)
    }

    /// Parses a `Settings` from a file using the RON format.
    fn parse_ron_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .with_context(|| "Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .with_context(|| "Failed to read settings file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .with_context(|| "Failed deserializing RON file")?;
        let settings = Settings::deserialize(&mut d)
            .with_context(|| "Failed parsing Ron file")?;
        d.end()
            .with_context(|| "Failed parsing Ron file")?;

        Ok(settings) 
    }
    
    /// Normalizes paths in the settings by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, base: &PathBuf) {
        match self.active_palette {
            Some(ref active_palette) if active_palette.is_relative() => {
                let active_palette = base.clone().join(active_palette);
                self.active_palette = Some(active_palette);
            },
            _ => (),
        }
    }

    /// Returns the default active palette.
    #[inline(always)]
    fn default_active_palette() -> Option<PathBuf> {
        None
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            active_palette: Settings::default_active_palette(),
        }
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\tactive_palette: {:?}",
            self.active_palette)
    }
}
