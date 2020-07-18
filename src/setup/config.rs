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
use crate::setup::LevelFilter;
use crate::setup::LoggerConfig;
use crate::command::CursorBehavior;
use crate::command::Positioning;
use crate::error::FileError;
use crate::error::FileErrorContext as _;
use crate::setup::StdoutLogOutput;
use crate::setup::LoadStatus;
use crate::utility::normalize_path;

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

/// Default value for default_settings_path.
const DEFAULT_DEFAULT_SETTINGS_PATH: &'static str = ".atma-settings";

/// The default value for default_palette_path.
pub const DEFAULT_DEFAULT_PALETTE_PATH: &'static str = "palette.atma";

/// Default value for load_default_palette.
const DEFAULT_LOAD_DEFAULT_PALETTE: bool = true;

/// Default value for default_positioning.
const DEFAULT_DEFAULT_POSITIONING: Positioning = Positioning::Cursor;

/// Default value for default_delete_cursor_behavior.
pub const DEFAULT_DEFAULT_DELETE_CURSOR_BEHAVIOR: CursorBehavior
    = CursorBehavior::MoveToStart;

/// Default value for default_default_delete_cursor_behavior.
pub const DEFAULT_DEFAULT_INSERT_CURSOR_BEHAVIOR: CursorBehavior
    = CursorBehavior::MoveAfterEnd;

/// Default value for default_default_move_cursor_behavior.
pub const DEFAULT_DEFAULT_MOVE_CURSOR_BEHAVIOR: CursorBehavior
    = CursorBehavior::RemainInPlace;

////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////
/// Application configuration config. Configures the logger and application
/// behavior.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The Config file's load status.
    #[serde(skip)]
    load_status: LoadStatus,

    /// The logger configuration.
    #[serde(default = "Config::default_logger_config")]
    pub logger_config: LoggerConfig,

    /// Module specific log levels.
    #[serde(default = "Config::default_log_levels")]
    pub log_levels: BTreeMap<Cow<'static, str>, LevelFilter>,

    /// The default path to look for the [`Settings`] file, relative to the
    /// application root.
    ///
    /// [`Settings`]: struct.Settings.html
    #[serde(default = "Config::default_default_settings_path")]
    pub default_settings_path: PathBuf,

    /// The default path for the [`Palette`] file, relative to the application
    /// root.
    ///
    /// [`Palette`]: struct.Palette.html
    #[serde(default = "Config::default_default_palette_path")]
    pub default_palette_path: PathBuf,

    /// Attempt to load the default palette if no active palette is set.
    #[serde(default = "Config::default_load_default_palette")]
    pub load_default_palette: bool,

    /// Default value when positioning is not given.
    #[serde(default = "Config::default_default_positioning")]
    pub default_positioning: Positioning,


    /// The default behavior of the cursor after a delete command is run.
    #[serde(default = "Config::default_default_delete_cursor_behavior")]
    pub default_delete_cursor_behavior: CursorBehavior,
    
    /// The default behavior of the cursor after an insert command is run.
    #[serde(default = "Config::default_default_insert_cursor_behavior")]
    pub default_insert_cursor_behavior: CursorBehavior,

    /// The default behavior of the cursor after a move command is run.
    #[serde(default = "Config::default_default_move_cursor_behavior")]
    pub default_move_cursor_behavior: CursorBehavior,
}


impl Config {
    /// Constructs a new `Config` with the default options.
    pub fn new() -> Self {
        Config {
            load_status: LoadStatus::default(),
            logger_config: Config::default_logger_config(),
            log_levels: Config::default_log_levels(),
            default_settings_path: Config::default_default_settings_path(),
            default_palette_path: Config::default_default_palette_path(),
            load_default_palette: DEFAULT_LOAD_DEFAULT_PALETTE,
            default_positioning: DEFAULT_DEFAULT_POSITIONING,
            default_delete_cursor_behavior: 
                DEFAULT_DEFAULT_DELETE_CURSOR_BEHAVIOR,
            default_insert_cursor_behavior: 
                DEFAULT_DEFAULT_INSERT_CURSOR_BEHAVIOR,
            default_move_cursor_behavior: 
                DEFAULT_DEFAULT_MOVE_CURSOR_BEHAVIOR,
        }
    }

    /// Returns the given `Config` with the given load_path.
    pub fn with_load_path<P>(mut self, path: P) -> Self
        where P: AsRef<Path>
    {
        self.set_load_path(path);
        self
    }

