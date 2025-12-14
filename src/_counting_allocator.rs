//! Utility for counting allocations made via an [Allocator]

use core::cell::RefCell;
use core::ops::Deref;
use core::ptr::NonNull;

use alloc::rc::Rc;

use allocator_api2::alloc::{AllocError, Allocator, Global, Layout};

/// Counting allocation
///
/// An [Allocator] useful for counting allocations
/// on an underlying allocator. Both number of
/// allocations and the number of bytes allocated
/// are tracked.
#[derive(Debug)]
pub struct CountingAllocator<A: Allocator> {
    allocator: A,
    counter: RefCell<Counter>,
}

#[derive(Debug)]
struct Counter {
    _n_allocs: usize,
    _n_deallocs: usize,

    _n_bytes_allocated: usize,
    _n_bytes_deallocated: usize,
}

impl<A: Allocator> CountingAllocator<A> {
    /// Wrap `allocator` to track the allocations.
    #[inline]
    pub fn new(allocator: A) -> Self {
        Self {
            allocator,
            counter: RefCell::new(Counter::new()),
        }
    }

    /// Number of allocations made so far
    #[inline]
    pub fn n_allocations(&self) -> usize {
        self.counter.borrow()._n_allocs
    }

    /// Number of deallocations made so far
    #[inline]
    pub fn n_deallocations(&self) -> usize {
        self.counter.borrow()._n_deallocs
    }

    /// Number of bytes allocated so far
    #[inline]
    pub fn n_bytes_allocated(&self) -> usize {
        self.counter.borrow()._n_bytes_allocated
    }

    /// Number of bytes deallocated so far
    #[inline]
    pub fn n_bytes_deallocated(&self) -> usize {
        self.counter.borrow()._n_bytes_deallocated
    }

    /// Net number of allocations; ie the number of
    /// allocations that are left to be deallocated.
    #[inline]
    pub fn net_allocations(&self) -> usize {
        let counter = self.counter.borrow();
        counter._n_allocs - counter._n_deallocs
    }

    /// Net number of bytes allocated; ie the number of
    /// bytes that are left to be deallocated.
    #[inline]
    pub fn net_bytes_allocated(&self) -> usize {
        let counter = self.counter.borrow();
        counter._n_bytes_allocated - counter._n_bytes_deallocated
    }
}

impl Default for CountingAllocator<Global> {
    fn default() -> Self {
        Self::new(Global)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    pub fn new() -> Self {
        Self {
            _n_allocs: 0,
            _n_deallocs: 0,
            _n_bytes_allocated: 0,
            _n_bytes_deallocated: 0,
        }
    }

    pub fn allocate(&mut self, layout: Layout) {
        self._n_allocs += 1;
        self._n_bytes_allocated += layout.size();
    }

    pub fn deallocate(&mut self, layout: Layout) {
        self._n_deallocs += 1;
        self._n_bytes_deallocated += layout.size();
    }
}

// SAFETY: Safety of `CountingAllocator<A>` relies on the Safety of `A`.
unsafe impl<A: Allocator> Allocator for CountingAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.counter.borrow_mut().allocate(layout);

        // println!("Allocated {:?}", ptr);
        self.allocator.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // println!("Deallocating {:?}", ptr);
        self.counter.borrow_mut().deallocate(layout);
        self.allocator.deallocate(ptr, layout)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SharedCountingAllocator<A: Allocator> {
    inner: Rc<CountingAllocator<A>>,
}

impl Default for SharedCountingAllocator<Global> {
    fn default() -> Self {
        Self {
            inner: Rc::new(CountingAllocator::default()),
        }
    }
}

impl<A: Allocator> Deref for SharedCountingAllocator<A> {
    type Target = CountingAllocator<A>;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

// SAFETY: Safety of `SharedCountingAllocator<A>` relies on the Safety of `A`.
unsafe impl<A: Allocator> Allocator for SharedCountingAllocator<A> {
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
    use crate::{AllocResult, AllocatorExt};

    use allocator_api2::vec::Vec;

    #[test]
    fn test_allocate() -> AllocResult<()> {
        let alloc = CountingAllocator::default();

        let ptr = alloc.allocate_from(42u8)?;
        drop(ptr);

        assert_eq!(alloc.net_allocations(), 0);
        assert_eq!(alloc.net_bytes_allocated(), 0);
        assert_eq!(alloc.n_allocations(), 1);
        assert_eq!(alloc.n_deallocations(), 1);
        assert_eq!(alloc.n_bytes_allocated(), 1);
        assert_eq!(alloc.n_bytes_deallocated(), 1);

        Ok(())
    }

    #[test]
    fn test_vector() {
        let alloc = CountingAllocator::default();
        let _v: Vec<u64, _> = Vec::with_capacity_in(100, &alloc);

        assert_eq!(alloc.net_allocations(), 1);
        assert_eq!(alloc.net_bytes_allocated(), 800);
        assert_eq!(alloc.n_allocations(), 1);
        assert_eq!(alloc.n_deallocations(), 0);
        assert_eq!(alloc.n_bytes_allocated(), 800);
        assert_eq!(alloc.n_bytes_deallocated(), 0);
    }
}
