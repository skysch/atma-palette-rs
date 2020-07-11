////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! A bijective map.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use serde::de::MapAccess;
use serde::de::Visitor;

use serde::Deserializer;
use serde::Serializer;

// Standard library imports.
use std::collections::BTreeMap;
use std::collections::btree_map;
use std::rc::Rc;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::iter::FusedIterator;
use std::iter::FromIterator;

////////////////////////////////////////////////////////////////////////////////
// BiMap
////////////////////////////////////////////////////////////////////////////////
pub struct BiMap<L, R> {
    forward: BTreeMap<Rc<L>, Rc<R>>,
    reverse: BTreeMap<Rc<R>, Rc<L>>,
}

impl<L, R> BiMap<L, R> {
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }
}

impl<L, R> BiMap<L, R> where L: Ord, R: Ord {
    /// Constructs a new empty BiMap.
    pub fn new() -> Self {
        BiMap {
            forward: BTreeMap::new(),
            reverse: BTreeMap::new(),
        }
    }

    /// Clears the map, removing all elements.
    pub fn clear(&mut self) {
        self.forward.clear();
        self.reverse.clear();
    }

    /// Returns a reference to the value associated with the given left key.
    ///
    /// The key may be any borrowed form of the map's left key type, but the
    /// ordering on the borrowed form must match the ordering on the key type.
    pub fn get_by_left<Q>(&self, left: &Q) -> Option<&R> 
        where
            Rc<L>: Borrow<Q>,
            Q: Ord + ?Sized, 
    {
        self.forward
            .get(left)
            .map(AsRef::as_ref)
    }

    /// Returns a reference to the value associated with the given right key.
    ///
    /// The key may be any borrowed form of the map's right key type, but the
    /// ordering on the borrowed form must match the ordering on the key type.
    pub fn get_by_right<Q>(&self, right: &Q) -> Option<&L> 
        where
            Rc<R>: Borrow<Q>,
            Q: Ord + ?Sized,
    {
        self.reverse
            .get(right)
            .map(AsRef::as_ref)
    }

    /// Returns true if the BiMap contains the given left key.
    ///
    /// The key may be any borrowed form of the map's left key type, but the
    /// ordering on the borrowed form must match the ordering on the key type.
    pub fn contains_left<Q>(&self, left: &Q) -> bool
        where
            Rc<L>: Borrow<Q>,
            Q: Ord + ?Sized, 
    {
        self.forward.contains_key(left)
    }

    /// Returns true if the BiMap contains the given right key.
    ///
    /// The key may be any borrowed form of the map's right key type, but the
    /// ordering on the borrowed form must match the ordering on the key type.
    pub fn contains_right<Q>(&self, right: &Q) -> bool
        where
            Rc<R>: Borrow<Q>,
            Q: Ord + ?Sized, 
    {
        self.reverse.contains_key(right)
    }

    pub fn insert(&mut self, left: L, right: R) -> Overwritten<L, R> {
        use Overwritten::*;
        let overwritten = match (
            self.remove_by_left(&left),
            self.remove_by_right(&right)) 
        {
            (None, None)         => Neither,
            (None, Some(r_pair)) => Right(r_pair.0, r_pair.1),
            (Some(l_pair), None) => if l_pair.1 == right {
                Pair(l_pair.0, l_pair.1)
            } else {
                Left(l_pair.0, l_pair.1)
            },
            (Some(l_pair), Some(r_pair)) => Both(l_pair, r_pair),
        };
        self.insert_unchecked(left, right);
        overwritten
    }

    /// Inserts the given left-right pair into the bimap without overwriting any
    /// existing values.
    ///
    /// Returns `Ok(())` if the pair was successfully inserted into the bimap.
    /// If either value exists in the map, `Err((left, right)` is returned with
    /// the attempted left-right pair and the map is unchanged.
    pub fn insert_no_overwrite(&mut self, left: L, right: R)
        -> Result<(), (L, R)>
    {
        if self.contains_left(&left) || self.contains_right(&right) {
            Err((left, right))
        } else {
            self.insert_unchecked(left, right);
            Ok(())
        }
    }


