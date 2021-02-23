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
use crate::command::ColorDisplay;
use crate::command::ColorStyle;
use crate::command::CursorBehavior;
use crate::command::GutterStyle;
use crate::command::LineStyle;
use crate::command::ListMode;
use crate::command::Positioning;
use crate::command::RuleStyle;
use crate::command::TextStyle;
use crate::error::FileError;
use crate::error::FileErrorContext as _;
use crate::setup::LoadStatus;
use crate::setup::TraceConfig;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Read as _;
use std::io::Write as _;
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
pub const DEFAULT_DEFAULT_PALETTE_PATH: &'static str = "new.atma-palette";

/// Default value for load_default_palette.
const DEFAULT_LOAD_DEFAULT_PALETTE: bool = true;

/// Default value for new_from_script_history.
const DEFAULT_NEW_FROM_SCRIPT_HISTORY: bool = false;

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

/// Default value for default_list_mode.
pub const DEFAULT_DEFAULT_LIST_MODE: ListMode = ListMode::Lines;

/// Default value for default_list_color_style.
pub const DEFAULT_DEFAULT_LIST_COLOR_STYLE: ColorStyle = ColorStyle::Tile;

/// Default value for default_list_text_style.
pub const DEFAULT_DEFAULT_LIST_TEXT_STYLE: TextStyle = TextStyle::Hex6;

/// Default value for default_list_rule_style.
pub const DEFAULT_DEFAULT_LIST_RULE_STYLE: RuleStyle = RuleStyle::Colored;

/// Default value for default_list_line_style.
pub const DEFAULT_DEFAULT_LIST_LINE_STYLE: LineStyle = LineStyle::Auto;

/// Default value for default_list_gutter_style.
pub const DEFAULT_DEFAULT_LIST_GUTTER_STYLE: GutterStyle = GutterStyle::Auto;

/// The default value for  invalid_color_display_fallback.
pub const DEFAULT_INVALID_COLOR_DISPLAY_FALLBACK: ColorDisplay = ColorDisplay {
    color_style: ColorStyle::None,
    text_style: TextStyle::Hex6,
};


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

    /// The trace configuration.
    #[serde(default = "Config::default_trace_config")]
    pub trace_config: TraceConfig,

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

    /// Whether to include script commands in the history when using `new
    /// --from-script`.
    #[serde(default = "Config::default_new_from_script_history")]
    pub new_from_script_history: bool,

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

    /// The default ListMode for the list command.
    #[serde(default = "Config::default_default_list_mode")]
    pub default_list_mode: ListMode,

    /// The default ColorStyle for the list command.
    #[serde(default = "Config::default_default_list_color_style")]
    pub default_list_color_style: ColorStyle,

    /// The default TextStyle for the list command.
    #[serde(default = "Config::default_default_list_text_style")]
    pub default_list_text_style: TextStyle,

    /// The default RuleStyle for the list command.
    #[serde(default = "Config::default_default_list_rule_style")]
    pub default_list_rule_style: RuleStyle,

    /// The default LineStyle for the list command.
    #[serde(default = "Config::default_default_list_line_style")]
    pub default_list_line_style: LineStyle,

    /// The default GutterStyle for the list command.
    #[serde(default = "Config::default_default_list_gutter_style")]
    pub default_list_gutter_style: GutterStyle,

    /// The fallback ColorDisplay for when the provided combination is invalid.
    #[serde(default = "Config::default_invalid_color_display_fallback")]
    pub invalid_color_display_fallback: ColorDisplay,
}


impl Config {
    /// Constructs a new `Config` with the default options.
    pub fn new() -> Self {
        Config {
            load_status: LoadStatus::default(),
            trace_config: Config::default_trace_config(),
            default_settings_path: Config::default_default_settings_path(),
            default_palette_path: Config::default_default_palette_path(),
            load_default_palette: DEFAULT_LOAD_DEFAULT_PALETTE,
            new_from_script_history: DEFAULT_NEW_FROM_SCRIPT_HISTORY,
            default_positioning: DEFAULT_DEFAULT_POSITIONING,
            default_delete_cursor_behavior: 
                DEFAULT_DEFAULT_DELETE_CURSOR_BEHAVIOR,
            default_insert_cursor_behavior: 
                DEFAULT_DEFAULT_INSERT_CURSOR_BEHAVIOR,
            default_move_cursor_behavior: 
                DEFAULT_DEFAULT_MOVE_CURSOR_BEHAVIOR,
            default_list_mode: DEFAULT_DEFAULT_LIST_MODE,
            default_list_color_style: DEFAULT_DEFAULT_LIST_COLOR_STYLE,
            default_list_text_style: DEFAULT_DEFAULT_LIST_TEXT_STYLE,
            default_list_rule_style: DEFAULT_DEFAULT_LIST_RULE_STYLE,
            default_list_line_style: DEFAULT_DEFAULT_LIST_LINE_STYLE,
            default_list_gutter_style: DEFAULT_DEFAULT_LIST_GUTTER_STYLE,
            invalid_color_display_fallback:
                DEFAULT_INVALID_COLOR_DISPLAY_FALLBACK,
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
        tracing::debug!("Serializing & writing Config file.");
        let pretty = ron::ser::PrettyConfig::new()
            .with_depth_limit(2)
            .with_separate_tuple_members(true)
            .with_enumerate_arrays(true)
            .with_extensions(ron::extensions::Extensions::IMPLICIT_SOME);
        let s = ron::ser::to_string_pretty(&self, pretty)
            .context("Failed to serialize RON file")?;
        let mut writer = BufWriter::new(file);
        writer.write_all(s.as_bytes())
            .context("Failed to write RON file")?;
        writer.flush()
            .context("Failed to flush file buffer")
    }

    /// Normalizes paths in the config by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, _base: &PathBuf) {
        // NOTE: No paths currently in config, nothing to do.

        // TODO: Normalize trace path?
    }

