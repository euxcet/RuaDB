use std::rc::Rc;
use std::mem::transmute;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashSet;

use crate::utils::bit::*;
use crate::utils::string::*;
use crate::bytevec;
use crate::index::btree::*;

use super::pagedef::*;
use super::filesystem::bufmanager::buf_page_manager::BufPageManager;

#[derive(Clone)]
pub struct FileHandler{
    fd: i32,
    cache: Rc<RefCell<BufPageManager>>,
    used_page: RefCell<HashSet<i32>>,
}

impl FileHandler {
    pub fn new(file_id: i32, cache: Rc<RefCell<BufPageManager>>) -> Self {
        let s = Self {
            fd: file_id,
            cache: cache,
            used_page: RefCell::new(HashSet::new()),
        };
        let header = unsafe{s.header_mut()};
        if header.has_used == 0 {
            header.has_used = 1;
            header.free_page = 0;
            header.free_large_page = 0;
            header.least_unused_page = 1;
            header.btrees_ptr = 0;
            header.column_types_ptr = 0;
            header.btree = 0;
        }
        s
    }

    pub fn close(&self) {
        self.cache.borrow_mut().write_back_file(self.fd, &self.used_page.borrow());
        assert!(self.cache.borrow_mut().file_manager.close_file(self.fd).is_ok());
    }

    // get a specific page
    unsafe fn header_mut<'b>(&self) -> &'b mut FileHeader { self.page_mut(0, true) }
    unsafe fn header<'b>(&self) -> &'b FileHeader { self.page_mut(0, false) } 
    unsafe fn lsp_mut<'b>(&self, page_id: u32) -> &'b mut LargeSlotPage { self.page_mut(page_id, true) }
    unsafe fn sp_mut<'b>(&self, page_id: u32) -> &'b mut StringPage { self.page_mut(page_id, true) }
    unsafe fn sp<'b>(&self, page_id: u32) -> &'b StringPage { self.page_mut(page_id, false) }
    unsafe fn ph_mut<'b>(&self, page_id: u32) -> &'b mut PageHeader { self.page_mut(page_id, false) }
    unsafe fn ph<'b>(&self, page_id: u32) -> &'b PageHeader { self.page_mut(page_id, false) }
    unsafe fn page_mut<'b, T>(&self, page_id: u32, dirty: bool) -> &'b mut T {
        let (page, index) = self.cache.borrow_mut().get_page(self.fd, page_id as i32);
        self.used_page.borrow_mut().insert(page_id as i32);
        if dirty {
            self.cache.borrow_mut().mark_dirty(index);
        }
        transmute(page)
    }

    fn add_page(&self) -> u32 {
        let header = unsafe{self.header_mut()};
        header.least_unused_page += 1;
        header.least_unused_page - 1
    }

    // alloc a slot to contain a section of a string.
    fn alloc_slot(&self) -> (u32, u32) {
        let max_number = SLOT_PER_PAGE;
        let header = unsafe{self.header_mut()};
        let need_new_page = header.free_page == 0;
        if need_new_page {
            header.free_page = self.add_page();
        }
        let page_id = header.free_page;
        let ph = unsafe{self.ph_mut(page_id)};
        let fsi = get_free_index(ph.free_slot);
        ph.is_large = false;
        assert!(fsi < max_number as u32);
        set_used(&mut ph.free_slot, fsi);

        if all_used(ph.free_slot, max_number) {
            unsafe{self.header_mut()}.free_page = ph.next_free_page;
        }
        (page_id, fsi)
    }

    fn alloc_large_slot(&self) -> (u32, u32) {
        let max_number = LARGE_SLOT_PER_PAGE;
        let header = unsafe{self.header_mut()};
        let need_new_page = header.free_large_page == 0;
        if need_new_page {
            header.free_large_page = self.add_page();
        }
        let page_id = header.free_large_page;
        let ph = unsafe{self.ph_mut(page_id)};
        let fsi = get_free_index(ph.free_slot);
        ph.is_large = true;
        assert!(fsi < max_number as u32);
        set_used(&mut ph.free_slot, fsi);
        if all_used(ph.free_slot, max_number) {
            unsafe{self.header_mut()}.free_large_page = ph.next_free_page;
        }
        (page_id, fsi)
    }

