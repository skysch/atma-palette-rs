////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line options.
////////////////////////////////////////////////////////////////////////////////
#![allow(variant_size_differences)] // TODO: Remove this.

// Local library imports.
use crate::cell::Position;
use crate::cell::CellSelection;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

use structopt::StructOpt;

// Standard library imports.
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
#[structopt(name = "atma")]
pub struct AtmaOptions {
    // Options common to all commands.
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub common: CommonOptions,
    /// Subcommand options.
    #[structopt(subcommand)]
    pub command: CommandOption,
}


////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub struct CommonOptions {
    /// The application config file to load.
    #[structopt(
        long = "config",
        parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// The user settings file to load.
    #[structopt(
        long = "settings",
        parse(from_os_str))]
    pub settings: Option<PathBuf>,

    /// The palette file to load.
    #[structopt(
        short = "p",
        long = "palette",
        parse(from_os_str))]
    pub palette: Option<PathBuf>,
    
    /// Provides more detailed messages.
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Silences all program output. (Overrides -v if both are provided.)
    #[structopt(short = "q", long = "quiet", alias = "silent")]
    pub quiet: bool,

    /// Print trace messages. (Overrides -q if both are provided.)
    #[structopt(long = "ztrace", hidden(true))]
    pub trace: bool,
}


////////////////////////////////////////////////////////////////////////////////
// CommandOption
////////////////////////////////////////////////////////////////////////////////
/// Atma palette editing commands.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub enum CommandOption {
    /// Create a new palette.
    New {
        /// The name of the palette.
        #[structopt(long = "name")]
        name: Option<String>,

        /// Disables undo/redo operations for the palette.
        #[structopt(long = "no-history")]
        no_history: bool,

        /// Creates a config file in the palette directory.
        #[structopt(long = "no_config_file")]
        no_config_file: bool,

        /// Creates a settings file in the palette directory.
        #[structopt(long = "no_settings_file")]
        no_settings_file: bool,
        
        /// Sets the palette as the default active palette.
        #[structopt(long = "set-active")]
        set_active: bool,
    },

    /// List palette contents.
    List {
        // TODO: Consider generalizing this to a string so we can parse simpler
        // selection terms?
        /// The selection of palette cells to list.
        selection: Option<CellSelection<'static>>,

        // Print by index or by page?
        // Display width?
        // Use colors?
        // Print names and groups?
        // Indicate expr types?
        // Indicate names, groups, positions?
        // Sort?
        // Compact?
    },

    /// Insert colors and ramps into a palette.
    Insert {
        #[structopt(subcommand)]
        insert_option: InsertOption,
    },
    
    /// Delete colors and ramps from a palette.
    Delete {
        // TODO: Consider generalizing this to a string so we can parse simpler
        // selection terms?
        /// The selection of palette cells to export.
        selection: Option<CellSelection<'static>>,
    },

    /// Move colors and ramps in a palette.
    Move,
    /// Set color expressions, names, or metadata for cells.
    Set,
    /// Unset color expressions, names, or metadata for cells.
    Unset,
    
    /// Revert previous operations.
    Undo {
        /// The number of operations to revert.
        count: usize,
    },
    /// Reapply previously reverted operations.
    Redo {
        /// The number of operations to reapply.
        count: usize,
    },
    
    /// Import color data into a palette.
    Import,
    /// Export palette data.
    Export {
        #[structopt(subcommand)]
        export_option: ExportOption,
    },
}

impl CommandOption {
    /// Returns true if the variant is `CommandOption::New`.
    pub fn is_new(&self) -> bool {
        match self {
            CommandOption::New { .. } => true,
            _ => false,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// InsertOption
////////////////////////////////////////////////////////////////////////////////
/// Options for the insert command.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub enum InsertOption {
    /// Inserts colors into the palette.
    Colors {
        /// The colors to insert.
        colors: Vec<String>,

        /// The name of the colors group.
        #[structopt(long = "name")]
        name: Option<String>,

        /// The position of the ramp start.
        #[structopt(long = "at")]
        at: Option<Position>
    },

    /// Insert a ramp into the palette.
    Ramp {
        /// The ramp interpolation points.
        points: Vec<String>,

        /// The number of colors in the ramp.
        #[structopt(short = "c", long = "count")]
        count: usize,

        /// The ramp interpolation function.
        #[structopt(short = "i", long = "interpolate")]
        interpolate: Option<String>,

        /// The name of the ramp group.
        #[structopt(long = "name")]
        name: Option<String>,

        /// The position of the ramp start
        #[structopt(long = "at")]
        at: Option<Position>
    },
}

////////////////////////////////////////////////////////////////////////////////
// ExportOption
////////////////////////////////////////////////////////////////////////////////
/// Options for the export command.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub enum ExportOption {
    /// Export palette data as a PNG file.
    Png {
        // TODO: Consider generalizing this to a string so we can parse simpler
        // selection terms?
        /// The selection of palette cells to export.
        selection: Option<CellSelection<'static>>,

        /// The output file name.
        #[structopt(
            short = "o",
            long = "output",
            parse(from_os_str))]
        output: PathBuf,
    },
}
