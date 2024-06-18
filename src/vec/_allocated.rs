//! Dynamic array implementation using the _allocated_ pattern.
//!
//! This implementation is heavily based on
//! [The Rustonomicon][nomicon] book.
//!
//! [nomicon]: https://doc.rust-lang.org/nomicon

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

use allocator_api2::alloc::Allocator;

use crate::_drop_guard::DropGuardResult;
use crate::_error::AllocResult;
use crate::_traits::{DropIn, FromIteratorIn};

use super::_raw_vec::{IntoIter, RawValIter, RawVec};

/// An allocated variant of a dynamic array.
pub struct AllocatedVec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T: fmt::Debug> fmt::Debug for AllocatedVec<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}

impl<T: PartialEq> PartialEq for AllocatedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        self.iter().zip(other.iter()).all(|(x, y)| x == y)
    }
}

impl<T: Eq> Eq for AllocatedVec<T> {}

impl<T> AllocatedVec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr()
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn new_in<A: Allocator>(alloc: &A) -> DropGuardResult<Self, &A> {
        Ok(RawVec::new_in(alloc)?.map(|buf| AllocatedVec { buf, len: 0 }))
    }

    pub fn try_with_capacity_in<A: Allocator>(
        alloc: &A,
        capacity: usize,
    ) -> DropGuardResult<Self, &A> {
        Ok(RawVec::try_with_capacity_in(alloc, capacity)?.map(|buf| AllocatedVec { buf, len: 0 }))
    }

    /// Shrink capacity to match `len`.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this `AllocatedVec`
    pub unsafe fn shrink_to_fit_in<A: Allocator>(&mut self, alloc: &A) -> AllocResult<()> {
        self.buf.shrink_in(alloc, self.len)
    }

    /// Insert element `elem` at the end of the vector. If memory needs to be
    /// allocated, `alloc` will be used.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this `AllocatedSortedVec`
    pub unsafe fn push_in<A: Allocator>(&mut self, alloc: &A, elem: T) -> AllocResult<()> {
        if self.len == self.capacity() {
            self.buf.grow_in(alloc)?;
        }

        // Safety: We just checked that there's enough space allocated
        let ptr = unsafe { self.ptr().add(self.len) };
        // Safety: We just checked that there's enough space allocated
        unsafe { std::ptr::write(ptr, elem) };

        // Can't overflow, we'll OOM first.
        self.len += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            // Safety: self.len < self.capacity
            let ptr = unsafe { self.ptr().add(self.len) };
            // Safety: ptr is an aligned pointer to allocated memory
            unsafe { Some(std::ptr::read(ptr)) }
        }
    }

    /// Insert element `elem` at `index`. If memory needs
    /// to be allocated, `alloc` will be used.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this `AllocatedSortedVec`
    pub unsafe fn insert_in<A: Allocator>(
        &mut self,
        alloc: &A,
        index: usize,
        elem: T,
    ) -> AllocResult<()> {
        assert!(index <= self.len, "index out of bounds");
        if self.capacity() == self.len {
            self.buf.grow_in(alloc)?;
        }

        // Safety: index <= self.len < self.capacity
        let ptr1 = unsafe { self.ptr().add(index) };
        // Safety: index <= self.len < self.capacity
        let ptr2 = unsafe { self.ptr().add(index + 1) };

        // Safety: All writes will occur within our buffer.
        unsafe { std::ptr::copy(ptr1, ptr2, self.len - index) };
        // Safety: Write occurs within buffer.
        unsafe { std::ptr::write(ptr1, elem) };

        self.len += 1;

        Ok(())
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `AllocatedVec<T>`. The collection may reserve more space.
    /// After calling reserve, capacity will be greater than or equal to
    /// `self.len() + additional`. Guaranteed to do nothing if capacity is
    /// already sufficient.
    ///
    /// # Safety
    ///
    /// alloc MUST be the allocator used to allocate this AllocatedVec
    pub unsafe fn try_reserve_in<A: Allocator>(
        &mut self,
        alloc: &A,
        additional: usize,
    ) -> AllocResult<()> {
        self.buf.try_reserve_in(alloc, self.len, additional)
    }

    /// Extend AllocatedVec using the contents of the iterator
    ///
    /// # Safety
    ///
    /// alloc MUST be the allocator used to allocate this AllocatedVec
    pub unsafe fn extend_in<A: Allocator, K: IntoIterator<Item = T>>(
        &mut self,
        alloc: &A,
        elems: K,
    ) -> AllocResult<()> {
        let elems = elems.into_iter();
        let (_, upper_bound) = elems.size_hint();
        if let Some(upper_bound) = upper_bound {
            self.try_reserve_in(alloc, upper_bound)?;
        }

        for e in elems {
            self.push_in(alloc, e)?;
        }

        Ok(())
    }

    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }
        let remaining_len = self.len - len;
        // Safety: len < self.len; ptr points to a section of our allocated buffer
        let ptr = unsafe { self.as_mut_ptr().add(len) };
        let s = std::ptr::slice_from_raw_parts_mut(ptr, remaining_len);
        self.len = len;
        // Safety: s is a valid slice of instances of `T`
        unsafe { std::ptr::drop_in_place(s) };
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        self.len -= 1;

        // Safety: ptr1 will be a valid pointer within our buffer
        let ptr1 = unsafe { self.ptr().add(index) };
        // Safety: ptr2 will be a valid pointer within our buffer
        let ptr2 = unsafe { self.ptr().add(index + 1) };

        // Safety: ptr1 is a valid pointer within our buffer
        let result = unsafe { std::ptr::read(ptr1) };

        // Safety: All writes will occur within our buffer.
        unsafe { std::ptr::copy(ptr2, ptr1, self.len - index) };

        result
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        let iter = RawValIter::new(self);

        // this is a mem::forget safety thing. If Drain is forgotten, we just
        // leak the whole AllocatedVec's contents. Also we need to do this *eventually*
        // anyway, so why not do it now?
        self.len = 0;

        Drain {
            iter,
            vec: PhantomData,
        }
    }
}

impl<T> DropIn for AllocatedVec<T> {
    #[inline]
    unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A) {
        while self.pop().is_some() {}
        self.buf.drop_in(alloc)
    }
}

impl<T> Deref for AllocatedVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        // Safety: len <= capacity
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for AllocatedVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        // Safety: len <= capacity
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> IntoIterator for AllocatedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        let iter = RawValIter::new(&self);
        // Safety: `buf` is a valid pointer to an array of `T` elements
        let buf = unsafe { std::ptr::read(&self.buf) };
        mem::forget(self);

        IntoIter::from_parts(iter, buf)
    }
}

pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut AllocatedVec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        // pre-drain the iter
        for _ in &mut *self {}
    }
}

impl<'a, T, A: Allocator> FromIteratorIn<'a, T, A> for AllocatedVec<T> {
    #[inline]
    fn from_iter_in<I: IntoIterator<Item = T>>(
        alloc: &'a A,
        iter: I,
    ) -> DropGuardResult<AllocatedVec<T>, &'a A> {
        let mut v = AllocatedVec::new_in(alloc)?;

        // Safety: `alloc` is the allocator used to allocate `v`
        unsafe {
            v.extend_in(alloc, iter)?;
        }

        Ok(v)
    }
}
