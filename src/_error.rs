use std::error::Error;
use std::fmt;

use allocator_api2::alloc::Layout;

/// An [`Error`] indicating allocation failure, with [`Layout`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AllocErrorWithLayout {
    layout: Layout,
}

impl AllocErrorWithLayout {
    /// Call the globally registered alloc error handler
    /// with the contained `Layout`.
    #[inline]
    pub fn handle_alloc_error(self) -> ! {
        std::alloc::handle_alloc_error(self.layout)
    }
}

impl From<Layout> for AllocErrorWithLayout {
    #[inline]
    fn from(layout: Layout) -> Self {
        AllocErrorWithLayout { layout }
    }
}

impl Error for AllocErrorWithLayout {}

impl fmt::Display for AllocErrorWithLayout {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("memory allocation failed")
    }
}

/// A [`Result`] representing an allocation.
///
/// See also: [`AllocResultExt`]
pub type AllocResult<T> = Result<T, AllocErrorWithLayout>;

/// An extension trait providing a convenient way to call
/// `handle_alloc_error` on a contained error, and otherwise
/// unwrap a result
///
/// See also: [`AllocResult`]
pub trait AllocResultExt {
    type Output;

    /// Unwrap an `AllocResult` by calling the globally
    /// registered alloc error handler if it is an error.
    fn handle_alloc_error(self) -> Self::Output;
}

impl<T> AllocResultExt for AllocResult<T> {
    type Output = T;

    #[inline]
    fn handle_alloc_error(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => e.handle_alloc_error(),
        }
    }
}
