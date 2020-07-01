////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common utility functions.
////////////////////////////////////////////////////////////////////////////////


/// Performs a set intersection of the ranges bound (inclusively) by the given
/// tuples.
pub(crate) fn split_intersect<T: Ord>(l: (T, T), r: (T, T)) -> Split<T> {
    // Check if range order is wrong.
    if l.0 > l.1 || r.0 > r.1 ||
        // Check if ranges are non-overlapping.
        l.1 < r.0 || l.0 > r.1
    {
        return Split::Zero;
    }

    let low  = if l.0 > r.0 { l.0 } else { r.0 };
    let high = if l.1 < r.1 { l.1 } else { r.1 };
    if low == high {
        Split::One(low)
    } else {
        Split::Two(low, high)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Split
////////////////////////////////////////////////////////////////////////////////
/// A type which may contain zero, one, or two of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Split<T> {
    /// No value present.
    Zero,
    /// One value present.
    One(T),
    /// Two values present.
    Two(T, T),
}

impl<T> Iterator for Split<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let mut res = None;
        replace_with(self, |curr|
            match curr {
                Split::Zero      => { res = None;    Split::Zero },
                Split::One(v)    => { res = Some(v); Split::Zero },
                Split::Two(a, b) => { res = Some(a); Split::One(b) },
            }
        );
        res
    }
}

impl<T> DoubleEndedIterator for Split<T> {
    fn next_back(&mut self) -> Option<T> {
        let mut res = None;
        replace_with(self, |curr|
            match curr {
                Split::Zero      => { res = None;    Split::Zero },
                Split::One(v)    => { res = Some(v); Split::Zero },
                Split::Two(a, b) => { res = Some(b); Split::One(a) },
            }
        );
        res
    }
}

impl<T> ExactSizeIterator for Split<T> {
    fn len(&self) -> usize {
        match self {
            Split::Zero      => 0,
            Split::One(_)    => 1,
            Split::Two(_, _) => 2,
        }
    }
}

impl<T> std::iter::FusedIterator for Split<T> {}


impl<T> Default for Split<T> {
    fn default() -> Self {
        Split::Zero
    }
}

impl<T> From<T> for Split<T> {
    fn from(value: T) -> Self {
        Split::One(value)
    }
}

impl<T> From<(T, T)> for Split<T> {
    fn from(value: (T, T)) -> Self {
        Split::Two(value.0, value.1)
    }
}

impl<T> From<Option<T>> for Split<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            None        => Split::Zero,
            Some(value) => Split::One(value),
        }
    }
}

impl<T> From<Option<(T, T)>> for Split<T> {
    fn from(value: Option<(T, T)>) -> Self {
        match value {
            None         => Split::Zero,
            Some((a, b)) => Split::Two(a, b),
        }
    }
}

impl<T> From<(Option<T>, Option<T>)> for Split<T> {
    fn from(value: (Option<T>, Option<T>)) -> Self {
        match (value.0, value.1) {
            (None,    None)    => Split::Zero,
            (Some(a), None)    => Split::One(a),
            (None,    Some(b)) => Split::One(b),
            (Some(a), Some(b)) => Split::Two(a, b),
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// replace_with
////////////////////////////////////////////////////////////////////////////////
// TODO: Replace this with std::mem::replace_with if it ever becomes 
// available.
/// Temporarily takes ownership of a value at a mutable location, and replace 
/// it with a new value based on the old one.
///
/// We move out of reference temporarily, to apply a closure, returning a new
/// value, which is then placed at the original value's location.
///
/// # An important note
///
/// The behavior on panic (or to be more precise, unwinding) is specified to
/// match the behavior of panicking inside a destructor, which itself is
/// simply specified to not unwind.
#[inline]
pub(crate) fn replace_with<T, F>(val: &mut T, replace: F)
    where F: FnOnce(T) -> T {
    // Guard against unwinding. Note that this is critical to safety, to avoid
    // the value behind the reference `val` is not dropped twice during
    // unwinding.
    let guard = ExitGuard;

    unsafe {
        // Take out the value behind the pointer.
        let old = std::ptr::read(val);
        // Run the closure.
        let new = replace(old);
        // Put the result back.
        std::ptr::write(val, new);
    }

    // Forget the guard, to avoid panicking.
    std::mem::forget(guard);
}

/// A guarding type which will abort upon drop.
///
/// This is used for catching unwinding and transforming it into abort.
///
/// The destructor should never be called naturally (use `std::mem::forget()`),
/// and only when unwinding.
struct ExitGuard;

impl Drop for ExitGuard {
    fn drop(&mut self) {
        // To avoid unwinding, we abort (we panic, which is equivalent to abort
        // inside an unwinding destructor) the program, which ensures that the
        // destructor of the invalidated value isn't runned, since this
        // destructor ought to be called only if unwinding happens.
        panic!("`replace_with` closure unwind");
    }
}
