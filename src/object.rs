use std;
use std::cmp;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::str;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::class::{ClassDefId, ClassSize};
use crate::gc::root::Slot;
use crate::gc::Address;
use crate::handle::{root, Handle};
use crate::mem;
use crate::vm::VM;
use crate::vtable::VTable;

#[repr(C)]
pub struct Header {
    // ptr to class
    vtable: AtomicUsize,

    // forwarding ptr
    // (used during mark-compact)
    fwdptr: AtomicUsize,
}

const MARK_BITS: usize = 2;
const MARK_MASK: usize = (2 << MARK_BITS) - 1;
const FWD_MASK: usize = !0 & !MARK_MASK;

impl Header {
    #[cfg(test)]
    fn new() -> Header {
        Header {
            vtable: AtomicUsize::new(0),
            fwdptr: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn size() -> i32 {
        std::mem::size_of::<Header>() as i32
    }

    #[inline(always)]
    pub fn vtbl(&self) -> &mut VTable {
        unsafe { &mut *self.vtblptr().to_mut_ptr::<VTable>() }
    }

    #[inline(always)]
    pub fn vtblptr(&self) -> Address {
        self.vtable.load(Ordering::Relaxed).into()
    }

    #[inline(always)]
    pub fn set_vtblptr(&mut self, addr: Address) {
        self.vtable.store(addr.to_usize(), Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn vtblptr_forward(&mut self, address: Address) {
        self.vtable.store(address.to_usize() | 1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn vtblptr_forwarded(&self) -> Option<Address> {
        let addr = self.vtable.load(Ordering::Relaxed);

        if (addr & 1) == 1 {
            Some((addr & !1).into())
        } else {
            None
        }
    }

    pub fn vtblptr_repair(&mut self) {
        let addr = self.vtable.load(Ordering::Relaxed);

        if (addr & 3) == 3 {
            // forwarding failed
            let vtblptr = (addr & !3).into();
            self.set_vtblptr(vtblptr);
        } else if (addr & 1) == 1 {
            // object was forwarded
            let fwd: Address = (addr & !1).into();
            let fwd = fwd.to_obj();
            let vtblptr = fwd.header().vtblptr();

            self.set_vtblptr(vtblptr);
        } else {
            // nothing to do
        }
    }

    #[inline(always)]
    pub fn vtblptr_forwarded_atomic(&self) -> Result<Address, Address> {
        let addr = self.vtable.load(Ordering::Relaxed);

        if (addr & 3) == 3 {
            Ok(Address::from_ptr(self as *const _))
        } else if (addr & 1) == 1 {
            Ok((addr & !1).into())
        } else {
            Err(addr.into())
        }
    }

    #[inline(always)]
    pub fn vtblptr_forward_atomic(
        &mut self,
        expected_vtblptr: Address,
        new_address: Address,
    ) -> Result<(), Address> {
        let fwd = new_address.to_usize() | 1;
        let result =
            self.vtable
                .compare_and_swap(expected_vtblptr.to_usize(), fwd, Ordering::SeqCst);

        if result == expected_vtblptr.to_usize() {
            Ok(())
        } else {
            // If update fails, this needs to be a forwarding pointer
            debug_assert!((result | 1) != 0);

            if (result & 3) == 3 {
                Err(Address::from_ptr(self as *const _))
            } else {
                Err((result & !1).into())
            }
        }
    }

    #[inline(always)]
    pub fn vtblptr_forward_failure_atomic(
        &mut self,
        expected_vtblptr: Address,
    ) -> Result<(), Address> {
        let fwd = expected_vtblptr.to_usize() | 3;
        let result =
            self.vtable
                .compare_and_swap(expected_vtblptr.to_usize(), fwd, Ordering::SeqCst);

        if result == expected_vtblptr.to_usize() {
            Ok(())
        } else {
            // If update fails, this needs to be a forwarding pointer
            debug_assert!((result | 1) != 0);

            if (result & 3) == 3 {
                Err(Address::from_ptr(self as *const _))
            } else {
                Err((result & !1).into())
            }
        }
    }

    #[inline(always)]
    pub fn fwdptr_non_atomic(&self) -> Address {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        (fwdptr & FWD_MASK).into()
    }

    #[inline(always)]
    pub fn set_fwdptr_non_atomic(&mut self, addr: Address) {
        debug_assert!((addr.to_usize() & MARK_MASK) == 0);
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr
            .store(addr.to_usize() | (fwdptr & MARK_MASK), Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn mark_non_atomic(&mut self) {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr.store(fwdptr | 1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn unmark_non_atomic(&mut self) {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr.store(fwdptr & FWD_MASK, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn is_marked_non_atomic(&self) -> bool {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);
        (fwdptr & MARK_MASK) != 0
    }

    #[inline(always)]
    pub fn try_mark_non_atomic(&self) -> bool {
        let fwdptr = self.fwdptr.load(Ordering::Relaxed);

        if (fwdptr & MARK_MASK) != 0 {
            return false;
        }

        self.fwdptr.store(fwdptr | 1, Ordering::Relaxed);
        true
    }

    #[inline(always)]
    pub fn try_mark(&self) -> bool {
        let old = self.fwdptr.load(Ordering::Relaxed);
        self.fwdptr
            .compare_exchange(old, old | 1, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
    }
}

// is used to reference any object
#[repr(C)]
pub struct Obj {
    header: Header,
    data: u8,
}

impl Obj {
    #[inline(always)]
    pub fn address(&self) -> Address {
        Address::from_ptr(self as *const _)
    }

    #[inline(always)]
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline(always)]
    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    #[inline(always)]
    pub fn data(&self) -> *const u8 {
        &self.data as *const u8
    }

    pub fn is_array_ref(&self) -> bool {
        self.header().vtbl().is_array_ref()
    }

    pub fn size_for_vtblptr(&self, vtblptr: Address) -> usize {
        let vtbl = unsafe { &*vtblptr.to_mut_ptr::<VTable>() };
        let cls = vtbl.class();

        match cls.size {
            ClassSize::Fixed(size) => size as usize,

            ClassSize::ObjArray => determine_array_size(self, mem::ptr_width()),
            ClassSize::FreeArray => determine_array_size(self, mem::ptr_width()),

            ClassSize::Array(element_size) => determine_array_size(self, element_size),

            ClassSize::Str => {
                let handle: Ref<Str> = Ref {
                    ptr: self as *const Obj as *const Str,
                };
                mem::align_usize(handle.size(), mem::ptr_width() as usize)
            }
        }
    }

    pub fn size(&self) -> usize {
        self.size_for_vtblptr(self.header().vtblptr())
    }

    pub fn visit_reference_fields<F>(&mut self, mut f: F)
    where
        F: FnMut(Slot),
    {
        let classptr = self.header().vtbl().classptr;
        let cls = unsafe { &*classptr };

        if let ClassSize::ObjArray = cls.size {
            let array = unsafe { &*(self as *const _ as *const StrArray) };

            // walk through all objects in array
            let mut ptr = Address::from_ptr(array.data());
            let last = ptr.add_ptr(array.len() as usize);

            while ptr < last {
                f(Slot::at(ptr));
                ptr = ptr.add_ptr(1);
            }

            return;
        }

        let addr = self.address();

        for &offset in &cls.ref_fields {
            let obj = addr.offset(offset as usize);
            f(Slot::at(obj));
        }
    }

    pub fn visit_reference_fields_within<F>(&mut self, limit: Address, mut f: F)
    where
        F: FnMut(Slot),
    {
        let classptr = self.header().vtbl().classptr;
        let cls = unsafe { &*classptr };

        if let ClassSize::ObjArray = cls.size {
            let array = unsafe { &*(self as *const _ as *const StrArray) };

            // walk through all objects in array
            let mut ptr = Address::from_ptr(array.data());
            let last = ptr.add_ptr(array.len() as usize);

            // visit elements until `limit` reached
            let limit = cmp::min(last, limit);

            while ptr < limit {
                f(Slot::at(ptr));
                ptr = ptr.add_ptr(1);
            }

            return;
        }

        let addr = self.address();

        // visit the whole object all the time
        for &offset in &cls.ref_fields {
            let obj = addr.offset(offset as usize);
            f(Slot::at(obj));
        }
    }

    pub fn copy_to(&self, dest: Address, size: usize) {
        unsafe {
            ptr::copy(
                self as *const Obj as *const u8,
                dest.to_mut_ptr::<u8>(),
                size,
            );
        }
    }
}

fn determine_array_size(obj: &Obj, element_size: i32) -> usize {
    let handle: Ref<ByteArray> = Ref {
        ptr: obj as *const Obj as *const ByteArray,
    };

    let value = Header::size() as usize
        + mem::ptr_width() as usize
        + element_size as usize * handle.len() as usize;

    mem::align_usize(value, mem::ptr_width() as usize)
}

#[repr(C)]
pub struct Ref<T> {
    ptr: *const T,
}

unsafe impl<T> Send for Ref<T> {}
unsafe impl<T> Sync for Ref<T> {}

impl<T> Ref<T> {
    pub fn null() -> Ref<T> {
        Ref { ptr: ptr::null() }
    }

    pub fn cast<R>(&self) -> Ref<R> {
        Ref {
            ptr: self.ptr as *const R,
        }
    }

    pub fn raw(&self) -> *const T {
        self.ptr
    }

    pub fn address(&self) -> Address {
        Address::from_ptr(self.ptr)
    }
}

// known limitation of #[derive(Copy, Clone)]
// traits need to be implemented manually
impl<T> Copy for Ref<T> {}
impl<T> Clone for Ref<T> {
    fn clone(&self) -> Ref<T> {
        *self
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Ref<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

impl<T> Into<Ref<T>> for usize {
    fn into(self) -> Ref<T> {
        Ref {
            ptr: self as *const T,
        }
    }
}

impl<T> Into<Ref<T>> for Address {
    fn into(self) -> Ref<T> {
        Ref { ptr: self.to_ptr() }
    }
}

#[repr(C)]
pub struct Testing {
    header: Header,
    failed: bool,
}

impl Testing {
    pub fn has_failed(&self) -> bool {
        self.failed
    }
}

#[repr(C)]
pub struct Str {
    header: Header,
    length: usize,
    data: u8,
}

impl Str {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn data(&self) -> *const u8 {
        &self.data as *const u8
    }

    pub fn content(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data(), self.len()) }
    }

    pub fn to_cstring(&self) -> CString {
        let view = self.content();

        CString::new(view).unwrap()
    }

    pub fn size(&self) -> usize {
        Header::size() as usize         // Object header
            + mem::ptr_width() as usize // length field
            + self.len() // string content
    }

    /// allocates string from buffer in permanent space
    pub fn from_buffer_in_perm(vm: &VM, buf: &[u8]) -> Ref<Str> {
        let mut handle = str_alloc_perm(vm, buf.len());
        handle.length = buf.len();

        unsafe {
            let data = handle.data() as *mut u8;

            // copy buffer content into Str
            ptr::copy_nonoverlapping(buf.as_ptr(), data, buf.len());
        }

        handle
    }

    /// allocates string from buffer in heap
    pub fn from_buffer(vm: &VM, buf: &[u8]) -> Ref<Str> {
        let mut handle = str_alloc_heap(vm, buf.len());
        handle.length = buf.len();

        unsafe {
            let data = handle.data() as *mut u8;

            // copy buffer content into Str
            ptr::copy_nonoverlapping(buf.as_ptr(), data, buf.len());
        }

        handle
    }

    pub fn from_str(vm: &VM, val: Handle<Str>, offset: usize, len: usize) -> Ref<Str> {
        let total_len = val.len();

        if offset > total_len {
            return Ref::null();
        }

        let len = std::cmp::min(total_len - offset, len);

        let slice = unsafe {
            let data = val.data().offset(offset as isize);
            slice::from_raw_parts(data, len)
        };

        if let Ok(_) = str::from_utf8(slice) {
            let mut handle = str_alloc_heap(vm, len);
            handle.length = len;

            unsafe {
                let dest = handle.data() as *mut u8;
                let src = val.data().offset(offset as isize);

                // copy buffer content into Str
                ptr::copy_nonoverlapping(src, dest, len);
            }

            handle
        } else {
            Ref::null()
        }
    }

    pub fn concat(vm: &VM, lhs: Handle<Str>, rhs: Handle<Str>) -> Handle<Str> {
        let len = lhs.len() + rhs.len();
        let mut handle = root(str_alloc_heap(vm, len));

        unsafe {
            handle.length = len;

            ptr::copy_nonoverlapping(lhs.data(), handle.data() as *mut u8, lhs.len());
            ptr::copy_nonoverlapping(
                rhs.data(),
                handle.data().offset(lhs.len() as isize) as *mut u8,
                rhs.len(),
            );
        }

        handle
    }

    // duplicate string into a new object
    pub fn dup(&self, vm: &VM) -> Ref<Str> {
        let len = self.len();
        let mut handle = str_alloc_heap(vm, len);

        unsafe {
            handle.length = len;

            ptr::copy_nonoverlapping(self.data(), handle.data() as *mut u8, len);
        }

        handle
    }
}

fn str_alloc_heap(vm: &VM, len: usize) -> Ref<Str> {
    str_alloc(vm, len, |vm, size| vm.gc.alloc(vm, size, false))
}

fn str_alloc_perm(vm: &VM, len: usize) -> Ref<Str> {
    str_alloc(vm, len, |vm, size| vm.gc.alloc_perm(size))
}

fn str_alloc<F>(vm: &VM, len: usize, alloc: F) -> Ref<Str>
where
    F: FnOnce(&VM, usize) -> Address,
{
    let size = Header::size() as usize      // Object header
                + mem::ptr_width() as usize // length field
                + len; // string content

    let size = mem::align_usize(size, mem::ptr_width() as usize);
    let ptr = alloc(vm, size);

    let clsid = vm.vips.str(vm);
    let cls = vm.class_defs.idx(clsid);
    let cls = cls.read();
    let vtable: *const VTable = &**cls.vtable.as_ref().unwrap();
    let mut handle: Ref<Str> = ptr.into();
    handle.header_mut().set_vtblptr(Address::from_ptr(vtable));

    handle
}

pub trait ArrayElement {
    const REF: bool;
}

impl ArrayElement for bool {
    const REF: bool = false;
}

impl ArrayElement for u8 {
    const REF: bool = false;
}

impl ArrayElement for char {
    const REF: bool = false;
}

impl ArrayElement for i32 {
    const REF: bool = false;
}

impl ArrayElement for i64 {
    const REF: bool = false;
}

impl ArrayElement for f32 {
    const REF: bool = false;
}

impl ArrayElement for f64 {
    const REF: bool = false;
}

impl ArrayElement for Ref<Str> {
    const REF: bool = true;
}

#[repr(C)]
pub struct Array<T: Copy> {
    header: Header,
    length: usize,
    data: u8,
    phantom: PhantomData<T>,
}

impl<T> Array<T>
where
    T: Copy + ArrayElement,
{
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn data(&self) -> *const T {
        &self.data as *const u8 as *const T
    }

    pub fn data_mut(&mut self) -> *mut T {
        &self.data as *const u8 as *mut T
    }

    pub fn get_at(&self, idx: usize) -> T {
        unsafe { *self.data().offset(idx as isize) }
    }

    pub fn set_at(&mut self, idx: usize, val: T) {
        unsafe {
            *self.data_mut().offset(idx as isize) = val;
        }
    }

    pub fn size(&self) -> usize {
        Header::size() as usize         // Object header
            + mem::ptr_width() as usize // length field
            + self.len() * std::mem::size_of::<T>() // array content
    }

    pub fn alloc(vm: &VM, len: usize, elem: T, clsid: ClassDefId) -> Ref<Array<T>> {
        let size = Header::size() as usize        // Object header
                   + mem::ptr_width() as usize    // length field
                   + len * std::mem::size_of::<T>(); // array content

        let ptr = vm.gc.alloc(vm, size, T::REF).to_usize();
        let cls = vm.class_defs.idx(clsid);
        let cls = cls.read();
        let vtable: *const VTable = &**cls.vtable.as_ref().unwrap();
        let mut handle: Ref<Array<T>> = ptr.into();
        handle.header_mut().set_vtblptr(Address::from_ptr(vtable));
        handle.length = len;

        for i in 0..handle.len() {
            unsafe {
                *handle.data_mut().offset(i as isize) = elem;
            }
        }

        handle
    }
}

pub fn offset_of_array_length() -> i32 {
    offset_of!(Array<i32>, length) as i32
}

pub fn offset_of_array_data() -> i32 {
    offset_of!(Array<i32>, data) as i32
}

pub type BoolArray = Array<bool>;
pub type ByteArray = Array<u8>;
pub type CharArray = Array<char>;
pub type IntArray = Array<i32>;
pub type LongArray = Array<i64>;
pub type FloatArray = Array<f32>;
pub type DoubleArray = Array<f64>;
pub type StrArray = Array<Ref<Str>>;

pub fn alloc(vm: &VM, clsid: ClassDefId) -> Ref<Obj> {
    let cls_def = vm.class_defs.idx(clsid);
    let cls_def = cls_def.read();

    let size = match cls_def.size {
        ClassSize::Fixed(size) => size as usize,
        _ => panic!("alloc only supports fix-sized types"),
    };

    let size = mem::align_usize(size, mem::ptr_width() as usize);

    let ptr = vm.gc.alloc(vm, size, false).to_usize();
    let vtable: *const VTable = &**cls_def.vtable.as_ref().unwrap();
    let mut handle: Ref<Obj> = ptr.into();
    handle.header_mut().set_vtblptr(Address::from_ptr(vtable));

    handle
}

pub struct Throwable {
    pub header: Header,
    pub msg: Ref<Str>,
    pub backtrace: Ref<IntArray>,
    pub elements: Ref<Obj>,
}

// Error is subclass of Throwable, possibly there is a better way to represent that
pub struct Error {
    pub header: Header,
    pub msg: Ref<Str>,
    pub backtrace: Ref<IntArray>,
    pub elements: Ref<Obj>,
}

// Exception is subclass of Throwable, possibly there is a better way to represent that
pub struct Exception {
    pub header: Header,
    pub msg: Ref<Str>,
    pub backtrace: Ref<IntArray>,
    pub elements: Ref<Obj>,
}

pub struct StackTraceElement {
    pub header: Header,
    pub name: Ref<Str>,
    pub line: i32,
}

#[cfg(test)]
mod tests {
    use crate::object::Header;

    #[test]
    fn header_markbit() {
        let mut h = Header::new();
        h.set_fwdptr_non_atomic(8.into());
        assert_eq!(false, h.is_marked_non_atomic());
        assert_eq!(8, h.fwdptr_non_atomic().to_usize());
        h.mark_non_atomic();
        assert_eq!(true, h.is_marked_non_atomic());
        assert_eq!(8, h.fwdptr_non_atomic().to_usize());
        h.set_fwdptr_non_atomic(16.into());
        assert_eq!(true, h.is_marked_non_atomic());
        assert_eq!(16, h.fwdptr_non_atomic().to_usize());
        h.unmark_non_atomic();
        assert_eq!(false, h.is_marked_non_atomic());
        assert_eq!(16, h.fwdptr_non_atomic().to_usize());
    }
}
