#![feature(arbitrary_self_types, allocator_api)]

mod alloc;
mod gc_ptr;
mod list;
mod root;
mod state;
mod trace;

use anymap::AnyMap;
use std::alloc::{Allocator, Global};
use std::cell::RefCell;
use std::pin::Pin;

use crate::state::GcState;

pub use crate::gc_ptr::GcPtr;
pub use crate::root::Root;
pub use crate::trace::{NullTrace, Trace};

thread_local! {
    static GCMAP: RefCell<AnyMap> = RefCell::new(AnyMap::new());
}

/// Allocate an unmanaged GcPtr
pub fn alloc_unmanaged<T: Trace>(data: T) -> GcPtr<T> {
    GcPtr::new(data)
}

/// Allocate an unmanaged GcPtr
pub fn alloc_unmanaged_in<T: Trace, A: Allocator + Clone>(data: T, allocator: A) -> GcPtr<T, A> {
    GcPtr::new_in(data, allocator)
}

/// Allocate a managed GcPtr
pub fn alloc<T: Trace>(data: T) -> GcPtr<T> {
    let gc_ptr = alloc_unmanaged(data);
    unsafe {
        manage(gc_ptr);
    }
    gc_ptr
}

/// Allocate a managed GcPtr
pub fn alloc_in<T: Trace, A: Allocator + Clone + 'static>(data: T, allocator: A) -> GcPtr<T, A> {
    let gc_ptr = alloc_unmanaged_in(data, allocator);
    unsafe {
        manage(gc_ptr);
    }
    gc_ptr
}

/// Manage a GcPtr
///
/// Invariants: ptr must not be dangling and must not already be managed
pub unsafe fn manage<T: Trace + ?Sized, A: Allocator + 'static>(ptr: GcPtr<T, A>) {
    with_gc(|gc| gc.manage(ptr))
}

/// Count objects managed by the GC
pub fn count_managed_objects() -> usize {
    with_gc(|gc: Pin<&GcState<Global>>| gc.objects().into_iter().count())
}

/// Count objects managed by the GC
pub fn count_managed_objects_with_allocator<A: Allocator + 'static>() -> usize {
    with_gc(|gc: Pin<&GcState<A>>| gc.objects().into_iter().count())
}

/// Count roots into the GC
pub fn count_roots() -> usize {
    with_gc(|gc: Pin<&GcState<Global>>| gc.roots().len())
}

/// Count roots into the GC
pub fn count_roots_with_allocator<A: Allocator + 'static>() -> usize {
    with_gc(|gc: Pin<&GcState<A>>| gc.roots().len())
}

fn new_root<A: Allocator + 'static>() -> usize {
    with_gc(|gc: Pin<&GcState<A>>| gc.new_root())
}

fn set_root<T: Trace + ?Sized, A: Allocator + 'static>(idx: usize, ptr: GcPtr<T, A>) {
    with_gc(|gc| gc.set_root(idx, ptr))
}

fn pop_root<A: Allocator + 'static>(idx: usize) {
    with_gc(|gc: Pin<&GcState<A>>| gc.pop_root(idx))
}

fn with_gc<T, F: FnOnce(Pin<&GcState<A>>) -> T, A: Allocator + 'static>(f: F) -> T {
    GCMAP.with(|gcmap| {
        let mut gcmap = gcmap.borrow_mut();
        let mut gc = gcmap.get();
        if gc.is_none() {
            gcmap.insert(GcState::<A>::default());
            gc = gcmap.get();
        }
        let gc = gc.unwrap();
        let gc: Pin<&GcState<A>> = unsafe { Pin::new_unchecked(gc) };
        f(gc)
    })
}

pub fn collect() {
    with_gc(|gc: Pin<&GcState<Global>>| gc.collect())
}

pub fn collect_with_allocator<A: Allocator + 'static>() {
    with_gc(|gc: Pin<&GcState<A>>| gc.collect())
}
