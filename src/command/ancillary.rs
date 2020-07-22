////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Ancillary command option inputs.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::color::Color;
use crate::cell::Position;
use crate::parse::any_literal_map_once;
use crate::parse::FailureOwned;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::uint;
use crate::parse::ParseResultExt as _;
use crate::parse::position;


// External library imports.
use colored::Colorize as _;
use serde::Serialize;
use serde::Deserialize;



////////////////////////////////////////////////////////////////////////////////
// CursorBehavior
////////////////////////////////////////////////////////////////////////////////
/// The behavior of the cursor after an operation is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum CursorBehavior {
    /// Move the cursor to the lowest position in the affected selection.
    MoveToStart,
    /// Move the cursor after the highest position in the affected selection.
    MoveAfterEnd,
    /// Move the cursor to the lowest open position.
    MoveToOpen,
    /// Do not move the cursor.
    RemainInPlace,
}

impl std::str::FromStr for CursorBehavior {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("move_to_start",   CursorBehavior::MoveToStart),
                    ("move_after_end",  CursorBehavior::MoveAfterEnd),
                    ("move_to_open",    CursorBehavior::MoveToOpen),
                    ("remain_in_place", CursorBehavior::RemainInPlace),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'move_to_start', 'move_after_end', \
                'move_to_open', or 'remain_in_place'")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Positioning
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for input or move positioning.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum Positioning {
    /// An explicit position.
    Position(Position),
    /// The position of the palette cursor.
    Cursor,
    /// The first open position.
    Open,
    /// No positioning.
    None,
}

impl Positioning {
    /// Returns true if the positioning is `None`.
    pub fn is_none(&self) -> bool {
        match self {
            Positioning::None => true,
            _                 => false,
        }
    }
}

impl std::str::FromStr for Positioning {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Ok(success) = any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("cursor", Positioning::Cursor),
                    ("open",   Positioning::Open),
                    ("none",   Positioning::None),
                ])
            (text)
            .end_of_text()
        {
            return Ok(success.value);
        }

        position(text)
            .end_of_text()
            .source_for("expected 'cursor', 'open', or a position")
            .finish()
            .map(Positioning::Position)
    }
}


////////////////////////////////////////////////////////////////////////////////
// HistorySetOption
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for setting palette history.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum HistorySetOption {
    /// Enables the palette history.
    Enable,
    /// Disables and clears the palette history.
    Disable,
    /// Clears the palette history.
    Clear,
}

