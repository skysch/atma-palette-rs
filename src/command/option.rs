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
    pub command: Option<CommandOption>,
}

////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub struct CommonOptions {
    /// The config file to use.
    #[structopt(
        long = "config-file",
        parse(from_os_str))]
    pub config_file: Option<PathBuf>,

    /// The settings file to use.
    #[structopt(
        long = "settings-file",
        parse(from_os_str))]
    pub settings_file: Option<PathBuf>,

    /// The palette file to use.
    #[structopt(
        short = "p",
        long = "palette",
        parse(from_os_str))]
    pub palette: Option<PathBuf>,

    /// Print palette operations instead of running them.
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
    
    /// Provides more detailed messages.
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Silences all program output. This override --verbose if both are provided.
    #[structopt(short = "q", long = "quiet", alias = "silent")]
    pub quiet: bool,

    /// Print trace messages. This override --quiet if both are provided.
    #[structopt(long = "ztrace", hidden(true))]
    pub trace: bool,
}

////////////////////////////////////////////////////////////////////////////////
// CommandOption
////////////////////////////////////////////////////////////////////////////////
/// Command line subcommand options.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub enum CommandOption {
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

    List,
    Insert {
        #[structopt(subcommand)]
        insert_options: InsertOption,
    },
    Delete,
    Move,
    Set,
    Unset,
    Undo {
        /// The number of times to undo.
        count: usize,
    },
    Redo {
        /// The number of times to redo.
        count: usize,
    },
    Import,
    Export,
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


#[allow(missing_docs)]
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

        /// The position of the ramp start
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

