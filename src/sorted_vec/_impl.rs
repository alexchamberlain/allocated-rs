//! Implementation of a sorted vec.

use allocator_api2::alloc::Global;
use core::fmt::Debug;
use core::mem::ManuallyDrop;
use core::ops::Deref;

use allocator_api2::alloc::Allocator;

use crate::_error::{AllocResult, AllocResultExt};
use crate::_traits::DropIn;

use super::_allocated::AllocatedSortedVec;

/// A sorted variant of a dynamic array
///
/// This demonstrates how to create a "normal" container
/// type out of an underlying allocated one.
#[derive(Debug)]
pub struct SortedVec<T, A: Allocator = Global>
where
    T: Ord,
{
    alloc: A,
    raw: ManuallyDrop<AllocatedSortedVec<T>>,
}

impl<T: Ord + PartialEq, A: Allocator> PartialEq<Self> for SortedVec<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T: Ord + PartialEq, A: Allocator> Eq for SortedVec<T, A> {}

impl<T: Ord, A: Allocator> Drop for SortedVec<T, A> {
    fn drop(&mut self) {
        // Safety: `alloc` is the allocator used to create `raw`
        unsafe {
            self.raw.drop_in(&self.alloc);
        }
    }
}

impl<T: Ord> SortedVec<T> {
    /// Create a new `SortedVec` using the `Global` allocator
    #[inline]
    pub fn new() -> Self {
        let raw = AllocatedSortedVec::new_in(&Global)
            .handle_alloc_error()
            .into_inner();

        Self { alloc: Global, raw }
    }
}

impl<T: Ord> Default for SortedVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord, A: Allocator> Deref for SortedVec<T, A> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.raw.deref()
    }
}

impl<Index, A: Allocator> SortedVec<Index, A>
where
    Index: Ord,
{
    /// Create a new `SortedVec` using `alloc`.
    #[inline]
    pub fn new_in(alloc: A) -> AllocResult<Self> {
        let raw = AllocatedSortedVec::new_in(&alloc)?.into_inner();
        Ok(Self { alloc, raw })
    }

    // /// Create an `AllocatedSortedVec` from an `AllocatedVec`; `v` MUST
    // /// be sorted.
    // #[inline]
    // pub fn from_vec(v: AllocatedVec<Index>) -> AllocatedSortedVec<Index> {
    //     assert!(is_sorted(&v));
    //     AllocatedSortedVec { value: v }
    // }

    /// Insert element `v` into the correct place in the vector.
    #[inline]
    pub fn insert(&mut self, v: Index) -> bool {
        // Safety: `alloc` is the allocator used to create `raw`
        unsafe { self.raw.insert_in(&self.alloc, v).unwrap() }
    }

    /// Remove element `v` from the vector. No memory will be
    /// freed by this method.
    #[inline]
    pub fn remove(&mut self, v: &Index) -> bool {
        self.raw.remove(v)
    }

    /// Returns `true` if the `SortedVec` contains an
    /// element of a given value.
    #[inline]
    pub fn contains(&self, v: &Index) -> bool {
        self.raw.contains(v)
    }

    /// Return a reference to a given value if it
    /// contained in the sorted vec.
    #[inline]
    pub fn get(&self, v: &Index) -> Option<&Index> {
        self.raw.get(v)
    }

    /// Removes the last element from a vector and returns it, or None if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Index> {
        self.raw.pop()
    }
}
