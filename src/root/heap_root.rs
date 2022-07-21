use std::alloc::{Allocator, Global};
use std::ops::Deref;
use std::pin::Pin;

use nocturne_gc::{GcPtr, Root, Trace};

use crate::root::Reroot;
use crate::Gc;

pub struct HeapRoot<T: ?Sized, A: Allocator + 'static = Global> {
    _root: Pin<Box<Root<A>, A>>,
    ptr: GcPtr<T, A>,
}

impl<'root, T> HeapRoot<T>
where
    T: Reroot<'root> + Trace,
    T::Rerooted: Trace,
{
    pub fn new(data: T) -> HeapRoot<T::Rerooted> {
        unsafe { HeapRoot::make(nocturne_gc::alloc(data)) }
    }
}

impl<'root, T, A: Allocator + Clone + 'static> HeapRoot<T, A>
where
    T: Reroot<'root> + Trace,
    T::Rerooted: Trace,
{
    pub fn new_in(data: T, allocator: A) -> HeapRoot<T::Rerooted, A> {
        unsafe { HeapRoot::make(nocturne_gc::alloc_in(data, allocator)) }
    }

    pub fn reroot(gc: Gc<'_, T, A>) -> HeapRoot<T::Rerooted, A> {
        unsafe { HeapRoot::make(Gc::raw(gc)) }
    }

    unsafe fn make(ptr: GcPtr<T, A>) -> HeapRoot<T::Rerooted, A> {
        let ptr = super::reroot(ptr);
        let root = Pin::from(Box::new_in(
            Root::<A>::with_allocator(),
            ptr.allocator().clone(),
        ));
        Pin::get_ref(root.as_ref()).enroot(ptr);
        HeapRoot { _root: root, ptr }
    }
}

impl<T: ?Sized, A: Allocator + 'static> HeapRoot<T, A> {
    pub fn gc<'root>(&'root self) -> Gc<'root, T, A> {
        unsafe { Gc::rooted(self.ptr) }
    }
}

impl<T: Trace + ?Sized, A: Allocator + Clone + 'static> Clone for HeapRoot<T, A> {
    fn clone(&self) -> HeapRoot<T, A> {
        unsafe {
            let root = Pin::from(Box::new_in(
                Root::with_allocator(),
                self.ptr.allocator().clone(),
            ));

            Pin::get_ref(root.as_ref()).enroot(self.ptr);

            HeapRoot {
                _root: root,
                ptr: self.ptr,
            }
        }
    }
}

impl<T: ?Sized, A: Allocator + 'static> Deref for HeapRoot<T, A> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.data() }
    }
}