    /// Returns the `Config`'s load path.
    pub fn load_path(&self) -> Option<&Path> {
        self.load_status.load_path()
    }

    /// Sets the `Config`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_status.set_load_path(path);
    }

    /// Returns true if the Config was modified.
    pub fn modified(&self) -> bool {
        self.load_status.modified()
    }

    /// Sets the Config modification flag.
    pub fn set_modified(&mut self, modified: bool) {
        self.load_status.set_modified(modified);
    }

    /// Constructs a new `Config` with options read from the given file path.
    pub fn read_from_path<P>(path: P) -> Result<Self, FileError> 
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!(
                "Failed to open config file for reading: {}",
                path.display()))?;
        let mut config = Config::read_from_file(file)?;
        config.set_load_path(path);
        Ok(config)
    }

    /// Open a file at the given path and write the `Config` into it.
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
                "Failed to create/open config file for writing: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write config file")?;
        Ok(())
    }
    
    /// Create a new file at the given path and write the `Config` into it.
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
                "Failed to create config file: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write config file")?;
        Ok(())
    }

    /// Write the `Config` into the file is was loaded from. Returns true if the
    /// data was written.
    pub fn write_to_load_path(&self)
        -> Result<bool, FileError>
    {
        match self.load_status.load_path() {
            Some(path) => {
                self.write_to_path(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Write the `Config` into a new file using the load path. Returns true
    /// if the data was written.
    pub fn write_to_load_path_if_new(&self)
        -> Result<bool, FileError>
    {
        match self.load_status.load_path() {
            Some(path) => {
                self.write_to_path_if_new(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Constructs a new `Config` with options parsed from the given file.
    pub fn read_from_file(mut file: File) -> Result<Self, FileError>  {
        Config::parse_ron_from_file(&mut file)
    }

    /// Parses a `Config` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, FileError> {
        let len = file.metadata()
            .context("Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .context("Failed to read config file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .context("Failed deserializing RON file")?;
        let config = Config::deserialize(&mut d)
            .context("Failed parsing RON file")?;
        d.end()
            .context("Failed parsing RON file")?;

        Ok(config) 
    }

    /// Write the `Config` into the given file.
    pub fn write_to_file(&self, mut file: File) -> Result<(), FileError> {
        self.generate_ron_into_file(&mut file)
    }

    /// Parses a `Config` from a file using the RON format.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), FileError> {
        debug!("Serializing & writing Config file.");
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

    /// Normalizes paths in the config by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, base: &PathBuf) {
        self.logger_config.log_path = self.logger_config.log_path
            .as_ref()
            .map(|p| normalize_path(base, p));
    }

    ////////////////////////////////////////////////////////////////////////////
    // Default constructors for serde.
    ////////////////////////////////////////////////////////////////////////////

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

    /// Returns the default value for default_settings_path.
    #[inline(always)]
    fn default_default_settings_path() -> PathBuf {
        DEFAULT_DEFAULT_SETTINGS_PATH.to_owned().into()
    }

    /// Returns the default value for default_palette_path.
    #[inline(always)]
    fn default_default_palette_path() -> PathBuf {
        DEFAULT_DEFAULT_PALETTE_PATH.to_owned().into()
    }

    /// Returns the default value for load_default_palette.
    #[inline(always)]
    fn default_load_default_palette() -> bool {
        DEFAULT_LOAD_DEFAULT_PALETTE
    }

    /// Returns the default value for default_positioning.
    #[inline(always)]
    fn default_default_positioning() -> Positioning {
        DEFAULT_DEFAULT_POSITIONING
    }

    /// Returns the default value for default_delete_cursor_behavior.
    #[inline]
    pub fn default_default_delete_cursor_behavior() -> CursorBehavior {
        DEFAULT_DEFAULT_DELETE_CURSOR_BEHAVIOR
    }

    /// Returns the default value for default_insert_cursor_behavior.
    #[inline]
    pub fn default_default_insert_cursor_behavior() -> CursorBehavior {
        DEFAULT_DEFAULT_INSERT_CURSOR_BEHAVIOR
    }

    /// Returns the default value for default_move_cursor_behavior.
    #[inline]
    pub fn default_default_move_cursor_behavior() -> CursorBehavior {
        DEFAULT_DEFAULT_MOVE_CURSOR_BEHAVIOR
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
            self.logger_config.level_filter)?;
        writeln!(fmt, "\tload_default_palette: {:?}",
            self.load_default_palette)?;
        writeln!(fmt, "\tdefault_positioning: {:?}",
            self.default_positioning)
    }
}