    /// Inserts the given left-right pair into the bimap without checking if the
    /// pair already exists.
    fn insert_unchecked(&mut self, left: L, right: R) {
        let left_rc = Rc::new(left);
        let right_rc = Rc::new(right);
        let _ = self.forward.insert(left_rc.clone(), right_rc.clone());
        let _ = self.reverse.insert(right_rc, left_rc);
    }

    /// Removes the left-right pair corresponding to the given left key.
    ///
    /// Returns the previous left-right pair if the map contained the left key
    /// and `None` otherwise.
    ///
    /// The key may be any borrowed form of the map's left key type, but the
    /// ordering on the borrowed form must match the ordering on the key type.
    pub fn remove_by_left<Q>(&mut self, left: &Q) -> Option<(L, R)>
        where
            Rc<L>: Borrow<Q>,
            Q: Ord + ?Sized, 
    {
        self.forward.remove(left).map(|right_rc| {
            let left_rc = self.reverse.remove(&right_rc).unwrap();
            (
                Rc::try_unwrap(left_rc).ok().unwrap(),
                Rc::try_unwrap(right_rc).ok().unwrap(),
            )
        })
    }

    /// Removes the left-right pair corresponding to the given right key.
    ///
    /// Returns the previous left-right pair if the map contained the right key
    /// and `None` otherwise.
    ///
    /// The key may be any borrowed form of the map's right key type, but the
    /// ordering on the borrowed form must match the ordering on the key type.
    pub fn remove_by_right<Q>(&mut self, right: &Q) -> Option<(L, R)>
        where
            Rc<R>: Borrow<Q>,
            Q: Ord + ?Sized, 
    {
        self.reverse.remove(right).map(|left_rc| {
            let right_rc = self.forward.remove(&left_rc).unwrap();
            (
                Rc::try_unwrap(left_rc).ok().unwrap(),
                Rc::try_unwrap(right_rc).ok().unwrap(),
            )
        })
    }

    /// Returns an iterator over the left-right pairs in the bimap in ascending
    /// order by left value.
    pub fn iter(&self) -> Iter<'_, L, R> {
        Iter {
            inner: self.forward.iter(),
        }
    }

    /// Returns an iterator over the left values in the bimap in ascending
    /// order.
    pub fn left_values(&self) -> LeftValues<'_, L, R> {
        LeftValues {
            inner: self.forward.iter(),
        }
    }

    /// Returns an iterator over the right values in the bimap in ascending
    /// order.
    pub fn right_values(&self) -> RightValues<'_, L, R> {
        RightValues {
            inner: self.reverse.iter(),
        }
    }

    /// Returns an iterator over the left-right pairs within the range keyed by
    /// the left in left-ascending order.
    pub fn left_range<'a, Q, A>(&'a self, range: A) -> LeftRange<'a, L, R>
    where
        Rc<L>: Borrow<Q>,
        A: std::ops::RangeBounds<Q>,
        Q: Ord + ?Sized,
    {
        LeftRange {
            inner: self.forward.range(range),
        }
    }

    /// Returns an iterator over the left-right pairs within the range keyed by
    /// the right in right-ascending order.
    pub fn right_range<'a, Q, A>(&'a self, range: A) -> RightRange<'a, L, R>
    where
        Rc<R>: Borrow<Q>,
        A: std::ops::RangeBounds<Q>,
        Q: Ord + ?Sized,
    {
        RightRange {
            inner: self.reverse.range(range),
        }
    }
}

impl<L, R> Clone for BiMap<L, R>
    where
        L: Clone + Ord,
        R: Clone + Ord,
{
    fn clone(&self) -> BiMap<L, R> {
        self.iter().map(|(l, r)| (l.clone(), r.clone())).collect()
    }
}

