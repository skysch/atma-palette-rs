////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Undo history definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::palette::Operation;

// External library imports.
use serde::Serialize;
use serde::Deserialize;


////////////////////////////////////////////////////////////////////////////////
// CursorState
////////////////////////////////////////////////////////////////////////////////
/// The history cursor state. Used to validate that undo/redo operations are
/// completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorState {
    Valid,
    AwaitingSetRedo,
    AwaitingSetUndo,
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState::Valid
    }
}


////////////////////////////////////////////////////////////////////////////////
// History
////////////////////////////////////////////////////////////////////////////////
/// Structure for supporting undo/redo operations.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct History {
    /// The undo/redo list of history operations.
    ops: Vec<Vec<Operation>>,
    /// The cursor position, separating undo ops from redo ops.
    cursor: usize,
    /// The state of the cursor.
    #[serde(skip)]
    cursor_state: CursorState,
}



impl History {
    /// Constructs a new `History`.
    pub fn new() -> Self {
        History {
            ops: Vec::with_capacity(8),
            cursor: 0,
            cursor_state: CursorState::default(),
        }
    }

    /// Returns the number of undo operations currently available.
    pub fn undo_count(&self) -> usize {
        self.cursor
    }

    /// Returns the number of redo operations currently available.
    pub fn redo_count(&self) -> usize {
        self.ops.len() - self.cursor
    }

    /// Pushes a new set of undo operations onto the history at the current
    /// cursor position. This will truncate the history if there are any ops
    /// beyond the cursor.
    pub fn push_undo_ops(&mut self, ops: Vec<Operation>) {
        assert_eq!(self.cursor_state, CursorState::Valid);

        if self.cursor < ops.len() {
            self.ops.push(ops);
            self.cursor += 1;
        } else {
            self.ops[self.cursor] = ops;
            self.ops.truncate(self.cursor);
            self.cursor += 1;
        }
    }

    /// Performs a complete undo using the given operation transform function.
    ///
    /// This is equivalent to calling the given function on the result of
    /// `pop_undo_ops` and passing the result to `set_current_redo_ops`.
    ///
    /// ### Parameters
    /// + `f`: The operation transform function. This function is responsible
    ///   for receiving the undo operations and returning the redo operations
    ///   for them. The function will only be called if there are available
    ///   undo ops.
    pub fn undo_with<F>(&mut self, f: F) 
        where F: FnOnce(&[Operation]) -> Vec<Operation>
    {
        let mut redo_ops = None;
        if let Some(undo_ops) = self.pop_undo_ops() {
            redo_ops = Some((f)(undo_ops));
        }

        if let Some(redo_ops) = redo_ops {
            self.set_current_redo_ops(redo_ops);
        }
    }

    /// Performs a complete redo using the given operation transform function.
    ///
    /// This is equivalent to calling the given function on the result of
    /// `pop_redo_ops` and passing the result to `set_current_undo_ops`.
    ///
    /// ### Parameters
    /// + `f`: The operation transform function. This function is responsible
    ///   for receiving the redo operations and returning the undo operations
    ///   for them. The function will only be called if there are available
    ///   redo ops.
    pub fn redo_with<F>(&mut self, f: F) 
        where F: FnOnce(&[Operation]) -> Vec<Operation>
    {
        let mut undo_ops = None;
        if let Some(redo_ops) = self.pop_redo_ops() {
            undo_ops = Some((f)(redo_ops));
        }

        if let Some(undo_ops) = undo_ops {
            self.set_current_undo_ops(undo_ops);
        }
    }

    /// Retrieves the next set of undo operations and moves the cursor. This
    /// operation must be followed by `set_current_redo_ops` to ensure that the
    /// history remains in a valid state.
    pub fn pop_undo_ops(&mut self) -> Option<&[Operation]> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor_state = CursorState::AwaitingSetRedo;
        self.cursor -= 1;
        self.ops
            .get(self.cursor)
            .map(|ops| &ops[..])
    }

    /// Sets the redo operations for the last returned undo operation. Must
    /// only be called after `pop_undo_ops`.
    pub fn set_current_redo_ops(&mut self, redo: Vec<Operation>) {
        assert_eq!(self.cursor_state, CursorState::AwaitingSetRedo);

        self.ops[self.cursor + 1] = redo;
        self.cursor_state = CursorState::Valid;
    }

    /// Retrieves the next set of redo operations and moves the cursor. This
    /// operation must be followed by `set_current_undo_ops` to ensure that the
    /// history remains in a valid state.
    pub fn pop_redo_ops(&mut self) -> Option<&[Operation]> {
        if self.cursor == self.ops.len() {
            return None;
        }
        self.cursor_state = CursorState::AwaitingSetUndo;
        self.cursor += 1;
        self.ops
            .get(self.cursor)
            .map(|ops| &ops[..])
    }

    /// Sets the undo operations for the last returned redo operation. Must
    /// only be called after `pop_repo_ops`.
    pub fn set_current_undo_ops(&mut self, undo: Vec<Operation>) {
        assert_eq!(self.cursor_state, CursorState::AwaitingSetUndo);

        self.ops[self.cursor - 1] = undo;
        self.cursor_state = CursorState::Valid;
    }
}


impl Default for History {
    fn default() -> Self {
        History::new()
    }
}
