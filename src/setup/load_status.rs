////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File source and modification tracking.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]


// Standard library imports.
use std::path::PathBuf;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// LoadStatus
////////////////////////////////////////////////////////////////////////////////
/// Structure for tracking a file's load status.
#[derive(Debug, Clone)]
pub struct LoadStatus {
    /// The path the data was initially loaded from.
    load_path: Option<PathBuf>,
    /// Whether the data has been modified since last save.
    modified: bool,
}

impl LoadStatus {
    /// Constructs a new `LoadStatus`.
    pub fn new() -> Self {
        LoadStatus {
            load_path: None,
            modified: false,
        }
    }

    /// Returns the `LoadStatus`'s load path.
    pub fn load_path(&self) -> Option<&Path> {
        self.load_path.as_ref().map(AsRef::as_ref)
    }

    /// Sets the `LoadStatus`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_path = Some(path.as_ref().to_owned());
    }

    /// Clears the `LoadStatus`'s load path.
    pub fn clear_load_path<P>(&mut self) {
        self.load_path = None;
    }

    /// Returns true if the data was modified.
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// Sets the data modification flag.
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified
    }
}

impl Default for LoadStatus {
    fn default() -> Self {
        LoadStatus::new()
    }
}
