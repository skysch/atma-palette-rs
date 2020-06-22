
# Palette data

The Palette consists of an array of Cells, together with information needed to manipulate, order, and group those cells.

# Cell references.

Cell in the palette are typically identified by resolving a `CellRef`. There are four variants of  `CellRef`:

+ `Index`, parsed as `@[uint]`. Refers to the cell by its index in the palette.

+ `Name`, parsed as `[name]`. Refers to the cell by an assigned name.

+ `Group`, parsed as `[name]@[uint]`. Refers to the cell by its index in an assigned group.

+ `Position`, parsed as `@[Pp][uint][Ll][uint]`. Refers to the cell by its assigned page and line numbers.

In the above notation, `[name]` consists of any sequence of non-`@` characters, while `[uint]` refers to a sequence of digits or `_` characters, with an optional base prefix (`0b`, `0o`, or `0x`.) Parsed names will have any whitespace stripped from the ends.


# Command Operations
## atma list
## atma insert
## atma delete
## atma move
## atma set
## atma undo
## atma redo
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


