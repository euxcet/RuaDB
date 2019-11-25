use std;
use std::ptr;
use std::alloc::{alloc, Layout};
use std::collections::HashSet;


use super::find_replace::FindReplace;
use super::super::utils::hashmap::Hashmap;
use super::super::fileio::file_manager::FileManager;
use super::super::pagedef::*;
use super::super::super::pagedef::*;

// use crate::filesystem::utils::hashmap::Hashmap;
// use crate::filesystem::bufmanager::find_replace::FindReplace;
// use crate::filesystem::fileio::file_manager::FileManager;
// use crate::filesystem::pagedef::*;

pub struct BufPageManager {
    last: i32,
    pub file_manager: FileManager,
    hash: Hashmap,
    replace: FindReplace,
    dirty: Vec<bool>,
    addr: Vec<*mut u8>, 
}

impl BufPageManager {
    fn alloc_page_mem() -> *mut u8 {
        unsafe{ alloc(Layout::new::<[u8; PAGE_SIZE as usize]>()) }
    }
    fn fetch_page(&mut self, file_id: i32, page_id: i32) -> (*mut u8, i32) {
        let index = self.replace.find();
        let mut b = self.addr[index as usize];
        if !b.is_null() {
            if self.dirty[index as usize] {
                let (k1, k2) = self.hash.get_keys(index);
                unsafe {
                    if self.file_manager.write_page(k1, k2, std::slice::from_raw_parts(b, PAGE_SIZE as usize), 0).is_err() {
                        panic!("write error!");
                    }
                }
                self.dirty[index as usize] = false;
            }
        } else {
            b = Self::alloc_page_mem();
            self.addr[index as usize] = b;
        }
        self.hash.replace(index, file_id, page_id);
        (b, index)
    }


    pub fn get_page(&mut self, file_id: i32, page_id: i32) -> (*mut u8, i32) {
        let index = self.hash.find_index(file_id, page_id);

        match index {
            -1 => {
                let (b, i) = self.fetch_page(file_id, page_id);
                unsafe{self.file_manager.read_page(file_id, page_id, std::slice::from_raw_parts_mut(b, PAGE_SIZE), 0).ok()};
                (b, i)
            },
            _ => {
                self.access(index);
                (self.addr[index as usize], index)
            }
        }
    }

    pub fn write_back_file(&mut self, file_id: i32, page_id_list: &HashSet<i32>) {
        for page_id in page_id_list {
            let index = self.hash.find_index(file_id, *page_id);
            if index != -1 {
                self.write_back_check(index, file_id, *page_id);
            }
        }
    }

    pub fn access(&mut self, index: i32) {
        if index == self.last {
            return;
        }
        self.replace.access(index);
        self.last = index;
    }

    pub fn mark_dirty(&mut self, index: i32) {
        self.dirty[index as usize] = true;
        self.access(index);
    }

    pub fn release(&mut self, index: i32) {
        self.dirty[index as usize] = false;
        self.replace.free(index);
        self.hash.remove(index);
    }

    pub fn write_back_check(&mut self, index: i32, fd: i32, pd: i32) {
        if self.dirty[index as usize] {
            let (f, p) = self.hash.get_keys(index);
            assert_eq!(f, fd);
            assert_eq!(p, pd);
            unsafe { self.file_manager.write_page(f, p, std::slice::from_raw_parts(self.addr[index as usize], PAGE_SIZE as usize), 0).ok(); }
            self.dirty[index as usize] = false;
        }
        self.replace.free(index);
        self.hash.remove(index);
    }

    pub fn write_back(&mut self, index: i32) {
        if self.dirty[index as usize] {
            let (f, p) = self.hash.get_keys(index);
            unsafe { self.file_manager.write_page(f, p, std::slice::from_raw_parts(self.addr[index as usize], PAGE_SIZE as usize), 0).ok(); }
            self.dirty[index as usize] = false;
        }
        self.replace.free(index);
        self.hash.remove(index);
    }

    pub fn close(&mut self) {
        for i in 0..CAP {
            self.write_back(i as i32);
        }
        for i in 0..CAP {
            if self.addr[i].is_null() {
                unsafe {
                    std::alloc::dealloc(self.addr[i], Layout::new::<[u8; PAGE_SIZE as usize]>());
                }
                self.addr[i] = ptr::null_mut();
            }
        }
    }

    pub fn get_key(&self, index: i32) -> (i32, i32) {
        self.hash.get_keys(index)
    }

    pub fn new() -> Self {
        let c = CAP as i32;
        let m = MOD as i32;

        Self {
            last: -1,
            file_manager: FileManager::new(),
            addr: vec![ptr::null_mut(); CAP],
            dirty: vec![false; CAP],
            hash: Hashmap::new(c, m),
            replace: FindReplace::new(c),
        }
    }

    pub unsafe fn to_slice_mut<'a>(data: *mut u8) -> &'a mut [u8] {
        std::slice::from_raw_parts_mut(data, PAGE_SIZE as usize)
    }

    pub unsafe fn to_slice<'a>(data: *const u8) -> &'a [u8] {
        std::slice::from_raw_parts(data, PAGE_SIZE as usize)
    }
}

