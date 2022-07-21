use std::{
    alloc::{Allocator, Global},
    marker::{PhantomData, PhantomPinned},
};

use nocturne_gc::{GcPtr, Trace};

use crate::Gc;

pub struct GcStore<'root, T: ?Sized + 'root, A: Allocator = Global> {
    ptr: GcPtr<T, A>,
    _marker: PhantomData<(&'root T, PhantomPinned)>,
}

impl<'root, T: Trace> GcStore<'root, T> {
    pub fn new(data: T) -> GcStore<'root, T> {
        GcStore {
            ptr: nocturne_gc::alloc_unmanaged(data),
            _marker: PhantomData,
        }
    }
}

impl<'root, T: Trace, A: Allocator + Clone> GcStore<'root, T, A> {
    pub fn new_in(data: T, allocator: A) -> GcStore<'root, T, A> {
        GcStore {
            ptr: nocturne_gc::alloc_unmanaged_in(data, allocator),
            _marker: PhantomData,
        }
    }
}

impl<'root, T: ?Sized, A: Allocator> GcStore<'root, T, A> {
    pub fn get(&self) -> &T {
        unsafe {
            if self.ptr.is_unmanaged() {
                self.ptr.data()
            } else {
                panic!("Cannot call `GcStore::get` after the GcStore has been rooted.")
            }
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unimplemented!()
    }

    pub fn get_maybe(&self) -> Option<&T> {
        unsafe {
            if self.ptr.is_unmanaged() {
                Some(self.ptr.data())
            } else {
                None
            }
        }
    }

    pub fn get_mut_maybe(&mut self) -> Option<&mut T> {
        unimplemented!()
    }

    pub fn raw(this: &GcStore<'root, T, A>) -> GcPtr<T, A> {
        this.ptr
    }
}

unsafe impl<'root, T: Trace + ?Sized, A: Allocator + 'static> Trace for GcStore<'root, T, A> {
    unsafe fn mark(&self) {
        self.ptr.mark();
    }

    unsafe fn manage(&self) {
        self.ptr.manage();
    }

    unsafe fn finalize(&mut self) {}
}

impl<'root, T: ?Sized + Trace, A: Allocator> From<Gc<'root, T, A>> for GcStore<'root, T, A> {
    fn from(gc: Gc<'root, T, A>) -> GcStore<'root, T, A> {
        GcStore {
            ptr: Gc::raw(gc),
            _marker: PhantomData,
        }
    }
}

impl<'root, T: ?Sized, A: Allocator> Drop for GcStore<'root, T, A> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr.is_unmanaged() {
                self.ptr.deallocate()
            }
        }
    }
}
