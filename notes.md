

# Supported Command Syntax

    atma
        [--config [PATH]]
        [--settings [PATH]]
        [-p|--palette [PATH]]
        [-v|--verbose]
        [-q|--quiet|--silent]
        [--ztrace]

    atma new palette [PATH]
        [--from-script PATH]
        [--name NAME]
        [--no-history]
        [--set-active]
        [--overwrite]
    atma new config [PATH]
        [--overwrite]
    atma new settings [PATH]
        [--overwrite]

    atma list [SELECTION]
        [--mode [grid|lines|list]]
        [--color-style [tile|none|text]]
        [--text-style [none|hex6|hex3|rgb]]
        [--rule-style [colored|none|plain]]
        [--line-style [auto|none|[SIZE]]]
        [--gutter-style [auto|none|[SIZE]]]
        [--max-width WIDTH]
        [--max-columns COLUMNS]
        [--max-height HEIGHT]
        [--no-color]

    atma insert [INSERT_EXPR]
        [--name NAME]
        [--at POSITIONING]

    atma delete [SELECTION]

    atma move [SELECTION]
        [--to POSITIONING]

    atma set name POSITION_SELECTOR [name]
    atma set group CELL_REF [name] [--remove]
    atma set expr CELL_REF INSERT_EXPR
    atma set cursor [POSITION]
    atma set history [enable|disable|clear]
    atma set active-palette [PATH]
    atma set delete-cursor-behavior CURSOR_BEHAVIOR
    atma set insert-cursor-behavior CURSOR_BEHAVIOR
    atma set move-cursor-behavior CURSOR_BEHAVIOR

    atma undo [COUNT]
    atma redo [COUNT]

    atma export png [SELECTION]..
        [(-o|--output) PATH]


# Unimplemented Command Syntax

    atma export script [--parametric]
    atma import script [--parametric]
    atma import png
    
# Scripting

Scripting should only support editing commands: `insert`, `delete`, `move`, `set`. Settings modifications should only apply to the current script session, not to the save (unless otherwise specified.)

Each script command is ended with `;`. Whitespace is ignored otherwise.

# List options

    + Print names and groups? (Verbosity)
    + Indicate expr types?
    + Tile indicators for names, groups, positions?

# Palette constraints

    + Maximum column #
    + Maximum line #
    + Maximum page #
    + Name validation
    + Maximum number of positions


# Cell selection & Cell selector

## All
*
## Index
:0
:0-:1
## Position
:0.0.0
:0.0.0-:0.0.1
:*.0.0
:0.*.0
:0.0.*
:0.*.*
:*.0.*
:*.*.0
## Group
group:0-group:1
group:*
## Name
name


# Insertable objects
## Color
    + #ABCDEF
    + rgb(1.0,1.0,1.0)
    + cmyk(1.0,1.0,1.0,1.0)
    + hsl(360.0,1.0,1.0)
    + hsv(360.0,1.0,1.0)
    + xyz(1.0,1.0,1.0)

## Reference
    + CellRef

## Color Copy
    + (CellRef)
    + copy(CellRef)

## Blend Expr
    + set_red(CELL_REF, VALUE, [Interpolate])
    + set_green(CELL_REF, VALUE, [Interpolate])
    + set_blueCELL_REF, VALUE, [Interpolate])
    + lighten(CELL_REF, VALUE, [Interpolate])
    + darken(CELL_REF, VALUE, [Interpolate])
    + saturate(CELL_REF, VALUE, [Interpolate])
    + desaturate(CELL_REF, VALUE, [Interpolate])
    + hue_shift(CELL_REF, VALUE, [Interpolate])
    + set_hue(CELL_REF, VALUE, [Interpolate])

    + [rgb_]blend(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]multiply(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]divide(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]subtract(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]difference(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]screen(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]overlay(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]hard_light(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]soft_light(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]color_dodge(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]color_burn(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]linear_dodge(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]linear_burn(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]vivid_light(CELL_REF, CELL_REF, [Interpolate])
    + [rgb_]linear_light(CELL_REF, CELL_REF, [Interpolate])

### [Interpolate]
    f32
    linear([RGB], f32)
    cubic([RGB], f32, [f32,f32])

## Ramp Function
    ramp(count, blend_fn, [InterpolateRange])

### [InterpolateRange]
    linear
    linear([RGB], [(f32, f32)])
    cubic
    cubic([RGB], [(f32, f32)], [(f32, f32)])


# Palette data

The Palette consists of an array of Cells, together with information needed to manipulate, order, and group those cells.

# Cell references.

Cell in the palette are typically identified by resolving a `CellRef`. There are four variants of  `CellRef`:

+ `Index`, parsed as `:X`. Refers to the cell by its index in the palette.

+ `Name`, parsed as `[name]`. Refers to the cell by an assigned name.

+ `Group`, parsed as `[name]:[uint]`. Refers to the cell by its index in an assigned group.

+ `Position`, parsed as `:X.Y.Z`. Refers to the cell by its assigned page, line, and column numbers.

In the above notation, `[name]` consists of any sequence of characters excluding `:`,`,`,`-`,`.`,`*`, and whitespace, while `X`,`Y`, and `Z` refer to a sequence of digits or `_` characters, with an optional base prefix (`0b`, `0o`, or `0x`.)

# Resolving cell references: Assigned vs Occupied.

Names, positions, and groups are only meaningful if they've been assigned to an index. However, the index may or may not be associated with a cell in the palette. If they are, it is called an occupied index. Index references are always occupied if they are assigned, so there is no difference there. However, an index which is unassigned is still useful (to assign it), wheras names, positions, and groups are not useful if they are unassigned.


# Parser design principles
## Use `&'t str`, not `&mut &'t str`.

This makes it easier to back up in case of a failure. 

## Use `std::Result`.
## If a function takes extra args, return a parser.
## If a function takes no extra args, it is the parser.
## Use `FnMut`
## Use `std::error::Error` for failure source.
## Do not box/own all parse errors.
## Impl `PartialEq` on results for testing.
## Return value, token, rest on success.
## Return context, expected, source, and rest on failure.
## Separate lexer and parser.

This allows us to easily filter lexed tokens, i.e., to remove whitespace or comments. It also allows injecting tokens, i.e., to specify indentation levels, or to analyze comments using a separate parser stream. 

Without a dedicated lexer, all intermediate syntactical structure must be filtered or created inline, and it becomes difficult to separate and analyze.

    result
    span
    lexer
    combinator
        text
        token
    primitive
        comment
        float
        integer
        list
        string

# Overwrite behavior when inserting multiple cells.

## Error
Error if any cell is occupied.

## Skip
Move insertion past any occupied cells.

## Move
Move existing cells past any inserted cells, and fix any references.

## Overwrite
Overwrite any existing cells, preserving any existing structure.

## Remove
Remove any existing cells and anything referencing them.

# Behaviors when there is no room for a new cell.

## Error
## Stop
## WrapLine
## WrapPage
