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
    type Err = InvalidCursorBehavior;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "move_to_start"   => Ok(CursorBehavior::MoveToStart),
            "move_after_end"  => Ok(CursorBehavior::MoveAfterEnd),
            "move_to_open"    => Ok(CursorBehavior::MoveToOpen),
            "remain_in_place" => Ok(CursorBehavior::RemainInPlace),
            _                 => Err(InvalidCursorBehavior),
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidCursorBehavior;

impl std::fmt::Display for InvalidCursorBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid cursor behavior: expected one of 'move_to_start', \
            'move_after_end', 'move_to_open', or 'remain_in_place'")
    }
}

impl std::error::Error for InvalidCursorBehavior {}


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
    type Err = InvalidPositioning;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "cursor" => Ok(Positioning::Cursor),
            "open"   => Ok(Positioning::Open),
            "none"   => Ok(Positioning::None),
            _        => match Position::from_str(text) {
                Ok(pos) => Ok(Positioning::Position(pos)),
                Err(_)  => Err(InvalidPositioning),
            }
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidPositioning;

impl std::fmt::Display for InvalidPositioning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid cursor positioning: expected one of 'cursor', \
            'open', 'none', or a position")
    }
}

impl std::error::Error for InvalidPositioning {}


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
    type Err = InvalidHistorySetOption;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "enable"  => Ok(HistorySetOption::Enable),
            "disable" => Ok(HistorySetOption::Disable),
            "clear"   => Ok(HistorySetOption::Clear),
            _         => Err(InvalidHistorySetOption),
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidHistorySetOption;

impl std::fmt::Display for InvalidHistorySetOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid history set option: expected one of 'enable', \
            'disable', or 'clear'")
    }
}

impl std::error::Error for InvalidHistorySetOption {}



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
    type Err = InvalidListMode;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "grid"  => Ok(ListMode::Grid),
            "lines" => Ok(ListMode::Lines),
            "list"  => Ok(ListMode::List),
            _       => Err(InvalidListMode),
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidListMode;

impl std::fmt::Display for InvalidListMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid list mode: expected one of 'grid', \
            'lines', or 'list'")
    }
}

impl std::error::Error for InvalidListMode {}


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
    type Err = InvalidColorStyle;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "none" => Ok(ColorStyle::None),
            "tile" => Ok(ColorStyle::Tile),
            "text" => Ok(ColorStyle::Text),
            _      => Err(InvalidColorStyle),
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidColorStyle;

impl std::fmt::Display for InvalidColorStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid color style: expected one of 'none', \
            'tile', or 'text'")
    }
}

impl std::error::Error for InvalidColorStyle {}


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
    type Err = InvalidTextStyle;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "none"  => Ok(TextStyle::None),
            "hex_6" => Ok(TextStyle::Hex6),
            "hex_3" => Ok(TextStyle::Hex3),
            "hex"   => Ok(TextStyle::Hex6),
            "rgb"   => Ok(TextStyle::Rgb),
            _       => Err(InvalidTextStyle),
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidTextStyle;

impl std::fmt::Display for InvalidTextStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid text style: expected one of 'none', \
            'hex_6', 'hex_3', 'hex', or 'rgb'")
    }
}

impl std::error::Error for InvalidTextStyle {}


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
    type Err = InvalidRuleStyle;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "none"    => Ok(RuleStyle::None),
            "colored" => Ok(RuleStyle::Colored),
            "plain"   => Ok(RuleStyle::Plain),
            _         => Err(InvalidRuleStyle),
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidRuleStyle;

impl std::fmt::Display for InvalidRuleStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid text style: expected one of 'none', \
            'colored', or 'plain'")
    }
}

impl std::error::Error for InvalidRuleStyle {}


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
    type Err = InvalidLineStyle;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "none" => Ok(LineStyle::None),
            "auto" => Ok(LineStyle::Auto),
            _      => match u16::from_str(text) {
                Ok(val) => Ok(LineStyle::Size(val)),
                Err(_)  => Err(InvalidLineStyle),
            }
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidLineStyle;

impl std::fmt::Display for InvalidLineStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid line style: expected one of 'none', \
            'auto', or an integer value")
    }
}

impl std::error::Error for InvalidLineStyle {}


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
    type Err = InvalidGutterStyle;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "none" => Ok(GutterStyle::None),
            "auto" => Ok(GutterStyle::Auto),
            _      => match u16::from_str(text) {
                Ok(val) => Ok(GutterStyle::Size(val)),
                Err(_)  => Err(InvalidGutterStyle),
            }
        }
    }
}

/// Error type for an invalid cursor behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvalidGutterStyle;

impl std::fmt::Display for InvalidGutterStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid gutter style: expected one of 'none', \
            'auto', or an integer value")
    }
}

impl std::error::Error for InvalidGutterStyle {}


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
            TextStyle::Hex6 => if let ColorStyle::Text = self.color_style {
                let [or, og, ob] = color.rgb_octets();
                print!("{} ",
                    format!("{:X}", color).truecolor(or, og, ob));
            } else {
                print!("{:X} ", color);
            },

            TextStyle::Hex3 => {
                let hex = color.rgb_hex();
                let r = (0xFF0000 & hex) >> 20;
                let g = (0x00FF00 & hex) >> 12;
                let b = (0x0000FF & hex) >> 4;
                if let ColorStyle::Text = self.color_style {
                    let [or, og, ob] = color.rgb_octets();
                    print!("{} ",
                        format!("#{:01X}{:01X}{:01X}", r, g, b)
                            .truecolor(or, og, ob));
                } else {
                    print!("#{:01X}{:01X}{:01X} ", r, g, b);
                }
            },

            TextStyle::Rgb  => if let ColorStyle::Text = self.color_style {
                let [or, og, ob] = color.rgb_octets();
                let [r, g, b] = color.rgb_ratios();
                print!("{} ", 
                    format!("rgb({:0.2},{:0.2},{:0.2})", r, g, b)
                        .truecolor(or, og, ob));

            } else {
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
