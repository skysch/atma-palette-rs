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
use crate::LevelFilter;
use crate::LoggerConfig;
use crate::StdoutLogOutput;
use anyhow::Error;
use anyhow::Context;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

use log::*;

// Standard library imports.
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// DEFAULT_CONFIG_PATH
////////////////////////////////////////////////////////////////////////////////
/// The default path to look for the [`Config`] file, relative to the
/// application root.
///
/// [`Config`]: struct.Config.html
pub const DEFAULT_CONFIG_PATH: &'static str = ".atma-config";

////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////
/// Application configuration config. Configures the logger and application
/// behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The path the config was initially loaded from.
    #[serde(skip)]
    load_path: Option<PathBuf>,

    /// The logger configuration.
    #[serde(default = "Config::default_logger_config")]
    pub logger_config: LoggerConfig,

    /// Module specific log levels.
    #[serde(default = "Config::default_log_levels")]
    pub log_levels: BTreeMap<Cow<'static, str>, LevelFilter>,
}


impl Config {
    /// Constructs a new `Config` with the default options.
    pub fn new() -> Self {
        Config {
            load_path: None,
            logger_config: Config::default_logger_config(),
            log_levels: Config::default_log_levels(),
        }
    }

    /// Returns the given `Config` with the given load_path.
    pub fn with_load_path<P>(mut self, path: P) -> Self
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
        self
    }

    /// Constructs a new `Config` with options read from the given file path.
    pub fn read_from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let path = path.as_ref().to_owned();
        let file = File::open(&path)
            .with_context(|| "Failed to open config file.")?;
        let mut config = Config::read_from_file(file)?;
        config.load_path = Some(path);
        Ok(config)
    }

    /// Constructs a new `Config` with options parsed from the given file.
    pub fn read_from_file(mut file: File) -> Result<Self, Error>  {
        Config::parse_ron_from_file(&mut file)
    }

    /// Parses a `Config` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .with_context(|| "Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .with_context(|| "Failed to read config file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .with_context(|| "Failed deserializing RON file")?;
        let config = Config::deserialize(&mut d)
            .with_context(|| "Failed parsing RON file")?;
        d.end()
            .with_context(|| "Failed parsing RON file")?;

        Ok(config) 
    }
    
    /// Open a file at the given path and write the `Config` into it.
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

    /// Write the `Config` into the file is was loaded from. Returns true if the
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

    /// Write the `Config` into the given file.
    pub fn write_to_file(&self, mut file: File) -> Result<(), Error> {
        self.generate_ron_into_file(&mut file)
    }

    /// Parses a `Config` from a file using the RON format.
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

    /// Sets the `Config`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
    }

    /// Normalizes paths in the config by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, base: &PathBuf) {
        match self.logger_config.log_path {
            Some(ref log_path) if log_path.is_relative() => {
                let log_path = base.clone().join(log_path);
                // Relative log file paths are relative to base.
                self.logger_config.log_path = Some(log_path);
            },
            _ => (),
        }
    }

    /// Returns the default [`LoggerConfig`].
    ///
    /// [`LoggerConfig`]: ../logger/struct.LoggerConfig.html
    #[inline(always)]
    fn default_logger_config() -> LoggerConfig {
        LoggerConfig {
            stdout_log_output: StdoutLogOutput::Colored,
            .. Default::default()
        }
    }

    /// Returns the default log levels for modules.
    #[inline(always)]
    fn default_log_levels() -> BTreeMap<Cow<'static, str>, LevelFilter> {
        Default::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\n\tlogger_config/stdout_log_output: {:?}",
            self.logger_config.stdout_log_output)?;
        writeln!(fmt, "\tlogger_config/level_filter: {:?}",
            self.logger_config.level_filter)
    }
}
