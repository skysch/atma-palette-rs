////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Module for the `list` command.
////////////////////////////////////////////////////////////////////////////////


// Internal module imports.
use crate::palette::Palette;
use crate::command::ColorDisplay;
use crate::command::RuleStyle;
use crate::command::LineStyle;
use crate::command::GutterStyle;
use crate::command::ListMode;
use crate::setup::Config;
use crate::setup::Settings;
use crate::error::PaletteError;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::cell::PositionSelector;

// External module imports.
use colored::Colorize as _;



/// Executes the `atma list` command.
pub fn list(
    palette: &Palette,
    selection: Option<CellSelection<'_>>,
    mode: ListMode,
    size: (u16, u16),
    color_display: ColorDisplay,
    _rule_style: RuleStyle,
    _line_style: LineStyle,
    _gutter_style: GutterStyle,
    _no_color: bool,
    max_columns: u16,
    config: &Config,
    settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    match mode {
        ListMode::Grid => list_grid(
            palette,
            selection,
            size,
            Position::ZERO,
            color_display,
            max_columns,
            config,
            settings),
        ListMode::Lines => list_lines(
            palette,
            selection,
            color_display,
            config,
            settings),
        ListMode::List => unimplemented!(),
    }
}

////////////////////////////////////////////////////////////////////////////////
// list_lines
////////////////////////////////////////////////////////////////////////////////
/// Prints palette information.
fn list_lines<'a>(
    palette: &Palette,
    selection: Option<CellSelection<'a>>,
    color_display: ColorDisplay,
    _config: &Config,
    _settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    tracing::debug!("Start listing for selection {:?}", selection);
    let selection = selection.unwrap_or(CellSelector::All.into());
    let index_selection = selection.resolve(palette.inner());
    tracing::debug!("Start listing for {:?}", index_selection);

    for idx in index_selection {
        if let Ok(Some(c)) = palette.inner()
            .color(&CellRef::Index(idx))
        {
            print!("{:4X} ", idx);
            color_display.print(c);

            if let Some(pos) = palette.inner()
                .assigned_position(&CellRef::Index(idx))
            {
                print!(" {}", pos);
            }
            if let Some(name) = palette.inner()
                .assigned_name(&CellRef::Index(idx))
            {
                print!(" \"{}\"", name);
            }

            for group in palette.inner()
                .assigned_groups(&CellRef::Index(idx))?
            {
                print!(" \"{}\"", group);
            }
            
        } else {
            print!("{:4X} ", idx);
            color_display.print_invalid();

        }
        println!();
    }
    Ok(())
}


////////////////////////////////////////////////////////////////////////////////
// list_grid
////////////////////////////////////////////////////////////////////////////////
/// List palette cells in a grid.
fn list_grid<'a>(
    palette: &Palette,
    _selection: Option<CellSelection<'a>>,
    size: (u16, u16),
    corner_position: Position,
    color_display: ColorDisplay,
    max_columns: u16,
    _config: &Config,
    _settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    if max_columns == 0 { return Ok(()); }
    let print_line_numbers = true;
    let print_column_rule = true;
    let print_line_names = true;
    let left_gutter_width = if !print_line_numbers { 0 } else { 8 };
    let min_right_gutter_width = if !print_line_names { 0 } else { 25 };
    let max_center_width = size.0 - left_gutter_width - min_right_gutter_width;
    let columns: u16 = std::cmp::min(
        max_center_width / color_display.width(),
        max_columns);
    let right_gutter_width = if !print_line_names { 0 } else {
        std::cmp::max(
            size.0
                - ((columns + 1) * color_display.width())
                - left_gutter_width,
            min_right_gutter_width)
    };

    tracing::trace!("left_gutter_width: {}", left_gutter_width);
    tracing::trace!("max_center_width: {}", max_center_width);
    tracing::trace!("columns: {}", columns);
    tracing::trace!("right_gutter_width: {}", right_gutter_width);

    let max_col = corner_position.column.saturating_add(columns - 1);
    let mut max_line = corner_position.line.saturating_add(size.1 - 2);
    const MAX_SKIP: u16 = 20;
    tracing::trace!("max_col: {}, max_line: {}", max_col, max_line);

    let mut line_buf = Vec::with_capacity(columns.into());
    let mut skipped: u16 = 0;
    let mut line = corner_position.line;

    let page_color = colored::Color::TrueColor { r: 0x11, g: 0xDD, b: 0x22 };
    let rule_color = colored::Color::TrueColor { r: 0x77, g: 0x77, b: 0x77 };

    if print_column_rule {
        let page_selector = PositionSelector {
            page: Some(corner_position.page),
            line: None,
            column: None,
        };
        if print_line_numbers {
            print!("{:width$} ",
                format!("{}", page_selector).color(page_color),
                width=left_gutter_width as usize);
        }
        max_line -= 1;
        let w = color_display.width() as usize;
        for column in 0..=max_col {
            if column % 5 == 0 || color_display.width() > 4 { 
                print!("{:<width$}", column, width=w);
            } else {
                print!("{:<width$}", "⭣".color(rule_color), width=w);
            }
        }
        if print_line_names {
            if let Some(name) = palette.inner().get_name(&page_selector) {
                let w = right_gutter_width as usize - 1;
                print!(" {}", if name.len() > w {
                        &name[..w]
                    } else {
                        name
                    }.color(page_color));
            }
        }
        println!();
    }

    while line < max_line {
        let mut print_line = false;
        for column in corner_position.column..=max_col {
            match palette.inner().color(&CellRef::Position(Position {
                    page: corner_position.page,
                    line,
                    column,
                }))
            {
                Ok(Some(c)) => {
                    line_buf.push(Ok(Some(c)));
                    print_line = true;
                },
                Ok(None)    => line_buf.push(Ok(None)),
                Err(e)      => line_buf.push(Err(e)),
            }
        }

        if print_line {
            let line_selector = PositionSelector {
                page: Some(corner_position.page),
                line: Some(line),
                column: None,
            };
            if print_line_numbers {
                print!("{:width$} ", 
                    format!("{}", line_selector),
                    width=left_gutter_width as usize);
            }
            if skipped > 0 {
                println!("\t ... {} empty line{}.",
                    skipped,
                    if skipped == 1 { "" } else { "s" });
            }
            skipped = 0;
            match line.checked_add(1) {
                None => break,
                Some(new_line) => { line = new_line; },
            }

            for elem in line_buf.drain(..) {
                match elem {
                    Ok(Some(c)) => color_display.print(c),
                    Err(PaletteError::UndefinedColor { cell_ref, circular }) => {
                        color_display.print_invalid();
                        tracing::warn!("{:?} {:?}", cell_ref, circular);
                    },
                    Ok(None)    => color_display.print_empty(),
                    Err(_)      => color_display.print_empty(),
                }
            }

            if print_line_names {
                if let Some(name) = palette.inner().get_name(&line_selector) {
                    let w = right_gutter_width as usize - 1;
                    print!(" {}", if name.len() > w {
                            &name[..w]
                        } else {
                            name
                        });
                }
            }
            println!();
        } else {
            match skipped.checked_add(1) {
                None => break,
                Some(new_skipped) if new_skipped > MAX_SKIP => break,
                Some(new_skipped) => { skipped = new_skipped; },
            }
        }
    }
    if skipped > 0 { println!("\t..."); }
    Ok(())
}
