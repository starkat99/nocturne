use pin_cell::PinCell;
use std::cell::{Cell, RefCell};
use std::collections::*;
use std::mem::{self, ManuallyDrop};
use std::ptr;

pub unsafe trait Trace {
    unsafe fn mark(&self);
    unsafe fn manage(&self);
    unsafe fn finalize(&mut self);
}

pub unsafe trait NullTrace: Trace {}

unsafe impl<T: Trace> Trace for Option<T> {
    unsafe fn mark(&self) {
        if let Some(inner) = self {
            inner.mark()
        }
    }
    unsafe fn manage(&self) {
        if let Some(inner) = self {
            inner.manage()
        }
    }

    unsafe fn finalize(&mut self) {
        if let Some(inner) = self {
            inner.finalize()
        }
    }
}

unsafe impl<T: NullTrace> NullTrace for Option<T> {}

unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    unsafe fn mark(&self) {
        match self {
            Ok(inner) => inner.mark(),
            Err(error) => error.mark(),
        }
    }
    unsafe fn manage(&self) {
        match self {
            Ok(inner) => inner.manage(),
            Err(error) => error.manage(),
        }
    }
    unsafe fn finalize(&mut self) {
        match self {
            Ok(inner) => inner.finalize(),
            Err(error) => error.finalize(),
        }
    }
}

unsafe impl<T: NullTrace, E: NullTrace> NullTrace for Result<T, E> {}

unsafe impl<T: Trace> Trace for [T] {
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark()
        }
    }
    unsafe fn manage(&self) {
        for elem in self {
            elem.manage()
        }
    }
    unsafe fn finalize(&mut self) {
        for elem in self {
            elem.finalize()
        }
    }
}

unsafe impl<T: NullTrace> NullTrace for [T] {}

unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    unsafe fn mark(&self) {
        <_ as AsRef<[T]>>::as_ref(self).mark()
    }
    unsafe fn manage(&self) {
        <_ as AsRef<[T]>>::as_ref(self).manage()
    }
    unsafe fn finalize(&mut self) {
        <_ as AsMut<[T]>>::as_mut(self).finalize()
    }
}
unsafe impl<T: NullTrace, const N: usize> NullTrace for [T; N] {}

macro_rules!
    trace_simple { ($($t:ty)*) => {$(
        unsafe impl Trace for $t {
            unsafe fn mark(&self) { }
            unsafe fn manage(&self) { }
            unsafe fn finalize(&mut self) {
                ptr::drop_in_place(self as *mut Self)
            }
        }
        unsafe impl NullTrace for $t { }
    )*}
}

trace_simple!(
    i8  i16 i32 i64 isize
    u8  u16 u32 u64 usize
    f32     f64
    char    bool
    str     String
    std::fs::File
    std::fs::FileType
    std::fs::Metadata
    std::fs::OpenOptions
    dyn std::io::BufRead
    dyn std::io::Read
    dyn std::io::Write
    std::io::Stdin
    std::io::Stdout
    std::io::Stderr
    std::io::Error
    std::net::TcpStream
    std::net::TcpListener
    std::net::UdpSocket
    std::net::Ipv4Addr
    std::net::Ipv6Addr
    std::net::SocketAddrV4
    std::net::SocketAddrV6
    std::path::Path
    std::path::PathBuf
    std::process::Command
    std::process::Child
    std::process::ChildStdout
    std::process::ChildStdin
    std::process::ChildStderr
    std::process::Output
    std::process::ExitStatus
    std::process::Stdio
    std::sync::Barrier
    std::sync::Condvar
    std::sync::Once
);

macro_rules! trace_tuples {
    ($(($($T:ident : $N:tt),*))*) => {$(
        unsafe impl<$($T: Trace,)*> Trace for ($($T,)*) {
            unsafe fn mark(&self) {
                $(self.$N.mark();)*
            }
            unsafe fn manage(&self) {
                $(self.$N.manage();)*
            }
            unsafe fn finalize(&mut self) {
                $(self.$N.finalize();)*
            }
        }
        unsafe impl<$($T: NullTrace,)*> NullTrace for ($($T,)*) { }
    )*};
}

trace_tuples! {
    ()
    (A: 0)
    (A: 0, B: 1)
    (A: 0, B: 1, C: 2)
    (A: 0, B: 1, C: 2, D: 3)
    (A: 0, B: 1, C: 2, D: 3, E: 4)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10)
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11)
}

unsafe impl<T: Trace> Trace for Vec<T> {
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark();
        }
    }

    unsafe fn manage(&self) {
        for elem in self {
            elem.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        for elem in &mut *self {
            elem.finalize();
        }
        let this = mem::transmute::<&mut Vec<T>, &mut Vec<ManuallyDrop<T>>>(self);
        ptr::drop_in_place(this as *mut Vec<ManuallyDrop<T>>);
    }
}

unsafe impl<T: NullTrace> NullTrace for Vec<T> {}

unsafe impl<T: Trace> Trace for VecDeque<T> {
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark();
        }
    }

    unsafe fn manage(&self) {
        for elem in self {
            elem.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        for elem in &mut *self {
            elem.finalize();
        }
        let this = mem::transmute::<&mut VecDeque<T>, &mut VecDeque<ManuallyDrop<T>>>(self);
        ptr::drop_in_place(this as *mut VecDeque<ManuallyDrop<T>>);
    }
}

unsafe impl<T: NullTrace> NullTrace for VecDeque<T> {}

