use std::alloc::{Allocator, Global};
use std::pin::Pin;
use std::ptr::NonNull;

use crate::alloc::{Allocation, Data};
use crate::trace::Trace;

pub struct GcPtr<T: ?Sized, A: Allocator = Global> {
    inner: NonNull<Allocation<T, A>>,
}

impl<T: Trace> GcPtr<T> {
    pub(crate) fn new(data: T) -> GcPtr<T> {
        GcPtr {
            inner: Allocation::new(data),
        }
    }
}

impl<T: Trace, A: Allocator> GcPtr<T, A> {
    pub(crate) fn new_in(data: T, allocator: A) -> GcPtr<T, A> {
        GcPtr {
            inner: Allocation::new_in(data, allocator),
        }
    }
}

impl<T: ?Sized, A: Allocator> GcPtr<T, A> {
    /// Get a reference to the allocator of the data
    ///
    /// Invariants: GcPtr must not be dangling
    pub unsafe fn allocator(&self) -> &A {
        self.inner.as_ref().allocator()
    }

    /// Get a reference to the GC'd data
    ///
    /// Invariants: GcPtr must not be dangling
    pub unsafe fn data(&self) -> &T {
        self.inner.as_ref().data()
    }

    /// Tell if this ptr is managed or not
    ///
    /// Invariants: GcPtr must not be dangling
    pub unsafe fn is_unmanaged(&self) -> bool {
        self.inner.as_ref().is_unmanaged()
    }

    /// Free the data behind this GcPtr
    ///
    /// Invariants: GcPtr must not be dangling, must not be managed and must not be read again
    pub unsafe fn deallocate(self) {
        drop(Box::from_raw_in(self.inner.as_ptr(), self.allocator()))
    }

    pub(crate) fn erased(self) -> NonNull<Allocation<Data, A>> {
        unsafe { NonNull::new_unchecked(self.inner.as_ptr() as *mut Allocation<Data, A>) }
    }

    pub(crate) unsafe fn erased_pinned<'a>(self) -> Pin<&'a Allocation<Data, A>> {
        Pin::new_unchecked(&*self.erased().as_ptr())
    }
}

unsafe impl<T: Trace + ?Sized, A: Allocator + 'static> Trace for GcPtr<T, A> {
    unsafe fn mark(&self) {
        self.inner.as_ref().mark();
    }

    unsafe fn manage(&self) {
        super::manage(*self)
    }

    unsafe fn finalize(&mut self) {}
}

impl<T: ?Sized, A: Allocator> Clone for GcPtr<T, A> {
    fn clone(&self) -> GcPtr<T, A> {
        *self
    }
}

impl<T: ?Sized, A: Allocator> Copy for GcPtr<T, A> {}
