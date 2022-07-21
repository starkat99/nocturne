use std::alloc::Allocator;
use std::alloc::Global;
use std::fmt;
use std::hash;
use std::marker::{PhantomData, PhantomPinned};
use std::ops::Deref;
use std::pin::Pin;

use nocturne_gc::{GcPtr, Trace};

pub struct Gc<'root, T: ?Sized + 'root, A: Allocator = Global> {
    ptr: GcPtr<T, A>,
    _marker: PhantomData<(&'root T, PhantomPinned)>,
}

impl<'root, T: ?Sized, A: Allocator> Clone for Gc<'root, T, A> {
    fn clone(&self) -> Gc<'root, T, A> {
        *self
    }
}

impl<'root, T: ?Sized, A: Allocator> Copy for Gc<'root, T, A> {}

unsafe impl<'root, T: Trace + ?Sized, A: Allocator> Trace for Gc<'root, T, A> {
    unsafe fn mark(&self) {}

    unsafe fn manage(&self) {}

    unsafe fn finalize(&mut self) {}
}

impl<'root, T: ?Sized, A: Allocator> Gc<'root, T, A> {
    pub unsafe fn rooted(ptr: GcPtr<T, A>) -> Gc<'root, T, A> {
        Gc {
            ptr,
            _marker: PhantomData,
        }
    }

    // NOTE: Problematic for copying collectors
    pub fn pin(self) -> Pin<Gc<'root, T, A>> {
        unsafe { Pin::new_unchecked(self) }
    }

    pub fn raw(this: Gc<'root, T, A>) -> GcPtr<T, A> {
        this.ptr
    }
}

impl<'root, T: ?Sized, A: Allocator> Deref for Gc<'root, T, A> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.data() }
    }
}

impl<'root, T: fmt::Debug + ?Sized, A: Allocator> fmt::Debug for Gc<'root, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &*self;
        write!(f, "Gc({:?})", inner)
    }
}

impl<'root, T: fmt::Display + ?Sized, A: Allocator> fmt::Display for Gc<'root, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(&*self, f)
    }
}

impl<'root, T: PartialEq + ?Sized, A: Allocator> PartialEq for Gc<'root, T, A> {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { self.ptr.data() == rhs.ptr.data() }
    }
}

impl<'root, T: Eq + ?Sized, A: Allocator> Eq for Gc<'root, T, A> {}

impl<'root, T: PartialOrd + ?Sized, A: Allocator> PartialOrd for Gc<'root, T, A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe { self.ptr.data().partial_cmp(other.ptr.data()) }
    }
}

impl<'root, T: Ord + ?Sized, A: Allocator> Ord for Gc<'root, T, A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe { self.ptr.data().cmp(other.ptr.data()) }
    }
}

impl<'root, T: hash::Hash + ?Sized, A: Allocator> hash::Hash for Gc<'root, T, A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        unsafe { self.ptr.data().hash(state) }
    }
}
