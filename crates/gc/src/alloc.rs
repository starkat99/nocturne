use crate::{list::List, trace::Trace};
use log::*;
use std::{
    alloc::{Allocator, Global},
    cell::Cell,
    mem,
    ptr::NonNull,
};

pub struct Data {
    _extern: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

struct Vtable {
    _extern: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
struct TraitObject {
    data: *const Data,
    vtable: *mut Vtable,
}

pub struct Allocation<T: ?Sized, A: Allocator = Global> {
    header: Header<A>,
    pub(crate) data: T,
}

struct Header<A: Allocator = Global> {
    list: List<Allocation<Data, A>>,
    vtable: *mut Vtable,
    marked: Cell<bool>,
    allocator: A,
}

impl<T: Trace> Allocation<T> {
    pub fn new(data: T) -> NonNull<Allocation<T>> {
        let vtable = extract_vtable(&data);

        let allocation = Box::new(Allocation {
            header: Header {
                list: List::default(),
                vtable,
                marked: Cell::new(false),
                allocator: Global,
            },
            data,
        });
        Box::leak(allocation).into()
    }
}

impl<T: Trace, A: Allocator> Allocation<T, A> {
    pub fn new_in(data: T, allocator: A) -> NonNull<Allocation<T, A>> {
        let vtable = extract_vtable(&data);

        // Create unitialized memory then initialize it, so we don't have to clone allocator
        let (allocation, allocator) = Box::into_raw_with_allocator(Box::new_uninit_in(allocator));

        unsafe {
            NonNull::new_unchecked(allocation)
                .as_mut()
                .write(Allocation {
                    header: Header {
                        list: List::default(),
                        vtable,
                        marked: Cell::new(false),
                        allocator,
                    },
                    data,
                })
                .into()
        }
    }
}

impl<T: ?Sized, A: Allocator> Allocation<T, A> {
    pub fn allocator(&self) -> &A {
        &self.header.allocator
    }
}

impl<A: Allocator> Allocation<Data, A> {
    pub unsafe fn free(self: *mut Allocation<Data, A>) {
        (&mut *self).dyn_data_mut().finalize();
        drop(Box::from_raw_in(self, (&*self).allocator()))
    }
}

impl<T: ?Sized, A: Allocator> Allocation<T, A> {
    pub unsafe fn mark(&self) {
        debug!(
            "MARKING object at: {:x}",
            self.erased() as *const _ as usize
        );
        if !self.header.marked.replace(true) {
            self.dyn_data().mark()
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn marked(&self) -> bool {
        self.header.marked.replace(false)
    }

    pub fn is_unmanaged(&self) -> bool {
        self.header.list.is_head()
    }

    fn dyn_data(&self) -> &dyn Trace {
        unsafe {
            let object = TraitObject {
                data: self.erased().data() as *const Data,
                vtable: self.header.vtable,
            };
            mem::transmute::<TraitObject, &dyn Trace>(object)
        }
    }

    fn dyn_data_mut(&mut self) -> &mut dyn Trace {
        unsafe {
            let object = TraitObject {
                data: self.erased().data() as *const Data,
                vtable: self.header.vtable,
            };
            mem::transmute::<TraitObject, &mut dyn Trace>(object)
        }
    }

    fn erased(&self) -> &Allocation<Data, A> {
        unsafe { &*(self as *const Allocation<T, A> as *const Allocation<Data, A>) }
    }
}

impl<A: Allocator> AsRef<List<Allocation<Data, A>>> for Allocation<Data, A> {
    fn as_ref(&self) -> &List<Allocation<Data, A>> {
        &self.header.list
    }
}

fn extract_vtable<T: Trace>(data: &T) -> *mut Vtable {
    unsafe {
        let obj = data as &dyn Trace;
        mem::transmute::<&dyn Trace, TraitObject>(obj).vtable
    }
}
