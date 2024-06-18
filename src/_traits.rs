//! Core traits for the _allocated_ pattern.
//!
//! This module defines traits that enable allocator-aware data structures:
//!
//! # Core traits
//!
//! - [`DropIn`]: Drop with allocator context (analogous to [`Drop`])
//! - [`RecursiveDropIn`]: Recursively drop elements and container
//! - [`FromIteratorIn`]: Create from iterator with allocator (analogous to [`FromIterator`])
//! - [`CollectIn`]: Collect into allocated container (analogous to [`Iterator::collect`])
//! - [`IntoIteratorIn`]: Convert to iterator with allocator
//!
//! # The `DropIn` trait
//!
//! The fundamental trait of this crate. Unlike [`Drop`], which doesn't know about
//! allocators, `DropIn` receives the allocator as a parameter:
//!
//! ```rust,ignore
//! pub trait DropIn {
//!     unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A);
//! }
//! ```
//!
//! This allows data structures to deallocate their memory without storing
//! the allocator themselves. The wrapper types ensure safety by guaranteeing
//! the same allocator is always used.
//!
//! # Iterator traits
//!
//! The `*In` iterator traits mirror std's iterator traits but are allocator-aware:
//!
//! ```rust,ignore
//! // Standard library:
//! let v: Vec<i32> = iter.collect();
//!
//! // With allocator:
//! let v: AllocatedVec<i32> = iter.collect_in(&alloc)?;
//! ```
//!
//! # Safety invariant
//!
//! **Critical:** Methods taking an allocator parameter are marked `unsafe` because
//! the caller must guarantee it's the same allocator instance used to allocate
//! the data structure. Using a different allocator is undefined behavior.

use allocator_api2::alloc::Allocator;

use super::_drop_guard::DropGuardResult;

/// Tag a structure as _allocated_; that is it needs to be dropped with the
/// context of an allocator. This is equivalent to [std::ops::Drop].
pub trait DropIn {
    /// Drop elements of this container, then deallocate
    /// any memory allocated.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate `self`.
    unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A);
}

impl<T> DropIn for Option<T>
where
    T: DropIn,
{
    unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A) {
        if let Some(t) = self {
            t.drop_in(alloc)
        }
    }
}

/// A trait for container types to drop elements, as well
/// as the container, using a provided allocator.
pub trait RecursiveDropIn: DropIn {
    /// `drop_in` elements of this container, then deallocate
    /// any memory allocated.
    ///
    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate `self`.
    unsafe fn recursive_drop_in<A: Allocator>(&mut self, alloc: &A);
}

/// Conversion from an [`Iterator`], and the equivalent of [std::iter::FromIterator].
///
/// By implementing `FromIteratorIn` for a type, you define how it will be
/// created from an iterator, which is common for types which describe a
/// collection of some kind.
///
/// If you want to create a collection from the contents of an iterator, the
/// [`CollectIn::collect_in()`] method is preferred. However, when you need to
/// specify the container type, [`FromIteratorIn::from_iter_in()`] can be more
/// readable than using a turbofish (e.g. `::<Vec<_>>()`).
///
/// See also: [`IntoIterator`].
pub trait FromIteratorIn<'a, U, A: Allocator>: Sized + DropIn {
    /// Create a container from elements of an iterator. Any memory that
    /// needs to be allocated will be allocated in `alloc`. The instance
    /// of `Self` MUST be `drop_in` using the same instance of the
    /// allocator.
    fn from_iter_in<T>(alloc: &'a A, iter: T) -> DropGuardResult<Self, &'a A>
    where
        T: IntoIterator<Item = U>;
}

/// Transforms an iterator into a collection. Similar to [std::iter::Iterator::collect].
///
/// [`CollectIn::collect_in()`] can take anything iterable, and turn it into a
/// relevant, allocated collection.
pub trait CollectIn<'a, U, A: Allocator>: Sized + IntoIterator<Item = U> {
    /// Collect elements of an iterator in a new container instance. Any memory that
    /// needs to be allocated will be allocated in `alloc`. The instance
    /// of the container MUST be `drop_in` using the same instance of the
    /// allocator.
    fn collect_in<B: FromIteratorIn<'a, U, A> + DropIn>(
        self,
        alloc: &'a A,
    ) -> DropGuardResult<B, &'a A> {
        FromIteratorIn::from_iter_in(alloc, self)
    }
}

impl<'a, U, A: Allocator, T: Sized + IntoIterator<Item = U>> CollectIn<'a, U, A> for T {}

/// Conversion into an iterator using a given `Allocator`.
///
/// This is useful for allocated data structures, and allows them to be used with various
/// iterator methods.
pub trait IntoIteratorIn<'a, A: Allocator> {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;

    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate `self`.
    unsafe fn into_iter_in(self, alloc: &'a A) -> Self::IntoIter;
}
