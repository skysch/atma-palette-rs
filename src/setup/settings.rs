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
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
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
    /// The path the settings were initially loaded from.
    #[serde(skip)]
    load_path: Option<PathBuf>,

    /// The name of the palette to open when no palette is specified.
    #[serde(default)]
    pub active_palette: Option<PathBuf>,
}


impl Settings {
    /// Constructs a new `Settings` with the default options.
    pub fn new() -> Self {
        Settings {
            load_path: None,
            active_palette: Settings::default_active_palette(),
        }
    }

    /// Returns the given `Settings` with the given load_path.
    pub fn with_load_path<P>(mut self, path: P) -> Self
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
        self
    }

    /// Constructs a new `Settings` with options read from the given file path.
    pub fn read_from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let path = path.as_ref().to_owned();
        let file = File::open(&path)
            .with_context(|| "Failed to open settings file.")?;
        let mut settings = Settings::read_from_file(file)?;
        settings.load_path = Some(path);
        Ok(settings)
    }

    /// Constructs a new `Settings` with options parsed from the given file.
    pub fn read_from_file(mut file: File) -> Result<Self, Error>  {
        Settings::parse_ron_from_file(&mut file)
    }

    /// Parses a `Settings` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
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
    
    /// Open a file at the given path and write the `Settings` into it.
    pub fn write_to_path<P>(&self, path: P)
        -> Result<(), Error>
        where P: AsRef<Path>
    {
        let path = path.as_ref().to_owned();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .with_context(|| "Failed to open config file.")?;
        self.write_to_file(file)
            .with_context(|| "Failed to write config file.")?;
        Ok(())
    }

    /// Write the `Settings` into the file is was loaded from. Returns true if the
    /// data was written.
    pub fn write_to_load_path(&self)
        -> Result<bool, Error>
    {
        match &self.load_path {
            Some(path) => {
                self.write_to_path(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Write the `Settings` into the given file.
    pub fn write_to_file(&self, mut file: File) -> Result<(), Error> {
        self.generate_ron_into_file(&mut file)
    }

    /// Parses a `Settings` from a file using the RON format.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
        let pretty = ron::ser::PrettyConfig::new()
            .with_depth_limit(2)
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true);
        let s = ron::ser::to_string_pretty(&self, pretty)
            .with_context(|| "Failed to serialize RON file")?;
        file.write_all(s.as_bytes())
            .with_context(|| "Failed to write RON file")
    }

    /// Sets the `Settings`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
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
        Settings::new()
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\tactive_palette: {:?}",
            self.active_palette)
    }
}
