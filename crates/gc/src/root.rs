use std::alloc::{Allocator, Global};
use std::marker::PhantomData;

use crate::gc_ptr::GcPtr;
use crate::trace::Trace;

pub struct Root<A: Allocator + 'static = Global> {
    idx: usize,
    _phantom: PhantomData<A>,
}

impl Root {
    pub fn new() -> Root {
        Root {
            idx: super::new_root::<Global>(),
            _phantom: Default::default(),
        }
    }
}

impl<A: Allocator + 'static> Root<A> {
    pub fn new_in(_allocator: A) -> Root {
        Root {
            idx: super::new_root::<A>(),
            _phantom: Default::default(),
        }
    }

    pub unsafe fn enroot<T: Trace + ?Sized>(&self, gc_ptr: GcPtr<T, A>) {
        super::set_root(self.idx, gc_ptr)
    }
}

impl<A: Allocator + 'static> Drop for Root<A> {
    fn drop(&mut self) {
        super::pop_root::<A>(self.idx);
    }
}