    // alloc a string in the file, return a pointer.
    pub fn alloc(&self, data: &Vec<u8>, is_large: bool) -> StrPointer {
        let length = if is_large {LARGE_SLOT_LENGTH} else {SLOT_LENGTH};
        let mut d_offset = 0;
        let mut slot = if is_large {self.alloc_large_slot()} else {self.alloc_slot()};
        let ptr = StrPointer { page: slot.0, offset: slot.1 };
        while d_offset < data.len() {
            let sp = unsafe{self.sp_mut(slot.0)};
            let ss = &mut sp.strs[slot.1 as usize];
            let len = min(data.len() - d_offset, length);
            ss.len = (data.len() - d_offset) as u64;
            copy_bytes_u8(&mut ss.bytes, &data[d_offset .. d_offset + len]);
            d_offset += len;
            if d_offset < data.len() {
                slot = if is_large {self.alloc_large_slot()} else {self.alloc_slot()};
                ss.next = StrPointer { page: slot.0, offset: slot.1 };
            }
        }
        ptr
    }

    // free a slot
    fn free_slot(&self, strp: &StrPointer) -> StrPointer {
        let this_page = strp.page;
        let offset = strp.offset;

        let ph = unsafe{self.ph(this_page)};
        if ph.is_large {
            let sp = unsafe{self.lsp_mut(this_page)};
            assert!((strp.offset as usize) < LARGE_SLOT_PER_PAGE);
            assert!(is_used(sp.header.free_slot, strp.offset as usize));
            set_free(&mut sp.header.free_slot, strp.offset as u32);
            let next = sp.strs[offset as usize].next.clone();
            sp.strs[offset as usize].len = 0;
            sp.strs[offset as usize].next.set_null();
            if free_num(sp.header.free_slot, LARGE_SLOT_PER_PAGE) == 1 {
                let header = unsafe{self.header_mut()};
                let stack_top = header.free_large_page;
                header.free_large_page = this_page;
                unsafe{self.sp_mut(this_page)}.header.next_free_page = stack_top;
            }
            next
        }
        else {
            let sp = unsafe{self.sp_mut(this_page)};
            assert!((strp.offset as usize) < SLOT_PER_PAGE);
            assert!(is_used(sp.header.free_slot, strp.offset as usize));
            set_free(&mut sp.header.free_slot, strp.offset as u32);
            let next = sp.strs[offset as usize].next.clone();
            sp.strs[offset as usize].len = 0;
            sp.strs[offset as usize].next.set_null();
            if free_num(sp.header.free_slot, SLOT_PER_PAGE) == 1 {
                let header = unsafe{self.header_mut()};
                let stack_top = header.free_page;
                header.free_page = this_page;
                unsafe{self.sp_mut(this_page)}.header.next_free_page = stack_top;
            }
            next
        }
    }

    // free a string in the file
    pub fn free(&self, strp: &StrPointer) {
        let mut p = strp.clone();
        while !p.is_null() {
            p = self.free_slot(&p);
        }
    }

    // insert a struct into the file
    pub fn insert<T, Size>(&self, data: &T) -> StrPointer
        where T: bytevec::ByteEncodable + bytevec::ByteDecodable,
              Size: bytevec::BVSize + bytevec::ByteEncodable + bytevec::ByteDecodable {
        let bytes = data.encode::<Size>().unwrap();
        self.alloc(&bytes, false)
    }

