use std::alloc::{Allocator, Global};
use std::cell::{Ref, RefCell};
use std::pin::Pin;
use std::ptr::NonNull;

use log::*;

use crate::alloc::{Allocation, Data};
use crate::gc_ptr::GcPtr;
use crate::list::List;
use crate::trace::Trace;

pub struct GcState<A: Allocator = Global> {
    objects: List<Allocation<Data, A>>,
    roots: RefCell<Vec<Option<NonNull<Allocation<Data, A>>>>>,
}

impl<A: Allocator> GcState<A> {
    pub fn collect(self: Pin<&Self>) {
        for (idx, root) in self.roots()[..].iter().enumerate() {
            if let Some(root) = root {
                debug!(
                    "TRACING from root at:       {:x} (idx {:x})",
                    &*root as *const _ as usize, idx
                );
                unsafe {
                    root.as_ref().mark();
                }
            }
        }

        for object in self.objects() {
            if !object.marked() {
                debug!(
                    "FREEING unmarked object at: {:x}",
                    &*object as *const _ as usize
                );
                unsafe {
                    Allocation::free(
                        &*object as *const Allocation<Data, A> as *mut Allocation<Data, A>,
                    )
                }
            }
        }
    }

    pub unsafe fn manage<T: Trace + ?Sized>(self: Pin<&Self>, ptr: GcPtr<T, A>) {
        // TODO I should not need a dynamic check here but I am making mistakes
        if ptr.is_unmanaged() {
            self.objects().insert(ptr.erased_pinned());
        }
        ptr.data().manage();
    }

    pub fn new_root(self: Pin<&Self>) -> usize {
        let mut roots = self.roots.borrow_mut();
        let ret = roots.len();
        roots.push(None);
        ret
    }

    pub fn set_root<T: Trace + ?Sized>(self: Pin<&Self>, idx: usize, ptr: GcPtr<T, A>) {
        let root: NonNull<Allocation<Data, A>> = ptr.erased();
        debug!(
            "ENROOTING root at: {:x} (idx {:x})",
            root.as_ptr() as usize,
            idx
        );
        self.roots.borrow_mut()[idx] = Some(root);
    }

    pub fn pop_root(self: Pin<&Self>, idx: usize) {
        debug_assert!(idx + 1 == self.roots.borrow().len());
        if let Some(root) = self.roots.borrow_mut().pop().unwrap() {
            debug!(
                "DROPPING root at: {:x} (idx {:x})",
                root.as_ptr() as usize,
                idx
            );
        }
    }

    pub fn roots(&self) -> Ref<'_, [Option<NonNull<Allocation<Data, A>>>]> {
        Ref::map(self.roots.borrow(), |v| &v[..])
    }

    pub fn objects<'a>(self: Pin<&'a Self>) -> Pin<&'a List<Allocation<Data, A>>> {
        unsafe { Pin::map_unchecked(self, |this| &this.objects) }
    }
}

impl<A: Allocator> Default for GcState<A> {
    fn default() -> Self {
        Self {
            objects: Default::default(),
            roots: Default::default(),
        }
    }
}
