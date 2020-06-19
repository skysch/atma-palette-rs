////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette data.
////////////////////////////////////////////////////////////////////////////////

use color::Color;

// External library imports.
// use anyhow::Error;
// use anyhow::Context as _;

use serde::Deserialize;
use serde::Serialize;

use ron::ser::PrettyConfig;
use ron::ser::to_string_pretty;

// Standard library imports.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;




#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Palette {
    cell_names: BTreeMap<String, usize>,
	cells: Vec<ColorCell>,
}


impl Palette {
	pub fn new() -> Self {
		Palette {
            cell_names: BTreeMap::new(),
			cells: Vec::new(),
		}
	}

    // /// Constructs a new `Palette` by parsing data from the file at the given
    // /// path.
    // pub fn new_from_path(path: &Path) -> Result<Self, Error>  {
    //     let mut file = OpenOptions::new()
    //         .read(true)
    //         .open(path)?;
    //     Palette::new_from_file(&mut file)
    // }

    // /// Constructs a new `Palette` by parsing data from the given file.
    // pub fn new_from_file(file: &mut File) -> Result<Self, Error>  {
    //     Palette::parse_ron_from_file(file)
    // }

    // /// Writes the `Palette` to the file at the given path.
    // pub fn write_to_path(&self, path: &Path) -> Result<(), Error>  {
    //     let mut file = OpenOptions::new()
    //         .read(true)
    //         .write(true)
    //         .create(true)
    //         .open(path)?;
    //     self.write_to_file(&mut file)
    // }

    // /// Writes the `Palette` to the given file.
    // pub fn write_to_file(&self, file: &mut File) -> Result<(), Error>  {
    //     self.generate_ron_into_file(file)
    // }

    // /// Parses a `Palette` from a file using the RON format.
    // fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
    //     let len = file.metadata()
    //         .with_context(|| "Failed to recover file metadata.")?
    //         .len();
    //     let mut buf = Vec::with_capacity(len as usize);
    //     let _ = file.read_to_end(&mut buf)
    //         .with_context(|| "Failed to read config file")?;

    //     use ron::de::Deserializer;
    //     let mut d = Deserializer::from_bytes(&buf)
    //         .with_context(|| "Failed deserializing RON file")?;
    //     let config = Palette::deserialize(&mut d)
    //         .with_context(|| "Failed parsing Ron file")?;
    //     d.end()
    //         .with_context(|| "Failed parsing Ron file")?;

    //     Ok(config)
    // }

    // /// Generates a RON formatted `Palette` by serializing into the given file.
    // fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
    //     let pretty = PrettyConfig::new()
    //         .with_depth_limit(2)
    //         .with_separate_tuple_members(true)
    //         .with_enumerate_arrays(true);
    //     let s = to_string_pretty(self, pretty)?;

    //     file.write_all(s.as_bytes())?;
    //     Ok(())
    // }
}

impl Default for Palette {
	fn default() -> Self {
		Palette::new()
	}
}




#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ColorCell {
	expr: Expr,
}


#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum Expr {
	/// An color expression with no color.
	Empty,
	/// A color.
	Color(Color),
	
}
