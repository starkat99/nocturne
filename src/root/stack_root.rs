use std::alloc::{Allocator, Global};
use std::pin::Pin;

use nocturne_gc::{GcPtr, Trace};

use crate::root::Reroot;
use crate::Gc;

pub struct Root<'root, A: Allocator + 'static = Global> {
    root: Pin<&'root mut nocturne_gc::Root<A>>,
}

impl<'root> Root<'root> {
    pub fn gc<T>(self, data: T) -> Gc<'root, T::Rerooted>
    where
        T: Reroot<'root> + Trace,
        T::Rerooted: Trace,
    {
        unsafe { self.make(nocturne_gc::alloc_unmanaged(data)) }
    }
}

impl<'root, A: Allocator + Unpin + 'static> Root<'root, A> {
    pub fn gc_in<T>(self, data: T, allocator: A) -> Gc<'root, T::Rerooted, A>
    where
        T: Reroot<'root> + Trace,
        T::Rerooted: Trace,
    {
        unsafe { self.make(nocturne_gc::alloc_unmanaged_in(data, allocator)) }
    }
}

impl<'root, A: Allocator + 'static> Root<'root, A> {
    #[doc(hidden)]
    pub unsafe fn new(root: &'root mut nocturne_gc::Root<A>) -> Root<'root, A> {
        Root {
            root: Pin::new_unchecked(root),
        }
    }
}

impl<'root, A: Allocator + Unpin + 'static> Root<'root, A> {
    pub fn reroot<T>(self, gc: Gc<'_, T, A>) -> Gc<'root, T::Rerooted, A>
    where
        T: Reroot<'root> + ?Sized,
        T::Rerooted: Trace,
    {
        unsafe { self.make(Gc::raw(gc)) }
    }

    pub(crate) unsafe fn make<T>(mut self, ptr: GcPtr<T, A>) -> Gc<'root, T::Rerooted, A>
    where
        T: Reroot<'root> + ?Sized,
        T::Rerooted: Trace,
    {
        let ptr = super::reroot(ptr);
        self.emplace(ptr);
        Gc::rooted(ptr)
    }

    unsafe fn emplace<T: Trace + ?Sized>(&mut self, ptr: GcPtr<T, A>) {
        Pin::get_mut(self.root.as_mut()).enroot(ptr)
    }
}

#[macro_export]
macro_rules! letroot {
    ($($root:ident),*) => {$(
        // Ensure the root is owned
        let mut $root = $crate::raw::Root::new();

        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $root = unsafe {
            $crate::Root::new(&mut $root)
        };
    )*};
    ($($root:ident in $allocator:expr),*) => {$(
        // Ensure the root is owned
        let mut $root = $crate::raw::Root::new_in($allocator);

        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $root = unsafe {
            $crate::Root::new(&mut $root)
        };
    )*};
}