impl std::str::FromStr for HistorySetOption {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("enable",  HistorySetOption::Enable),
                    ("disable", HistorySetOption::Disable),
                    ("clear",   HistorySetOption::Clear),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'enable', 'disable', or 'clear'.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// ListMode
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for list mode.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum ListMode {
    /// Display colors in a grid.
    Grid,
    /// Display one color per line.
    Lines,
    /// Display colors in a line.
    List,
}

impl std::str::FromStr for ListMode {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("grid",  ListMode::Grid),
                    ("lines", ListMode::Lines),
                    ("list",  ListMode::List),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'grid', 'lines', or 'list'.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// ColorStyle
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for color display style.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum ColorStyle {
    /// Do not display cell colors.
    None,
    /// Display cell colors with a color tile.
    Tile,
    /// Display cell colors with colored text.
    Text,
}

impl std::str::FromStr for ColorStyle {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("none", ColorStyle::None),
                    ("tile", ColorStyle::Tile),
                    ("text", ColorStyle::Text),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'none', 'tile', or 'text'.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// TextStyle
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for text display style.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum TextStyle {
    /// Do not display cell colors as text.
    None,
    /// Display cell colors using a 6-digit hex code.
    Hex6,
    /// Display cell colors using a 3-digit hex code.
    Hex3,
    /// Display cell colors using RGB notation.
    Rgb,
}

impl std::str::FromStr for TextStyle {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("none",  TextStyle::None),
                    ("hex_6", TextStyle::Hex6),
                    ("hex_3", TextStyle::Hex3),
                    ("hex",   TextStyle::Hex6),
                    ("rgb",   TextStyle::Rgb),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'none', 'hex', 'hex_6', 'hex_3', or 'rgb'.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// RuleStyle
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for the column rule display.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum RuleStyle {
    /// Do not display column rule.
    None,
    /// Display column rule with colors.
    Colored,
    /// Display column rule without colors.
    Plain,
}

impl std::str::FromStr for RuleStyle {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("none",  RuleStyle::None),
                    ("colored", RuleStyle::Colored),
                    ("plain", RuleStyle::Plain),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'none', 'colored', or 'plain'.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// LineStyle
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for the line display.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum LineStyle {
    /// Do not display line info.
    None,
    /// Display line info automatically detected settings.
    Auto,
    /// Display line info using the given width.
    Size(u16),
}

impl std::str::FromStr for LineStyle {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let size = uint::<u16>("u16")
            (text)
            .end_of_text();
        if size.is_ok() {
            return size.map_value(LineStyle::Size).finish();
        }

        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("none", LineStyle::None),
                    ("auto", LineStyle::Auto),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'none', 'auto', or integer value.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// GutterStyle
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for the gutter display.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum GutterStyle {
    /// Do not display gutter info.
    None,
    /// Display gutter info automatically detected settings.
    Auto,
    /// Display gutter info using the given width.
    Size(u16),
}

impl std::str::FromStr for GutterStyle {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let size = uint::<u16>("u16")
            (text)
            .end_of_text();
        if size.is_ok() {
            return size.map_value(GutterStyle::Size).finish();
        }

        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("none", GutterStyle::None),
                    ("auto", GutterStyle::Auto),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'none', 'auto', or integer value.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// ColorDisplay
////////////////////////////////////////////////////////////////////////////////
/// Combined color display settings.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct ColorDisplay {
    /// The ColorStyle.
    pub color_style: ColorStyle,
    /// The TextStyle.
    pub text_style: TextStyle,
}


impl ColorDisplay {
    /// Returns the total dedicated width of the color output, including
    /// whitespace.
    pub fn width(&self) -> u16 {
        let tile_width = match self.color_style {
            ColorStyle::None => 0,
            ColorStyle::Tile => 2,
            ColorStyle::Text => 0,
        };
        let text_width = match self.text_style {
            TextStyle::None => 0,
            TextStyle::Hex6 => 8,
            TextStyle::Hex3 => 5,
            TextStyle::Rgb  => 20,
        };
        tile_width + text_width
    }

    /// Prints a color using the color display mode.
    pub fn print(&self, color: Color) {
        match self.color_style {
            ColorStyle::Tile => {
                let [r, g, b] = color.rgb_octets();
                print!("{}", "  ".on_truecolor(r, g, b));
            },
            _ => (),
        }
        match self.text_style {
            TextStyle::Hex6 => {
                print!("{:X} ", color);
            },

            TextStyle::Hex3 => {
                let hex = color.rgb_hex();
                let r = (0xFF0000 & hex) >> 20;
                let g = (0x00FF00 & hex) >> 12;
                let b = (0x0000FF & hex) >> 4;
                print!("#{:01X}{:01X}{:01X} ", r, g, b);
            },

            TextStyle::Rgb  => {
                let [r, g, b] = color.rgb_ratios();
                print!("rgb({:0.2},{:0.2},{:0.2}) ", r, g, b);
            },

            _ => (),
        }
    }

    /// Prints an empty space using the color display mode.
    pub fn print_empty(&self) {
        match self.color_style {
            ColorStyle::Tile => {
                print!("{}", "▄▀"
                    .truecolor(0x33, 0x33, 0x33)
                    .on_truecolor(0x77, 0x77, 0x77));
            },
            _ => (),
        }

        match self.text_style {
            TextStyle::Hex6 => print!("        "),
            TextStyle::Hex3 => print!("     "),
            TextStyle::Rgb  => print!("                    "),
            _ => (),
        }
    }

    /// Prints an invalid color using the color display mode.
    pub fn print_invalid(&self) {
        match self.color_style {
            ColorStyle::Tile => {
                print!("{}", "??".truecolor(0x88, 0x88, 0x88));
            },
            _ => (),
        }

        match self.text_style {
            TextStyle::Hex6 => print!("{} ",
                    "???????".truecolor(0x88, 0x88, 0x88)),
            
            TextStyle::Hex3 => print!("{} ",
                    "????".truecolor(0x88, 0x88, 0x88)),
            
            TextStyle::Rgb  => print!("{} ",
                    "???????????????????".truecolor(0x88, 0x88, 0x88)),

            _ => (),
        }

    }
}
