
# Palette data

The Palette consists of an array of Cells, together with information needed to manipulate, order, and group those cells.

# Operations and their inverses

## InsertCell

A cell will be inserted at a free position in the array. This operation will never displace an existing cell, so it is reversable by the RemoveCell operation.

## RemoveCell

The cell at the given index will be removed. Expression-references to the removed cell will not be corrected, and cell meta-data will not be removed.


## SetCellName
## SetCellGroup
## SetCellPosition
## RemoveCellMetaData
## SetCellExpr
## ReassignCellExprReferences
