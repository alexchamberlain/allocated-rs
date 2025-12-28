#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use std::ptr::NonNull;

use allocator_api2::alloc::AllocError;
use allocator_api2::alloc::Allocator;
use allocator_api2::alloc::Global;
use allocator_api2::alloc::Layout;

use allocated::RawDropGuard;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

fn allocate_point_basic<A: Allocator>(
    alloc: &A,
    point: Point,
) -> Result<NonNull<Point>, AllocError> {
    let layout = Layout::new::<Point>();
    let ptr = alloc.allocate(layout)?;
    let ptr = ptr.cast::<Point>();
    // SAFETY: `ptr` is valid, as we've just allocated it with the correct layout.
    unsafe {
        ptr.write(point);
    }
    Ok(ptr)
}

/// # Safety
///
/// `ptr` must be _currently allocated_ via the provided `alloc`
unsafe fn deallocate_point_basic<A: Allocator>(alloc: &A, ptr: NonNull<Point>) {
    let layout = Layout::new::<Point>();
    // SAFETY: ptr is valid per function Safety, and layout is correct for `Point`.
    unsafe { alloc.deallocate(ptr.cast(), layout) }
}

fn allocate_from<A: Allocator, T>(alloc: &A, value: T) -> Result<RawDropGuard<T, &A>, AllocError> {
    let layout = Layout::new::<T>();
    let ptr = alloc.allocate(layout)?;
    let ptr = ptr.cast::<T>();
    // SAFETY: `ptr` is valid, as we've just allocated it with the correct layout.
    unsafe {
        ptr.write(value);
    }
    // SAFETY: `ptr` is valid and allocated by `alloc`.
    Ok(RawDropGuard::new(ptr, alloc, layout))
}

fn main() -> Result<(), AllocError> {
    let point = Point { x: 1.0, y: 2.0 };
    println!("{:?}", point);

    let alloc = Global;
    let ptr = allocate_point_basic(&alloc, point.clone())?;
    // SAFETY: We've just allocated this, and we know it isn't shared.
    println!("{:?}", unsafe { ptr.as_ref() });

    // SAFETY: We used the `alloc` to create this `ptr` just above.
    unsafe {
        deallocate_point_basic(&alloc, ptr);
    }

    let ptr = allocate_from(&alloc, point)?;
    std::mem::drop(ptr);

    Ok(())
}
