//! Utility for tracking allocations made via an [Allocator]

use std::backtrace::Backtrace;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;

use allocator_api2::alloc::{AllocError, Allocator, Global, Layout};

/// Implementation of an [Allocator], which tracks
/// active allocations.
#[derive(Debug)]
pub struct TrackingAllocator<A: Allocator> {
    allocator: A,
    tracker: RefCell<Tracker>,
}

#[derive(Debug)]
struct Tracker {
    outstanding: HashMap<NonNull<u8>, Backtrace>,
    zero_allocs: usize,
}

impl<A: Allocator> TrackingAllocator<A> {
    /// Wrap an `allocator` in a `TrackingAllocator`.
    #[inline]
    pub fn new(allocator: A) -> Self {
        Self {
            allocator,
            tracker: RefCell::new(Tracker::new()),
        }
    }

    /// Outstanding allocations; map of pointer to where it was allocated.
    #[inline]
    pub fn outstanding(&self) -> Ref<'_, HashMap<NonNull<u8>, Backtrace>> {
        Ref::map(self.tracker.borrow(), |v| &v.outstanding)
    }
}

impl Default for TrackingAllocator<Global> {
    fn default() -> Self {
        Self::new(Global)
    }
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            outstanding: Default::default(),
            zero_allocs: 0,
        }
    }

    pub fn record_allocate(&mut self, layout: &Layout, ptr: NonNull<u8>) {
        if layout.size() == 0 {
            self.zero_allocs += 1;
        } else {
            self.outstanding.insert(ptr, Backtrace::force_capture());
        }
    }

    pub fn record_deallocate(&mut self, layout: &Layout, ptr: NonNull<u8>) {
        if layout.size() == 0 {
            assert!(self.zero_allocs >= 1);
            self.zero_allocs -= 1;
        } else {
            let v = self.outstanding.remove(&ptr);
            assert!(v.is_some());
        }
    }
}

// Safety: guaranteed by `A`
unsafe impl<A: Allocator> Allocator for TrackingAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.allocator.allocate(layout);
        if let Ok(ptr) = ptr {
            self.tracker
                .borrow_mut()
                .record_allocate(&layout, as_non_null_ptr(ptr));
        }
        ptr
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // println!("Deallocating {:?}", ptr);
        self.tracker.borrow_mut().record_deallocate(&layout, ptr);
        self.allocator.deallocate(ptr, layout)
    }
}

#[inline]
fn as_non_null_ptr<T>(n: NonNull<[T]>) -> NonNull<T> {
    // SAFETY: We know `self` is non-null.
    unsafe { NonNull::new_unchecked(n.as_ptr() as *mut T) }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SharedTrackingAllocator<A: Allocator> {
    inner: Rc<TrackingAllocator<A>>,
}

impl Default for SharedTrackingAllocator<Global> {
    fn default() -> Self {
        Self {
            inner: Rc::new(TrackingAllocator::default()),
        }
    }
}

impl<A: Allocator> Deref for SharedTrackingAllocator<A> {
    type Target = TrackingAllocator<A>;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

// Safety: guaranteed by `A`
unsafe impl<A: Allocator> Allocator for SharedTrackingAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.inner.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.inner.deallocate(ptr, layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AllocResult;
    use crate::_allocator_ext::AllocatorExt;

    use allocator_api2::vec::Vec;

    #[test]
    fn test_allocate() -> AllocResult<()> {
        let alloc = TrackingAllocator::default();

        let ptr = alloc.allocate_from(42u8)?;
        drop(ptr);

        assert!(alloc.outstanding().is_empty());

        Ok(())
    }

    #[test]
    fn test_vector() {
        let alloc = TrackingAllocator::default();
        let _v: Vec<u64, _> = Vec::with_capacity_in(100, &alloc);

        assert_eq!(alloc.outstanding().len(), 1);
    }
}