    // get a struct from the file
    pub fn get<T, Size>(&self, ptr: &StrPointer) -> T
        where T: bytevec::ByteEncodable + bytevec::ByteDecodable,
              Size: bytevec::BVSize + bytevec::ByteEncodable + bytevec::ByteDecodable {
        let mut res: Vec<u8> = Vec::new();
        let mut t = ptr;
        while !t.is_null() {
            let sp = unsafe{self.sp(t.page)};
            let ss = &sp.strs[t.offset as usize];
            let len = min(ss.len as usize, SLOT_LENGTH);
            let bytes = &ss.bytes[..len];
            res.extend_from_slice(bytes);
            t = &ss.next;
        }
        T::decode::<Size>(&res).unwrap()
    }

    pub fn get_mut<T>(&self, ptr: &StrPointer) -> &mut T {
        let sp = unsafe{self.lsp_mut(ptr.page)};
        unsafe{transmute(sp.strs[ptr.offset as usize].bytes.as_mut_ptr())}
    }

    // update a struct in the file
    pub fn update<T, Size>(&self, ptr: &StrPointer, data: &T)
        where T: bytevec::ByteEncodable + bytevec::ByteDecodable,
              Size: bytevec::BVSize + bytevec::ByteEncodable + bytevec::ByteDecodable {
        let length = SLOT_LENGTH;
        let data = data.encode::<Size>().unwrap();
        let mut d_offset = 0; 
        let mut t = ptr;
        while d_offset < data.len() {
            let sp = unsafe{self.sp_mut(t.page)};
            let ss = &mut sp.strs[t.offset as usize];
            let len = min(data.len() - d_offset, length);
            ss.len = (data.len() - d_offset) as u64;
            copy_bytes_u8(&mut ss.bytes, &data[d_offset .. d_offset + len]);
            d_offset += len;
            if d_offset < data.len() && ss.next.to_u64() == 0 {
                let slot = self.alloc_slot();
                ss.next = StrPointer { page: slot.0, offset: slot.1 };
            }
            else if d_offset == data.len() && !ss.next.is_null() {
                self.free(&ss.next);
                ss.next.set_null();
            }
            t = &ss.next;
        }
    }

    pub fn update_sub(&self, ptr: &StrPointer, offset: usize, data: Vec<u8>) {
        let length = SLOT_LENGTH;
        let mut offset = offset;
        let mut t = ptr;
        while offset > length {
            let sp = unsafe{self.sp(t.page)};
            t = &sp.strs[t.offset as usize].next;
            offset -= length;
        }
        let mut d_offset: usize = 0;
        while d_offset < data.len() {
            let sp = unsafe{self.sp_mut(t.page)};
            let ss = &mut sp.strs[t.offset as usize];
            let len = min(data.len() - d_offset, length);
            copy_bytes_u8_offset(&mut ss.bytes, &data[d_offset .. d_offset + len], offset);
            offset = 0;
            d_offset += len;
            if d_offset < data.len() && ss.next.to_u64() == 0 {
                let slot = self.alloc_slot();
                ss.next = StrPointer { page: slot.0, offset: slot.1 };
            }
            t = &ss.next;
        }
    }

    // delete a struct in the file
    pub fn delete(&self, ptr: &StrPointer) {
        self.free(ptr);
    }

    pub fn get_column_types_ptr(&self) -> u64 {
        let header = unsafe { self.header_mut() };
        header.column_types_ptr
    }

    pub fn set_column_types_ptr(&self, ptr: u64) {
        let header = unsafe { self.header_mut() };
        header.column_types_ptr = ptr;
    }

    pub fn set_btrees_ptr(&self, ptr: u64) {
        let header = unsafe { self.header_mut() };
        header.btrees_ptr = ptr;
    }

    pub fn get_btrees_ptr(&self) -> u64 {
        let header = unsafe { self.header_mut() };
        header.btrees_ptr
    }

    pub fn get_born_btree_ptr(&self) -> u64 {
        let header = unsafe { self.header_mut() };
        header.btree
    }

    pub fn set_born_btree_ptr(&self, ptr: u64) {
        let header = unsafe { self.header_mut() };
        header.btree = ptr;
    }
}