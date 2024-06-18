//! Implementation of an allocated sorted vec.

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

use allocator_api2::alloc::Allocator;

use crate::_drop_guard::DropGuardResult;
use crate::_error::AllocResult;
use crate::_traits::DropIn;
use crate::vec::AllocatedVec;

pub fn is_sorted<T>(data: &[T]) -> bool
where
    T: Ord,
{
    data.windows(2).all(|w| w[0] <= w[1])
}

/// An allocated, sorted variant of a dynamic array. This can
/// be used to build more advanced data structures that
/// require ordered elements.
#[derive(Debug, PartialEq, Eq)]
pub struct AllocatedSortedVec<Index>
where
    Index: Ord,
{
    value: AllocatedVec<Index>,
}

impl<Index: Ord> Drop for AllocatedSortedVec<Index> {
    fn drop(&mut self) {
        panic!("AllocatedSortedVec should never be dropped.");
    }
}

impl<Index> AllocatedSortedVec<Index>
where
    Index: Ord,
{
    /// Create a new `AllocatedSortedVec`. Subsequent method calls MUST use
    /// the same `alloc` instance.
    #[inline]
    pub fn new_in<A: Allocator>(alloc: &A) -> DropGuardResult<Self, &A> {
        Ok(AllocatedVec::new_in(alloc)?.map(|value| AllocatedSortedVec { value }))
    }

    /// Create an `AllocatedSortedVec` from an `AllocatedVec`; `v` MUST
    /// be sorted. Subsequent method calls MUST use
    /// the same `alloc` instance.
    #[inline]
    pub fn from_vec(v: AllocatedVec<Index>) -> AllocatedSortedVec<Index> {
        assert!(is_sorted(&v));
        AllocatedSortedVec { value: v }
    }

    /// Create a new `AllocatedSortedVec` with a single entry `i`. Subsequent method calls MUST use
    /// the same `alloc` instance.
    #[inline]
    pub fn one_in<A: Allocator>(alloc: &A, i: Index) -> DropGuardResult<Self, &A> {
        let mut raw =
            AllocatedVec::try_with_capacity_in(alloc, 1)?.map(|value| AllocatedSortedVec { value });

        // Safety: `alloc` is the allocator used to create `raw`
        unsafe {
            raw.value.push_in(alloc, i)?;
        }

        Ok(raw)
    }

    /// Insert element `v` into the correct place in the vector. If memory needs
    /// to be allocated, `alloc` will be used.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this `AllocatedSortedVec`
    #[inline]
    pub unsafe fn insert_in<A: Allocator>(&mut self, alloc: &A, v: Index) -> AllocResult<bool> {
        match self.value.binary_search(&v) {
            Ok(_pos) => Ok(false),
            Err(pos) => {
                self.value.insert_in(alloc, pos, v)?;
                Ok(true)
            }
        }
    }

    /// Remove element `v` from the vector. No memory will be
    /// freed by this method.
    #[inline]
    pub fn remove(&mut self, v: &Index) -> bool {
        match self.value.binary_search(v) {
            Ok(pos) => {
                self.value.remove(pos);
                true
            }
            Err(_pos) => false,
        }
    }

    /// Returns `true` if the `SortedVec` contains an
    /// element of a given value.
    #[inline]
    pub fn contains(&self, v: &Index) -> bool {
        match self.value.binary_search(v) {
            Ok(_pos) => true,
            Err(_pos) => false,
        }
    }

    /// Return a reference to a given value if it
    /// contained in the sorted vec.
    #[inline]
    pub fn get(&self, v: &Index) -> Option<&Index> {
        match self.value.binary_search(v) {
            Ok(pos) => Some(&self.value[pos]),
            Err(_pos) => None,
        }
    }

    /// Removes the last element from a vector and returns it, or None if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Index> {
        self.value.pop()
    }
}

impl<T: std::cmp::Ord> Deref for AllocatedSortedVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.value
    }
}

impl<Index> DropIn for AllocatedSortedVec<Index>
where
    Index: Ord,
{
    #[inline]
    unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A) {
        self.value.drop_in(alloc)
    }
}

impl<Index: 'static + Ord + Hash + Eq + Copy> PartialEq<HashSet<Index>>
    for AllocatedSortedVec<Index>
{
    fn eq(&self, other: &HashSet<Index>) -> bool {
        self.len() == other.len() && self.iter().all(|i| other.contains(i))
    }
}