unsafe impl<T: Trace> Trace for LinkedList<T> {
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark();
        }
    }

    unsafe fn manage(&self) {
        for elem in self {
            elem.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        for elem in &mut *self {
            elem.finalize();
        }
        let this = mem::transmute::<&mut LinkedList<T>, &mut LinkedList<ManuallyDrop<T>>>(self);
        ptr::drop_in_place(this as *mut LinkedList<ManuallyDrop<T>>);
    }
}

unsafe impl<T: NullTrace> NullTrace for LinkedList<T> {}

unsafe impl<T: Trace + Ord> Trace for BinaryHeap<T> {
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark();
        }
    }

    unsafe fn manage(&self) {
        for elem in self {
            elem.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        let iter = IntoIterator::into_iter(ptr::read(self));
        let iter = mem::transmute::<binary_heap::IntoIter<T>, binary_heap::IntoIter<ManuallyDrop<T>>>(
            iter,
        );
        iter.for_each(|mut elem| elem.finalize());
    }
}

unsafe impl<T: NullTrace + Ord> NullTrace for BinaryHeap<T> {}

unsafe impl<T, S> Trace for HashSet<T, S>
where
    T: Eq + std::hash::Hash + Trace,
    S: std::hash::BuildHasher,
{
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark();
        }
    }

    unsafe fn manage(&self) {
        for elem in self {
            elem.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        let iter = IntoIterator::into_iter(ptr::read(self));
        let iter =
            mem::transmute::<hash_set::IntoIter<T>, hash_set::IntoIter<ManuallyDrop<T>>>(iter);
        iter.for_each(|mut elem| elem.finalize());
    }
}

unsafe impl<T, S> NullTrace for HashSet<T, S>
where
    T: Eq + std::hash::Hash + NullTrace,
    S: std::hash::BuildHasher,
{
}

unsafe impl<K, V, S> Trace for HashMap<K, V, S>
where
    K: Eq + std::hash::Hash + Trace,
    V: Trace,
    S: std::hash::BuildHasher,
{
    unsafe fn mark(&self) {
        for (key, value) in self {
            key.mark();
            value.mark();
        }
    }

    unsafe fn manage(&self) {
        for (key, value) in self {
            key.manage();
            value.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        let iter = IntoIterator::into_iter(ptr::read(self));
        let iter = mem::transmute::<
            hash_map::IntoIter<K, V>,
            hash_map::IntoIter<ManuallyDrop<K>, ManuallyDrop<V>>,
        >(iter);
        iter.for_each(|(mut key, mut value)| {
            key.finalize();
            value.finalize();
        });
    }
}

unsafe impl<K, V, S> NullTrace for HashMap<K, V, S>
where
    K: Eq + std::hash::Hash + NullTrace,
    V: NullTrace,
    S: std::hash::BuildHasher,
{
}

unsafe impl<T> Trace for BTreeSet<T>
where
    T: Eq + Ord + Trace,
{
    unsafe fn mark(&self) {
        for elem in self {
            elem.mark();
        }
    }

    unsafe fn manage(&self) {
        for elem in self {
            elem.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        let iter = IntoIterator::into_iter(ptr::read(self));
        let iter =
            mem::transmute::<btree_set::IntoIter<T>, btree_set::IntoIter<ManuallyDrop<T>>>(iter);
        iter.for_each(|mut elem| elem.finalize());
    }
}

unsafe impl<T> NullTrace for BTreeSet<T> where T: Eq + Ord + NullTrace {}

unsafe impl<K, V> Trace for BTreeMap<K, V>
where
    K: Eq + Ord + Trace,
    V: Trace,
{
    unsafe fn mark(&self) {
        for (key, value) in self {
            key.mark();
            value.mark();
        }
    }

    unsafe fn manage(&self) {
        for (key, value) in self {
            key.manage();
            value.manage();
        }
    }

    unsafe fn finalize(&mut self) {
        let iter = IntoIterator::into_iter(ptr::read(self));
        let iter = mem::transmute::<
            btree_map::IntoIter<K, V>,
            btree_map::IntoIter<ManuallyDrop<K>, ManuallyDrop<V>>,
        >(iter);
        iter.for_each(|(mut key, mut value)| {
            key.finalize();
            value.finalize();
        });
    }
}

unsafe impl<K, V> NullTrace for BTreeMap<K, V>
where
    K: Eq + Ord + NullTrace,
    V: NullTrace,
{
}

unsafe impl<T: NullTrace> Trace for Cell<T> {
    unsafe fn mark(&self) {}
    unsafe fn manage(&self) {}
    unsafe fn finalize(&mut self) {
        ptr::drop_in_place(self as *mut Self)
    }
}

unsafe impl<T: NullTrace> NullTrace for Cell<T> {}

unsafe impl<T: NullTrace> Trace for RefCell<T> {
    unsafe fn mark(&self) {}
    unsafe fn manage(&self) {}
    unsafe fn finalize(&mut self) {
        ptr::drop_in_place(self as *mut Self)
    }
}

unsafe impl<T: NullTrace> NullTrace for RefCell<T> {}

unsafe impl<T: Trace> Trace for PinCell<T> {
    unsafe fn mark(&self) {
        self.borrow().mark()
    }
    unsafe fn manage(&self) {
        self.borrow().manage()
    }
    unsafe fn finalize(&mut self) {
        self.get_mut().finalize()
    }
}

unsafe impl<T: NullTrace> NullTrace for PinCell<T> {}
