use crate::{Gc, GcStore};

pub unsafe trait Store<'root> {
    type Accessor: 'root;
    unsafe fn rooted(this: &'root Self) -> Self::Accessor;
}

unsafe impl<'root, 'r, T: ?Sized + 'root, A: Allocator + 'static> Store<'root>
    for GcStore<'r, T, A>
{
    type Accessor = Gc<'root, T, A>;
    unsafe fn rooted(this: &'root Self) -> Self::Accessor {
        Gc::rooted(GcStore::raw(this))
    }
}

macro_rules! transmute_store {
    ($(for<$($T:ident),*> $from:ty => $to:ty;)*) => {$(
        unsafe impl<'root, 'r, A: Allocator + 'static, $($T: ?Sized + 'root,)*> Store<'root> for $from {
            type Accessor = &'root $to;
            unsafe fn rooted(this: &'root $from) -> &'root $to {
                std::mem::transmute::<&'root $from, &'root $to>(this)
            }
        }
    )*}
}

use pin_cell::PinCell;
use std::{alloc::Allocator, collections::*};

transmute_store! {
    for<T> Box<GcStore<'r, T, A>, A> => Box<Gc<'root, T, A>, A>;
    for<T> Option<GcStore<'r, T, A>> => Option<Gc<'root, T, A>>;
    for<T> [GcStore<'r, T, A>] => [Gc<'root, T, A>];
    for<T> Vec<GcStore<'r, T, A>, A> => Vec<Gc<'root, T, A>, A>;
    for<T> VecDeque<GcStore<'r, T, A>, A> => VecDeque<Gc<'root, T, A>, A>;
    for<T> HashSet<GcStore<'r, T, A>, A> => HashSet<Gc<'root, T, A>, A>;
    for<T> BTreeSet<GcStore<'r, T, A>> => BTreeSet<Gc<'root, T, A>>;
    for<T> BinaryHeap<GcStore<'r, T, A>> => BinaryHeap<Gc<'root, T, A>>;
    for<T> PinCell<GcStore<'r, T, A>> => PinCell<Gc<'root, T, A>>;
}
