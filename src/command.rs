////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface options.
////////////////////////////////////////////////////////////////////////////////


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
    pub command: Option<CommandOptions>,
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
        short = "u",
        long = "use-config",
        parse(from_os_str))]
    pub use_config: Option<PathBuf>,

    /// Print copy operations instead of running them.
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
    
    /// Shorten filenames by omitting path prefixes.
    #[structopt(short = "s", long = "short-names")]
    pub short_names: bool,
    
    /// Promote file access warnings into errors.
    #[structopt(short = "e", long = "error")]
    pub promote_warnings_to_errors: bool,
    
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
// CommandOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line subcommand options.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub enum CommandOptions {
    List,
    Insert,
    Delete,
    Move,
    Set,
    Unset,
    Undo,
    Redo,
    Import,
    Export,
}

