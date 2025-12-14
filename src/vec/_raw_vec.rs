//! Dynamic array implementation using the _allocated_ pattern.
//!
//! This implementation is heavily based on
//! [The Rustonomicon][nomicon] book.
//!
//! [nomicon]: https://doc.rust-lang.org/nomicon

use core::alloc::Layout;
use core::cmp;
use core::mem;
use core::ptr::NonNull;

use allocator_api2::alloc::Allocator;

use crate::_allocator_ext::AllocatorExt;
use crate::_drop_guard::{DropGuard, DropGuardResult};
use crate::_error::AllocResult;
use crate::_traits::DropIn;

pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

// Safety: `T` is Send, so an array of `T` is Send.
unsafe impl<T: Send> Send for RawVec<T> {}
// Safety: `T` is Sync, so an array of `T` is Sync.
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn new_in<A: Allocator>(alloc: A) -> DropGuardResult<Self, A> {
        let cap = if mem::size_of::<T>() == 0 { !0 } else { 0 };

        // Safety: `ptr` is dangling, so any allocator is valid.
        Ok(unsafe {
            DropGuard::new(
                RawVec {
                    ptr: NonNull::dangling(),
                    cap,
                },
                alloc,
            )
        })
    }

    pub fn try_with_capacity_in<A: Allocator>(
        alloc: &A,
        capacity: usize,
    ) -> DropGuardResult<Self, &A> {
        let (ptr, cap) = if mem::size_of::<T>() == 0 {
            (NonNull::dangling(), !0)
        } else {
            let ptr = alloc.allocate_array(capacity)?.into_inner();

            (ptr, capacity)
        };

        // Safety: `ptr` was just allocated using `alloc`.
        Ok(unsafe { DropGuard::new(RawVec { ptr, cap }, alloc) })
    }

    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this RawVec
    pub unsafe fn grow_in<A: Allocator>(&mut self, alloc: &A) -> AllocResult<()> {
        // since we set the capacity to usize::MAX when T has size 0,
        // getting to here necessarily means the Vec is overfull.
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        let new_ptr = if self.cap == 0 {
            alloc.allocate_array(new_cap)
        } else {
            // Safety: Requirements match function safety notes.
            unsafe { alloc.grow_array(self.ptr, self.cap, new_cap) }
        };

        self.ptr = new_ptr?.into_inner().cast();
        self.cap = new_cap;

        Ok(())
    }

    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this RawVec
    pub unsafe fn try_reserve_in<A: Allocator>(
        &mut self,
        alloc: &A,
        old_len: usize,
        additional: usize,
    ) -> AllocResult<()> {
        let new_len = old_len + additional;

        if new_len < self.cap {
            return Ok(());
        }

        let (new_cap, new_layout) = if self.cap == 0 {
            (new_len, Layout::array::<T>(new_len).unwrap())
        } else {
            // This can't overflow because we ensure self.cap <= isize::MAX.
            let new_cap = cmp::max(2 * self.cap, new_len);

            // `Layout::array` checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the `unwrap` should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            alloc.allocate(new_layout)
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            // Safety: Requirements match function safety notes.
            unsafe { alloc.grow(self.ptr.cast(), old_layout, new_layout) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match new_ptr {
            Ok(p) => p.cast(),
            Err(_) => {
                return Err(new_layout.into());
            }
        };
        self.cap = new_cap;

        Ok(())
    }

    /// # Safety
    ///
    /// `alloc` MUST be the allocator used to allocate this RawVec
    pub unsafe fn shrink_in<A: Allocator>(&mut self, alloc: &A, new_cap: usize) -> AllocResult<()> {
        assert!(new_cap <= self.cap);
        if new_cap == self.cap {
            return Ok(());
        }

        if new_cap == 0 {
            assert!(self.cap > 0);

            let elem_size = mem::size_of::<T>();

            if self.cap != 0 && elem_size != 0 {
                // Safety: ptr allocated by alloc with given capacity
                unsafe { alloc.deallocate_array(self.ptr, self.cap) };
            }

            self.ptr = NonNull::dangling();
            self.cap = 0;

            return Ok(());
        }

        // Safety: ptr was allocated using alloc
        self.ptr = unsafe {
            alloc
                .shrink_array(self.ptr, self.cap, new_cap)?
                .into_inner()
        };
        self.cap = new_cap;

        Ok(())
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        panic!("RawVec should never be dropped.");
    }
}

impl<T> DropIn for RawVec<T> {
    #[inline]
    unsafe fn drop_in<A: Allocator>(&mut self, alloc: &A) {
        let elem_size = mem::size_of::<T>();

        if self.cap != 0 && elem_size != 0 {
            // SAFETY: We know that `self.ptr` is non-null and `self.cap` is the capacity.
            unsafe {
                alloc.deallocate_array(self.ptr, self.cap);
            }
        }
    }
}

pub struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    pub fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.is_empty() {
                slice.as_ptr()
            } else {
                // SAFETY: slice guarantees alignment
                unsafe { slice.as_ptr().add(slice.len()) }
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else if mem::size_of::<T>() == 0 {
            self.start = (self.start as usize + 1) as *const _;
            // Safety: T is zero-sized; dangling is well aligned
            Some(unsafe { core::ptr::read(NonNull::<T>::dangling().as_ptr()) })
        } else {
            let old_ptr = self.start;
            // Safety: self.start < self.end; self.start is always valid
            self.start = unsafe { self.start.offset(1) };
            // Safety: old_ptr is always an aligned pointer to allocated memory
            unsafe { Some(core::ptr::read(old_ptr)) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len =
            (self.end as usize - self.start as usize) / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else if mem::size_of::<T>() == 0 {
            self.end = (self.end as usize - 1) as *const _;
            // Safety: T is zero-sized
            Some(unsafe { core::ptr::read(NonNull::<T>::dangling().as_ptr()) })
        } else {
            // Safety: Moving towards start
            self.end = unsafe { self.end.offset(-1) };
            // Safety: self.end >= self.start
            Some(unsafe { core::ptr::read(self.end) })
        }
    }
}

pub struct IntoIter<T> {
    _buf: RawVec<T>, // we don't actually care about this. Just need it to live.
    iter: RawValIter<T>,
}

impl<T> IntoIter<T> {
    pub(super) fn from_parts(iter: RawValIter<T>, _buf: RawVec<T>) -> Self {
        Self { iter, _buf }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}
