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
            header.least_unused_page = 1;
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
        assert!(fsi < max_number as u32);
        set_used(&mut ph.free_slot, fsi);
        if all_used(ph.free_slot, max_number) {
            unsafe{self.header_mut()}.free_page = ph.next_free_page;
        }
        (page_id, fsi)
    }

    // alloc a string in the file, return a pointer.
    pub fn alloc(&self, s: &Vec<u8>) -> StrPointer {
        let slot_num = (s.len() + SLOT_LENGTH - 1) / SLOT_LENGTH;
        let mut spt = StrPointer::new(0);

        for i in (0..slot_num).rev() {
            let (page_id, offset) = self.alloc_slot();
            let sp = unsafe{self.sp_mut(page_id)}; 
            let len = if i == slot_num - 1 {s.len() - (slot_num - 1) * SLOT_LENGTH} else {SLOT_LENGTH};
            sp.strs[offset as usize].next = spt;
            let begin = i * SLOT_LENGTH;
            sp.strs[offset as usize].len = (s.len() - begin) as u64;
            copy_bytes_u8(&mut sp.strs[offset as usize].bytes, &s[begin .. begin + len]);
            spt = StrPointer { page: page_id, offset: offset };
        }
        spt
    }

    // free a slot
    fn free_slot(&self, strp: &mut StrPointer) {
        let this_page = strp.page;
        let offset = strp.offset;
        let sp = unsafe{self.sp_mut(this_page)};
        assert!((strp.offset as usize) < SLOT_PER_PAGE);
        set_free(&mut sp.header.free_slot, strp.offset as u32);
        *strp = sp.strs[offset as usize].next;
        if free_num(sp.header.free_slot, SLOT_PER_PAGE) == 1 {
            let header = unsafe{self.header_mut()};
            let stack_top = header.free_page;
            header.free_page = this_page;
            unsafe{self.sp_mut(this_page)}.header.next_free_page = stack_top;
        }
    }

    // free a string in the file
    fn free(&self, strp: &mut StrPointer) {
        while !strp.is_null() {
            self.free_slot(strp);
        }
    }

    // insert a struct into the file
    pub fn insert<T, Size>(&self, data: &T) -> StrPointer
        where T: bytevec::ByteEncodable + bytevec::ByteDecodable,
              Size: bytevec::BVSize + bytevec::ByteEncodable + bytevec::ByteDecodable {
        let bytes = data.encode::<Size>().unwrap();
        self.alloc(&bytes)
    }

    // get a struct from the file
    pub fn get<T, Size>(&self, ptr: &StrPointer) -> T
        where T: bytevec::ByteEncodable + bytevec::ByteDecodable,
              Size: bytevec::BVSize + bytevec::ByteEncodable + bytevec::ByteDecodable {
        let mut res: Vec<u8> = Vec::new();
        let mut t = ptr.clone();
        while !t.is_null() {
            let sp = unsafe{self.sp(t.page)};
            let ss = &sp.strs[t.offset as usize];
            let len = min(ss.len as usize, SLOT_LENGTH);
            let bytes = &ss.bytes[..len];
            res.extend_from_slice(bytes);
            t = sp.strs[t.offset as usize].next.clone();
        }
        T::decode::<Size>(&res).unwrap()
    }

    pub fn get_btree_node(&self, ptr: &StrPointer) -> &mut BTreeNode {
        let sp = unsafe{self.sp(ptr.page)};
        let ss = &sp.strs[ptr.offset as usize];
        let ptr = ss.bytes.as_ptr() as *const BTreeNode;
        let t: &mut BTreeNode = unsafe{&mut *ptr};
        t
    }

    // update a struct in the file
    pub fn update<T, Size>(&self, ptr: &mut StrPointer, data: &T)
        where T: bytevec::ByteEncodable + bytevec::ByteDecodable,
              Size: bytevec::BVSize + bytevec::ByteEncodable + bytevec::ByteDecodable {
        self.free(ptr);
        *ptr = self.insert::<T, Size>(data);
    }

    pub fn update_sub(&self, ptr: &StrPointer, offset: usize, data: Vec<u8>) {
        let mut offset = offset;
        let mut t = ptr.clone();

        let slot_index = offset / SLOT_LENGTH;
        for _ in 0..slot_index {
            let sp = unsafe{self.sp(t.page)};
            t = sp.strs[t.offset as usize].next.clone();
        }
        offset -= slot_index * SLOT_LENGTH;

        let mut done: usize = 0;
        while done < data.len() {
            let sp = unsafe{self.sp_mut(t.page)};
            let slot_len = min(sp.strs[t.offset as usize].len as usize, SLOT_LENGTH);
            let copy_len = min(slot_len - offset, data.len() - done);
            assert!(copy_len > 0);
            copy_bytes_u8_offset(&mut sp.strs[t.offset as usize].bytes, &data[done .. done + copy_len], offset);
            t = sp.strs[t.offset as usize].next.clone();
            done += copy_len;
            offset = 0;
        }
    }

    // delete a struct in the file
    pub fn delete(&self, ptr: &mut StrPointer) {
        self.free(ptr);
    }
}