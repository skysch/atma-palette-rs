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
use crate::cell::CellSelection;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::command::Positioning;
use crate::command::CursorBehavior;
use crate::command::HistorySetOption;
use crate::palette::InsertExpr;

// External library imports.
use structopt::StructOpt;

// Standard library imports.
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
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

impl AtmaOptions {
    /// Returns true if the commands depends on the palette.
    pub fn should_load_palette(&self) -> bool {
        match &self.command {
            CommandOption::New { .. } => false,
            CommandOption::Set { set_option } => match set_option {
                SetOption::ActivePalette { .. } |
                SetOption::DeleteCursorBehavior { .. } |
                SetOption::InsertCursorBehavior { .. } |
                SetOption::MoveCursorBehavior { .. } => false,
                _ => true,
            },
            _ => true,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
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
#[derive(StructOpt)]
pub enum CommandOption {
    /// Create a new palette.
    New {
        #[structopt(subcommand)]
        new_option: NewOption,
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
        /// The color expression objects to insert.
        exprs: Vec<InsertExpr>,

        /// The name of the insert group.
        #[structopt(long = "name")]
        name: Option<String>,

        /// The start position for the inserted objects.
        #[structopt(long = "at")]
        at: Option<Positioning>,
    },
    
    /// Delete colors and ramps from a palette.
    Delete {
        /// The selection of palette cells to export.
        selection: Option<CellSelection<'static>>,
    },

    /// Move colors and ramps in a palette.
    Move {
        /// The selection of palette cells to export.
        selection: Option<CellSelection<'static>>,

        /// The position to move the cells to.
        #[structopt(long = "to")]
        to: Option<Positioning>
    },

    /// Change settings, or assign color expressions, names, or metadata to
    /// cells.
    Set {
        #[structopt(subcommand)]
        set_option: SetOption,
    },
    
    /// Revert previous operations.
    Undo {
        /// The number of operations to revert.
        count: Option<usize>,
    },
    /// Reapply previously reverted operations.
    Redo {
        /// The number of operations to reapply.
        count: Option<usize>,
    },
    
    /// Import color data into a palette.
    Import,
    /// Export palette data.
    Export {
        #[structopt(subcommand)]
        export_option: ExportOption,
    },
}


////////////////////////////////////////////////////////////////////////////////
// NewOption
////////////////////////////////////////////////////////////////////////////////
/// Options for the new command.
#[derive(Debug, Clone)]
#[derive(StructOpt)]
pub enum NewOption {
    /// Create a new palette.
    Palette {
        /// The path of the new palette.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
        
        /// Sets the palette as the default active palette.
        #[structopt(long = "set-active")]
        set_active: bool,

        /// Disables undo/redo operations for the palette.
        #[structopt(long = "no-history")]
        no_history: bool,

        /// The name of the palette.
        #[structopt(long = "name")]
        name: Option<String>,
    },
    
    /// Create a new config file.
    Config {
        /// The path of the new config file.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },

    /// Create a new settings file.
    Settings {
        /// The path of the new settings file.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },
}


////////////////////////////////////////////////////////////////////////////////
// ExportOption
////////////////////////////////////////////////////////////////////////////////
/// Options for the set command.
#[derive(Debug, Clone)]
#[derive(StructOpt)]
pub enum SetOption {
    /// Assign or unassign a name to a position selector.
    Name {
        /// The position selector to name.
        position_selector: PositionSelector,

        /// The name to assign.
        name: Option<String>,
    },

    /// Assign or unassign selected cells to a group.
    Group {
        /// The selection to assign or unassign.
        selection: CellSelection<'static>,

        /// The name to assign.
        name: Option<String>,

        /// Unassign the group from the selected cells.
        #[structopt(long = "remove")]
        remove: bool
    },

    /// Assign a color expression to a cell.
    Expr {
        /// The cell to set the expression for.
        at: CellRef<'static>,

        /// The color expression to set.
        expr: InsertExpr,
    },

    /// Sets the palette cursor position.
    Cursor {
        /// The cursor position.
        position: Position,
    },

    /// Sets the history for the palette.
    History {
        /// The history setting.
        history_set_option: HistorySetOption,
    },

    /// Sets the active palette.
    ActivePalette {
        /// The path of the active palette.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },

    /// Sets the cursor positioning behavior for the delete command.
    DeleteCursorBehavior {
        /// The behavior of the cursor.
        cursor_behavior: Option<CursorBehavior>,
    },

    /// Sets the cursor positioning behavior for the delete command.
    InsertCursorBehavior {
        /// The behavior of the cursor.
        cursor_behavior: Option<CursorBehavior>,
    },

    /// Sets the cursor positioning behavior for the delete command.
    MoveCursorBehavior {
        /// The behavior of the cursor.
        cursor_behavior: Option<CursorBehavior>,
    },
}


////////////////////////////////////////////////////////////////////////////////
// ExportOption
////////////////////////////////////////////////////////////////////////////////
/// Options for the export command.
#[derive(Debug, Clone)]
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
