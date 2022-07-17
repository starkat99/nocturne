#![feature(allocator_api)]

mod gc;
mod gc_store;
mod no_trace;
mod root;
mod store;

#[cfg(test)]
mod tests;

pub use nocturne_derive::*;
pub use nocturne_gc::collect;

pub mod raw {
    pub use crate::root::Reroot;
    pub use crate::store::*;
    pub use nocturne_gc::{alloc, alloc_unmanaged, manage, GcPtr, Root};
    pub use nocturne_gc::{count_managed_objects, count_roots};
    pub use nocturne_gc::{NullTrace, Trace};
}

pub use self::gc::*;
pub use self::gc_store::*;
pub use self::no_trace::*;
pub use self::root::Root;

pub trait Finalize {
    fn finalize(&mut self);
}

pub unsafe trait UnsafeFinalize {
    fn finalize(&mut self);
}

impl<T: UnsafeFinalize + ?Sized> Finalize for T {
    fn finalize(&mut self) {
        UnsafeFinalize::finalize(self)
    }
}

pub trait GC<'root>: crate::raw::Reroot<'root> + crate::raw::Trace {}
