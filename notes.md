
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


# Command Operations
## atma list {page #|selection}
## atma insert {expr|ramp} {@cellref}
## atma delete {selection}
## atma move {selection} {@cellref}
## atma set {name|position|group} {selection}
## atma unset {name|position|group} {selection}
## atma undo {#}
## atma redo {#}
## atma import
## atma export



# Composite Operations
## Undo
## Redo

## InsertRamp
## InsertRange
## DeleteRange
## MoveRange
## SetRange
## FixRange
## SetParameters


# Primitive Operations
## InsertCell
## RemoveCell
## AssignName
## UnassignName
## ClearNames
## AssignPosition
## UnassignPosition
## ClearPosition
## AssignGroup
## UnassignGroup
## ClearGroup
## SetExpr


# Parser design principles
## Use `&'t str`, not `&mut &'t str`.
## Use `std::Result`.
## If a function takes extra args, return a parser.
## If a function takes no extra args, it is the parser.
## Use `FnMut`
## Use `std::error::Error` for failure source.
## Do not box/own all parse errors.
## Impl `PartialEq` on results for testing.
## Return value, token, rest on success.
## Return context, expected, source, and rest on failure.