    ////////////////////////////////////////////////////////////////////////////
    // Default constructors for serde.
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the default [`TraceConfig`].
    ///
    /// [`TraceConfig`]: ../logger/struct.TraceConfig.html
    #[inline(always)]
    fn default_trace_config() -> TraceConfig {
        TraceConfig::default()
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

    /// Returns the default value for new_from_script_history.
    #[inline(always)]
    fn default_new_from_script_history() -> bool {
        DEFAULT_NEW_FROM_SCRIPT_HISTORY
    }

    /// Returns the default value for default_positioning.
    #[inline(always)]
    fn default_default_positioning() -> Positioning {
        DEFAULT_DEFAULT_POSITIONING
    }

    /// Returns the default value for default_delete_cursor_behavior.
    #[inline]
    fn default_default_delete_cursor_behavior() -> CursorBehavior {
        DEFAULT_DEFAULT_DELETE_CURSOR_BEHAVIOR
    }

    /// Returns the default value for default_insert_cursor_behavior.
    #[inline]
    fn default_default_insert_cursor_behavior() -> CursorBehavior {
        DEFAULT_DEFAULT_INSERT_CURSOR_BEHAVIOR
    }

    /// Returns the default value for default_move_cursor_behavior.
    #[inline]
    pub fn default_default_move_cursor_behavior() -> CursorBehavior {
        DEFAULT_DEFAULT_MOVE_CURSOR_BEHAVIOR
    }

    /// Returns the default value for default_list_mode.
    #[inline]
    fn default_default_list_mode() -> ListMode {
        DEFAULT_DEFAULT_LIST_MODE
    }

    /// Returns the default value for default_list_color_style.
    #[inline]
    fn default_default_list_color_style() -> ColorStyle {
        DEFAULT_DEFAULT_LIST_COLOR_STYLE
    }

    /// Returns the default value for default_list_text_style.
    #[inline]
    fn default_default_list_text_style() -> TextStyle {
        DEFAULT_DEFAULT_LIST_TEXT_STYLE
    }

    /// Returns the default value for default_list_rule_style.
    #[inline]
    fn default_default_list_rule_style() -> RuleStyle {
        DEFAULT_DEFAULT_LIST_RULE_STYLE
    }

    /// Returns the default value for default_list_line_style.
    #[inline]
    fn default_default_list_line_style() -> LineStyle {
        DEFAULT_DEFAULT_LIST_LINE_STYLE
    }

    /// Returns the default value for default_list_gutter_style.
    #[inline]
    fn default_default_list_gutter_style() -> GutterStyle {
        DEFAULT_DEFAULT_LIST_GUTTER_STYLE
    }

    /// Returns the default value for invalid_color_display_fallback.
    #[inline]
    fn default_invalid_color_display_fallback() -> ColorDisplay {
        DEFAULT_INVALID_COLOR_DISPLAY_FALLBACK
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\ttrace_config.output_path: {:?}",
            self.trace_config.output_path)?;
        writeln!(fmt, "\ttrace_config.ansi_colors: {:?}",
            self.trace_config.ansi_colors)?;
        writeln!(fmt, "\ttrace_config.output_stdout: {:?}",
            self.trace_config.output_stdout)?;
        writeln!(fmt, "\ttrace_config.filters:")?;
        for filter in &self.trace_config.filters {
            writeln!(fmt, "\t\t{:?}", filter)?;
        }
        writeln!(fmt, "\tload_default_palette: {:?}",
            self.load_default_palette)?;
        writeln!(fmt, "\tnew_from_script_history: {:?}",
            self.new_from_script_history)?;
        writeln!(fmt, "\tdefault_positioning: {:?}",
            self.default_positioning)?;
        writeln!(fmt, "\tdefault_delete_cursor_behavior: {:?}",
            self.default_delete_cursor_behavior)?;
        writeln!(fmt, "\tdefault_insert_cursor_behavior: {:?}",
            self.default_insert_cursor_behavior)?;
        writeln!(fmt, "\tdefault_move_cursor_behavior: {:?}",
            self.default_move_cursor_behavior)
    }
}
