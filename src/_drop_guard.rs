//! Convenience structs to avoid leaking memory in the face of errors.
//!
//! # Overview
//!
//! This module provides RAII guards that prevent memory leaks during fallible
//! operations by ensuring cleanup happens even if early returns occur.
//!
//! ## Types
//!
//! - [`DropGuard<T, A>`]: Guards allocated types implementing [`DropIn`]
//! - [`RawDropGuard<T, A>`]: Guards raw pointers with manual drop and dealloc
//!
//! # The problem
//!
//! When creating an allocated data structure through multiple fallible steps,
//! any early return could leak memory:
//!
//! ```rust,ignore
//! let keys = AllocatedVec::new_in(alloc)?;
//! let values = AllocatedVec::new_in(alloc)?; // If this fails, keys leaks!
//! ```
//!
//! # The solution
//!
//! `DropGuard` ensures cleanup even on error:
//!
//! ```rust,ignore
//! let keys = AllocatedVec::new_in(alloc)?;    // Returns DropGuard
//! let values = AllocatedVec::new_in(alloc)?;  // If this fails, keys is cleaned up
//!
//! // Success: extract inner values and transfer to final structure
//! let final_struct = MyStruct {
//!     keys: keys.into_inner(),
//!     values: values.into_inner(),
//! };
//! ```
//!
//! # Usage pattern
//!
//! All `*_in` constructor methods return `DropGuardResult<T, A>` which is
//! `AllocResult<DropGuard<T, A>>`. Use `into_inner()` to extract the value
//! when you're ready to transfer ownership to a permanent structure.

use std::alloc::Layout;
use std::mem;
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use allocator_api2::alloc::Allocator;

use super::_error::AllocResult;
use super::_traits::DropIn;

/// A temporary guard object that will `DropIn` the `value`
/// using the associated `alloc` on `Drop`.
#[derive(Debug)]
pub struct DropGuard<T: DropIn, A: Allocator> {
    value: ManuallyDrop<T>,
    alloc: A,
}

/// An `AllocResult` containing a `DropGuard`
pub type DropGuardResult<T, A> = AllocResult<DropGuard<T, A>>;

impl<T: DropIn, A: Allocator> DropGuard<T, A> {
    /// Create a `DropGuard` to avoid leaking `value` in
    /// the event of errors.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the correct [Allocator]
    /// instance to `drop_in` `value`.
    #[inline]
    pub unsafe fn new(value: T, alloc: A) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            alloc,
        }
    }

    /// Split the `DropGuard` into its constituent parts. The caller is taking
    /// responsibility for `drop_in` T in `A`.
    #[inline(always)]
    pub fn split(self) -> (ManuallyDrop<T>, A) {
        let x = MaybeUninit::new(self);

        // Deliberately shadow the value so we can't even try to drop it.
        let x = x.as_ptr();

        // SAFETY: Valid pointer as part of struct
        let value_ptr = unsafe { &(*x).value as *const ManuallyDrop<T> };
        // SAFETY: Valid pointer as part of struct
        let value = unsafe { std::ptr::read(value_ptr) };
        // SAFETY: Valid pointer as part of struct
        let alloc_ptr = unsafe { &(*x).alloc as *const A };
        // SAFETY: Valid pointer as part of struct
        let alloc = unsafe { std::ptr::read(alloc_ptr) };

        (value, alloc)
    }

    /// Apply a function `f` to the inner value. Note that the
    /// value will not be dropped by this function.
    #[inline]
    pub fn map<U: DropIn, F>(self, f: F) -> DropGuard<U, A>
    where
        F: FnOnce(T) -> U,
    {
        let (value, alloc) = self.split();
        let value: T = ManuallyDrop::into_inner(value);
        let value: U = f(value);
        DropGuard {
            value: ManuallyDrop::new(value),
            alloc,
        }
    }

    /// Unwrap the inner value. This is intended to transfer ownership into
    /// the permanent data structure.
    #[inline]
    pub fn into_inner(self) -> ManuallyDrop<T> {
        let (value, _alloc) = self.split();
        value
    }

    /// Unwrap the allocator. This is intended to transfer ownership into
    /// the permanent data structure.
    #[inline]
    pub fn alloc(&self) -> &A {
        &self.alloc
    }
}

impl<T: DropIn, A: Allocator> Deref for DropGuard<T, A> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: DropIn, A: Allocator> DerefMut for DropGuard<T, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: DropIn, A: Allocator> Drop for DropGuard<T, A> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self.value` will not be used again
        let mut value = unsafe { ManuallyDrop::take(&mut self.value) };
        // SAFETY: `value` was allocated using `self.alloc` per `new` Safety notice
        unsafe { value.drop_in(&self.alloc) }
    }
}

/// A temporary guard object that will `Drop` and deallocate the `value`
/// using the associated `alloc` on `Drop`.
pub struct RawDropGuard<T, A: Allocator> {
    value: NonNull<T>,
    alloc: A,
    layout: Layout,
}

/// An `AllocResult` containing a `RawDropGuard`
pub type RawDropGuardResult<T, A> = AllocResult<RawDropGuard<T, A>>;

impl<T, A: Allocator> RawDropGuard<T, A> {
    pub fn new(value: NonNull<T>, alloc: A, layout: Layout) -> Self {
        Self {
            value,
            alloc,
            layout,
        }
    }

    /// Unwrap the inner value. This is intended to transfer ownership into
    /// the permanent data structure.
    #[inline]
    pub fn into_inner(self) -> NonNull<T> {
        let value = self.value;
        mem::forget(self);
        value
    }
}

impl<T, A: Allocator> Drop for RawDropGuard<T, A> {
    #[inline]
    fn drop(&mut self) {
        // Safety: guarenteed by `new`
        unsafe {
            std::ptr::drop_in_place(self.value.as_ptr());
        }
        // Safety: guarenteed by `new`
        unsafe { self.alloc.deallocate(self.value.cast(), self.layout) }
    }
}
