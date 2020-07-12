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
use crate::utility::normalize_path;
use crate::error::FileError;
use crate::error::FileErrorContext as _;

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
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
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

    /// Returns the `Settings`' load path.
    pub fn load_path(&self) -> Option<&Path> {
        self.load_path.as_ref().map(AsRef::as_ref)
    }

    /// Sets the `Settings`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
    }

    /// Constructs a new `Settings` with options read from the given file path.
    pub fn read_from_path<P>(path: P) -> Result<Self, FileError> 
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!(
                "Failed to open settings file for reading: {}",
                path.display()))?;
        let mut settings = Settings::read_from_file(file)?;
        settings.load_path = Some(path.to_owned());
        Ok(settings)
    }

    /// Open a file at the given path and write the `Settings` into it.
    pub fn write_to_path<P>(&self, path: P)
        -> Result<(), FileError>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .with_context(|| format!(
                "Failed to create/open settings file for writing: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write settings file")?;
        Ok(())
    }

    /// Create a new file at the given path and write the `Settings` into it.
    pub fn write_to_path_if_new<P>(&self, path: P)
        -> Result<(), FileError>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!(
                "Failed to create settings file: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write settings file")?;
        Ok(())
    }

    /// Write the `Settings` into the file is was loaded from. Returns true if
    /// the data was written.
    pub fn write_to_load_path(&self)
        -> Result<bool, FileError>
    {
        match &self.load_path {
            Some(path) => {
                self.write_to_path(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Write the `Settings` into a new file using the load path. Returns true
    /// if the data was written.
    pub fn write_to_load_path_if_new(&self)
        -> Result<bool, FileError>
    {
        match &self.load_path {
            Some(path) => {
                self.write_to_path_if_new(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Constructs a new `Settings` with options parsed from the given file.
    pub fn read_from_file(mut file: File) -> Result<Self, FileError>  {
        Settings::parse_ron_from_file(&mut file)
    }

    /// Parses a `Settings` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, FileError> {
        let len = file.metadata()
            .context("Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .context("Failed to read settings file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .context("Failed deserializing RON file")?;
        let settings = Settings::deserialize(&mut d)
            .context("Failed parsing Ron file")?;
        d.end()
            .context("Failed parsing Ron file")?;

        Ok(settings) 
    }
    
    /// Write the `Settings` into the given file.
    pub fn write_to_file(&self, mut file: File) -> Result<(), FileError> {
        self.generate_ron_into_file(&mut file)
    }

    /// Parses a `Settings` from a file using the RON format.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), FileError> {
        debug!("Serializing & writing Settings file.");
        let pretty = ron::ser::PrettyConfig::new()
            .with_depth_limit(2)
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true)
            .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME);
        let s = ron::ser::to_string_pretty(&self, pretty)
            .context("Failed to serialize RON file")?;
        file.write_all(s.as_bytes())
            .context("Failed to write RON file")
    }

    /// Normalizes paths in the settings by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, base: &PathBuf) {
        self.active_palette = self.active_palette
            .as_ref()
            .map(|p| normalize_path(base, p));
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