impl<L, R> Debug for BiMap<L, R>
    where
        L: Debug + Ord,
        R: Debug + Ord,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, (left, right)) in self.forward.iter().enumerate() {
            let comma = if i == 0 { "" } else { ", " };
            write!(f, "{}{:?} <> {:?}", comma, left, right)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<L, R> Default for BiMap<L, R>
    where
        L: Ord,
        R: Ord,
{
    fn default() -> BiMap<L, R> {
        BiMap::new()
    }
}

impl<L, R> FromIterator<(L, R)> for BiMap<L, R>
where
    L: Ord,
    R: Ord,
{
    fn from_iter<I>(iter: I) -> BiMap<L, R>
    where
        I: IntoIterator<Item = (L, R)>,
    {
        let mut bimap = BiMap::new();
        for (left, right) in iter {
            let _ = bimap.insert(left, right);
        }
        bimap
    }
}

impl<'a, L, R> IntoIterator for &'a BiMap<L, R>
where
    L: Ord,
    R: Ord,
{
    type Item = (&'a L, &'a R);
    type IntoIter = Iter<'a, L, R>;

    fn into_iter(self) -> Iter<'a, L, R> {
        self.iter()
    }
}

impl<L, R> IntoIterator for BiMap<L, R>
where
    L: Ord,
    R: Ord,
{
    type Item = (L, R);
    type IntoIter = IntoIter<L, R>;

    fn into_iter(self) -> IntoIter<L, R> {
        IntoIter {
            inner: self.forward.into_iter(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// IntoIter
////////////////////////////////////////////////////////////////////////////////
/// An owning iterator over the left-right pairs in a `BiMap`.
#[derive(Debug)]
pub struct IntoIter<L, R> {
    inner: btree_map::IntoIter<Rc<L>, Rc<R>>,
}

impl<L, R> Iterator for IntoIter<L, R> {
    type Item = (L, R);

    fn next(&mut self) -> Option<Self::Item> {
        // unwraps are safe because right2left is gone
        self.inner.next().map(|(l, r)| {
            (
                Rc::try_unwrap(l).ok().unwrap(),
                Rc::try_unwrap(r).ok().unwrap(),
            )
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<L, R> DoubleEndedIterator for IntoIter<L, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // unwraps are safe because right2left is gone
        self.inner.next_back().map(|(l, r)| {
            (
                Rc::try_unwrap(l).ok().unwrap(),
                Rc::try_unwrap(r).ok().unwrap(),
            )
        })
    }
}

impl<L, R> ExactSizeIterator for IntoIter<L, R> {}

impl<L, R> FusedIterator for IntoIter<L, R> {}

////////////////////////////////////////////////////////////////////////////////
// Iter
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the left-right pairs in a `BiMap`.
///
/// This struct is created by the [`iter`] method of `BiMap`.
///
/// [`iter`]: BiMap::iter
#[derive(Debug)]
pub struct Iter<'a, L, R> {
    inner: btree_map::Iter<'a, Rc<L>, Rc<R>>,
}

impl<'a, L, R> Iterator for Iter<'a, L, R> {
    type Item = (&'a L, &'a R);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(l, r)| (&**l, &**r))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for Iter<'a, L, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(l, r)| (&**l, &**r))
    }
}

impl<'a, L, R> ExactSizeIterator for Iter<'a, L, R> {}

impl<'a, L, R> FusedIterator for Iter<'a, L, R> {}


////////////////////////////////////////////////////////////////////////////////
// LeftValues
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the left values in a `BiMap`.
///
/// This struct is created by the [`left_values`] method of `BiMap`.
///
/// [`left_values`]: BiMap::left_values
#[derive(Debug)]
pub struct LeftValues<'a, L, R> {
    inner: btree_map::Iter<'a, Rc<L>, Rc<R>>,
}

impl<'a, L, R> Iterator for LeftValues<'a, L, R> {
    type Item = &'a L;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(l, _)| &**l)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for LeftValues<'a, L, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(l, _)| &**l)
    }
}

impl<'a, L, R> ExactSizeIterator for LeftValues<'a, L, R> {}

impl<'a, L, R> FusedIterator for LeftValues<'a, L, R> {}


