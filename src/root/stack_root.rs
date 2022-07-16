use std::pin::Pin;

use nocturne_gc::{GcPtr, Trace};

use crate::root::Reroot;
use crate::Gc;

pub struct Root<'root> {
    root: Pin<&'root mut nocturne_gc::Root>,
}

impl<'root> Root<'root> {
    #[doc(hidden)]
    pub unsafe fn new(root: &'root mut nocturne_gc::Root) -> Root<'root> {
        Root {
            root: Pin::new_unchecked(root),
        }
    }

    pub fn gc<T>(self, data: T) -> Gc<'root, T::Rerooted>
    where
        T: Reroot<'root> + Trace,
        T::Rerooted: Trace,
    {
        unsafe { self.make(nocturne_gc::alloc_unmanaged(data)) }
    }

    pub fn reroot<T>(self, gc: Gc<'_, T>) -> Gc<'root, T::Rerooted>
    where
        T: Reroot<'root> + ?Sized,
        T::Rerooted: Trace,
    {
        unsafe { self.make(Gc::raw(gc)) }
    }

    pub(crate) unsafe fn make<T>(mut self, ptr: GcPtr<T>) -> Gc<'root, T::Rerooted>
    where
        T: Reroot<'root> + ?Sized,
        T::Rerooted: Trace,
    {
        let ptr = super::reroot(ptr);
        self.emplace(ptr);
        Gc::rooted(ptr)
    }

    unsafe fn emplace<T: Trace + ?Sized>(&mut self, ptr: GcPtr<T>) {
        Pin::get_mut(Pin::as_mut(&mut self.root)).enroot(ptr)
    }
}

#[macro_export]
macro_rules! letroot {
    ($($root:ident)*) => {$(
        // Ensure the root is owned
        let mut $root = $crate::raw::Root::new();

        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $root = unsafe {
            $crate::Root::new(&mut $root)
        };
    )*}
}
