//! Dynamic array implementation using the _allocated_ pattern.
//!
//! This implementation is heavily based on
//! [The Rustonomicon][nomicon] book.
//!
//! [nomicon]: https://doc.rust-lang.org/nomicon

use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use allocator_api2::alloc::Allocator;
use allocator_api2::alloc::Global;

use crate::_drop_guard::DropGuard;
use crate::_error::{AllocResult, AllocResultExt};
use crate::_traits::DropIn;

use super::_allocated::{AllocatedVec, Drain};

#[derive(Debug)]
pub struct Vec<T, A: Allocator = Global> {
    alloc: A,
    raw: ManuallyDrop<AllocatedVec<T>>,
}

impl<T: PartialEq, A: Allocator> PartialEq<Self> for Vec<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T: PartialEq, A: Allocator> Eq for Vec<T, A> {}

impl<T, A: Allocator> Drop for Vec<T, A> {
    fn drop(&mut self) {
        // Safety: `self.raw` was allocated by `self.alloc`
        unsafe {
            self.raw.drop_in(&self.alloc);
        }
    }
}

impl<T> Vec<T> {
    #[inline]
    pub fn new() -> Self {
        let raw = AllocatedVec::new_in(&Global)
            .handle_alloc_error()
            .into_inner();

        Self { alloc: Global, raw }
    }
}

impl<T, A: Allocator + Clone> Vec<T, A> {
    #[inline]
    pub fn from_drop_guard(dg: DropGuard<AllocatedVec<T>, A>) -> Self {
        Self {
            alloc: dg.alloc().clone(),
            raw: dg.into_inner(),
        }
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A: Allocator> Deref for Vec<T, A> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.raw
    }
}

impl<T, A: Allocator> DerefMut for Vec<T, A> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.raw
    }
}

impl<T, A: Allocator> Vec<T, A> {
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    pub fn new_in(alloc: A) -> AllocResult<Self> {
        let raw = AllocatedVec::new_in(&alloc)?.into_inner();

        Ok(Vec { alloc, raw })
    }

    pub fn try_with_capacity_in(alloc: A, capacity: usize) -> AllocResult<Self> {
        let raw = AllocatedVec::try_with_capacity_in(&alloc, capacity)?.into_inner();

        Ok(Vec { alloc, raw })
    }

    /// Shrink capacity to match `len`.
    pub fn shrink_to_fit(&mut self) {
        // Safety: `alloc` was used to allocate `raw`
        unsafe { self.raw.shrink_to_fit_in(&self.alloc).handle_alloc_error() }
    }

    pub fn truncate(&mut self, len: usize) {
        self.raw.truncate(len)
    }

    /// Insert element `elem` at the end of the vector.
    pub fn push(&mut self, elem: T) {
        // Safety: `alloc` was used to allocate `raw`
        unsafe { self.raw.push_in(&self.alloc, elem).handle_alloc_error() }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    /// Insert element `elem` at `index`.
    pub fn insert(&mut self, index: usize, elem: T) {
        // Safety: `alloc` was used to allocate `raw`
        unsafe {
            self.raw
                .insert_in(&self.alloc, index, elem)
                .handle_alloc_error()
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `Vec<T>`. The collection may reserve more space.
    /// After calling reserve, capacity will be greater than or equal to
    /// `self.len() + additional`. Guaranteed to do nothing if capacity is
    /// already sufficient.
    pub fn try_reserve(&mut self, additional: usize) {
        // Safety: `alloc` was used to allocate `raw`
        unsafe {
            self.raw
                .try_reserve_in(&self.alloc, additional)
                .handle_alloc_error()
        }
    }

    /// Extend Vec using the contents of the iterator
    pub fn extend<K: IntoIterator<Item = T>>(&mut self, elems: K) {
        // Safety: `alloc` was used to allocate `raw`
        unsafe { self.raw.extend_in(&self.alloc, elems).handle_alloc_error() }
    }

    pub fn clear(&mut self) {
        self.raw.clear()
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.raw.remove(index)
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        self.raw.drain()
    }
}

pub trait SliceExt {
    type Item;
    fn to_allocated_vec(&self) -> Vec<Self::Item>;
}

impl<T: Clone> SliceExt for [T] {
    type Item = T;

    fn to_allocated_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        result.extend(self.iter().cloned());
        result
    }
}
