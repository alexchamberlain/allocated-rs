//! Extension trait for [`Allocator`] with convenience methods.
//!
//! This module provides [`AllocatorExt`], which adds generic helper methods
//! to any type implementing [`Allocator`].
//!
//! # Provided methods
//!
//! ## Single allocation
//! - [`AllocatorExt::allocate_from`]: Allocate and initialize a single `T`
//! - [`AllocatorExt::deallocate_from`]: Deallocate a single `T`
//!
//! ## Array allocation
//! - [`AllocatorExt::allocate_array`]: Allocate an array of `T` with given capacity
//! - [`AllocatorExt::deallocate_array`]: Deallocate an array
//! - [`AllocatorExt::grow_array`]: Grow an existing array
//! - [`AllocatorExt::shrink_array`]: Shrink an existing array
//!
//! # Example
//!
//! ```
//! use allocated::AllocatorExt;
//! use allocator_api2::alloc::Global;
//!
//! let alloc = Global;
//!
//! // Allocate a single value
//! let guard = alloc.allocate_from(42).unwrap();
//! drop(guard); // Automatically cleaned up
//!
//! // Allocate an array
//! let guard = alloc.allocate_array::<u32>(10).unwrap();
//! // ... use array ...
//! drop(guard); // Automatically cleaned up
//! ```
//!
//! # Returns
//!
//! All allocation methods return [`RawDropGuard`] to prevent leaks on error.

use allocator_api2::alloc::Allocator;
use std::alloc::Layout;
use std::ptr::NonNull;

use super::{AllocErrorWithLayout, RawDropGuard, RawDropGuardResult};

/// An extension trait for `Allocator`, which defines convenience generic methods.
pub trait AllocatorExt: Allocator + Sized {
    /// Allocate a single instance of `T` and then move `item` into it.
    ///
    /// # Feedback
    /// See also `deallocate_from`. I like that both methods end in `_from` for
    /// symmetry, but only the allocate side actually takes an instance of `T`.
    fn allocate_from<T>(&self, item: T) -> RawDropGuardResult<T, &Self>;

    /// Allocate an array of `T` of `capacity`.
    fn allocate_array<T>(&self, capacity: usize) -> RawDropGuardResult<T, &Self>;

    /// Grow an array of `T` elements from `old_capacity` to `new_capacity`.
    ///
    /// # Safety
    ///
    /// `old_ptr` MUST have been allocated by this allocator.
    unsafe fn grow_array<T>(
        &self,
        old_ptr: NonNull<T>,
        old_capacity: usize,
        new_capacity: usize,
    ) -> RawDropGuardResult<T, &Self>;

    /// Shrink an array of `T` elements from `old_capacity` to `new_capacity`.
    ///
    /// # Safety
    ///
    /// `old_ptr` MUST have been allocated by this allocator.
    unsafe fn shrink_array<T>(
        &self,
        old_ptr: NonNull<T>,
        old_capacity: usize,
        new_capacity: usize,
    ) -> RawDropGuardResult<T, &Self>;

    /// Deallocate a pointer to an instance of `T`.
    ///
    /// # Safety
    ///
    /// `ptr` MUST have been allocated by this allocator.
    ///
    /// # Feedback
    /// See also `allocate_from`. I like that both methods end in `_from` for
    /// symmetry, but only the allocate side actually takes an instance of `T`.
    unsafe fn deallocate_from<T>(&self, ptr: NonNull<T>);

    /// Deallocate an array of `capacity` `T` elements.
    ///
    /// # Safety
    ///
    /// `ptr` MUST have been allocated by this allocator, and must point to an array of `capacity` elements.
    unsafe fn deallocate_array<T>(&self, ptr: NonNull<T>, capacity: usize);
}

impl<A: Allocator> AllocatorExt for A {
    fn allocate_from<T>(&self, item: T) -> RawDropGuardResult<T, &Self> {
        let layout = Layout::new::<T>();
        let ptr = self
            .allocate(layout)
            .map_err(|_e| AllocErrorWithLayout::from(layout))?
            .cast();
        // Safety: `ptr` is valid as we've just allocated it
        unsafe {
            ptr.write(item);
        }
        Ok(RawDropGuard::new(ptr, self, layout))
    }

    fn allocate_array<T>(&self, capacity: usize) -> RawDropGuardResult<T, &Self> {
        let layout = Layout::array::<T>(capacity).unwrap();

        let ptr = self
            .allocate(layout)
            .map_err(|_e| AllocErrorWithLayout::from(layout))?
            .cast();

        Ok(RawDropGuard::new(ptr, self, layout))
    }

    unsafe fn grow_array<T>(
        &self,
        old_ptr: NonNull<T>,
        old_capacity: usize,
        new_capacity: usize,
    ) -> RawDropGuardResult<T, &Self> {
        let old_layout = Layout::array::<T>(old_capacity).unwrap();
        let new_layout = Layout::array::<T>(new_capacity).unwrap();

        let ptr = self
            .grow(old_ptr.cast(), old_layout, new_layout)
            .map_err(|_e| AllocErrorWithLayout::from(new_layout))?
            .cast();

        Ok(RawDropGuard::new(ptr, self, new_layout))
    }

    unsafe fn shrink_array<T>(
        &self,
        old_ptr: NonNull<T>,
        old_capacity: usize,
        new_capacity: usize,
    ) -> RawDropGuardResult<T, &Self> {
        let old_layout = Layout::array::<T>(old_capacity).unwrap();
        let new_layout = Layout::array::<T>(new_capacity).unwrap();

        let ptr = self
            .shrink(old_ptr.cast(), old_layout, new_layout)
            .map_err(|_e| AllocErrorWithLayout::from(new_layout))?
            .cast();

        Ok(RawDropGuard::new(ptr, self, new_layout))
    }

    unsafe fn deallocate_from<T>(&self, ptr: NonNull<T>) {
        let layout = Layout::new::<T>();
        self.deallocate(ptr.cast(), layout)
    }

    unsafe fn deallocate_array<T>(&self, ptr: NonNull<T>, capacity: usize) {
        let layout = Layout::array::<T>(capacity).unwrap();
        self.deallocate(ptr.cast(), layout)
    }
}