////////////////////////////////////////////////////////////////////////////////
// RightValues
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the right values in a `BiMap`.
///
/// This struct is created by the [`right_values`] method of `BiMap`.
///
/// [`right_values`]: BiMap::right_values
#[derive(Debug)]
pub struct RightValues<'a, L, R> {
    inner: btree_map::Iter<'a, Rc<R>, Rc<L>>,
}

impl<'a, L, R> Iterator for RightValues<'a, L, R> {
    type Item = &'a R;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(r, _)| &**r)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for RightValues<'a, L, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(r, _)| &**r)
    }
}

impl<'a, L, R> ExactSizeIterator for RightValues<'a, L, R> {}

impl<'a, L, R> FusedIterator for RightValues<'a, L, R> {}

////////////////////////////////////////////////////////////////////////////////
// LeftRange
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct LeftRange<'a, L, R> {
    inner: btree_map::Range<'a, Rc<L>, Rc<R>>,
}

impl<'a, L, R> Iterator for LeftRange<'a, L, R> {
    type Item = (&'a L, &'a R);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(l, r)| (&**l, &**r))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for LeftRange<'a, L, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(l, r)| (&**l, &**r))
    }
}

impl<'a, L, R> ExactSizeIterator for LeftRange<'a, L, R> {}

impl<'a, L, R> FusedIterator for LeftRange<'a, L, R> {}

////////////////////////////////////////////////////////////////////////////////
// RightRange
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct RightRange<'a, L, R> {
    inner: btree_map::Range<'a, Rc<R>, Rc<L>>,
}

impl<'a, L, R> Iterator for RightRange<'a, L, R> {
    type Item = (&'a L, &'a R);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(r, l)| (&**l, &**r))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, L, R> DoubleEndedIterator for RightRange<'a, L, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(r, l)| (&**l, &**r))
    }
}

impl<'a, L, R> ExactSizeIterator for RightRange<'a, L, R> {}

impl<'a, L, R> FusedIterator for RightRange<'a, L, R> {}


////////////////////////////////////////////////////////////////////////////////
// Serde support
////////////////////////////////////////////////////////////////////////////////
/// Serializer for `BiMap`.
impl<L, R> Serialize for BiMap<L, R>
    where
        L: Serialize + Ord,
        R: Serialize + Ord,
{
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_map(self.iter())
    }
}

/// Visitor to construct `BiMap` from serialized map entries.
struct BiMapVisitor<L, R> {
    marker: std::marker::PhantomData<BiMap<L, R>>,
}

impl<'de, L, R> Visitor<'de> for BiMapVisitor<L, R>
    where
        L: Deserialize<'de> + Ord,
        R: Deserialize<'de> + Ord,
{
    type Value = BiMap<L, R>;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a map")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut entries: A)
        -> Result<Self::Value, A::Error>
    {
        let mut map = BiMap::new();
        while let Some((l, r)) = entries.next_entry()? {
            let _ = map.insert(l, r);
        }
        Ok(map)
    }
}

/// Deserializer for `BiMap`.
impl<'de, L, R> Deserialize<'de> for BiMap<L, R>
    where
        L: Deserialize<'de> + Ord,
        R: Deserialize<'de> + Ord,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_map(BiMapVisitor {
            marker: Default::default(),
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// Overwritten
////////////////////////////////////////////////////////////////////////////////
/// The previous left-right pairs, if any, that were overwritten by a call to
/// the [`insert`](BiHashMap::insert) method of a bimap.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Overwritten<L, R> {
    /// Neither the left nor the right value previously existed in the bimap.
    Neither,

    /// The left value existed in the bimap, and the previous left-right pair
    /// is returned.
    Left(L, R),

    /// The right value existed in the bimap, and the previous left-right pair
    /// is returned.
    Right(L, R),

    /// The left-right pair already existed in the bimap, and the previous
    /// left-right pair is returned.
    Pair(L, R),

    /// Both the left and the right value existed in the bimap, but as part of
    /// separate pairs. The first tuple is the left-right pair of the previous
    /// left value, and the second is the left-right pair of the previous right
    /// value.
    Both((L, R), (L, R)),
}
