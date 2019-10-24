use std;
use std::ptr;
use std::alloc::{alloc, Layout};

use super::find_replace::FindReplace;
use super::super::utils::hashmap::Hashmap;
use super::super::fileio::file_manager::FileManager;
use super::super::pagedef::*;

// use crate::filesystem::utils::hashmap::Hashmap;
// use crate::filesystem::bufmanager::find_replace::FindReplace;
// use crate::filesystem::fileio::file_manager::FileManager;
// use crate::filesystem::pagedef::*;

pub struct BufPageManager {
    last: i32,
    file_manager: FileManager,
    hash: Hashmap,
    replace: FindReplace,
    dirty: [bool; CAP],
    addr: [*mut u8; CAP],
}

impl BufPageManager {
    fn alloc_page_mem() -> *mut u8 {
        unsafe {
            alloc(Layout::new::<[u8; PAGE_SIZE as usize]>())
        }
    }
    fn fetch_page(&mut self, type_id: i32, page_id: i32) -> (*mut u8, i32) {
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
        self.hash.replace(index, type_id, page_id);

        (b, index)
    }

    pub fn alloc_page(&mut self, file_id: i32, page_id: i32, read: bool) -> (*mut u8, i32) {
        let (b, index) = self.fetch_page(file_id, page_id);
        if read {
            unsafe {
                self.file_manager.read_page(file_id, page_id, std::slice::from_raw_parts_mut(b, PAGE_SIZE as usize), 0);
            }
        }

        (b, index)
    }

    pub fn get_page(&mut self, file_id: i32, page_id: i32) -> (*mut u8, i32) {
        let index = self.hash.find_index(file_id, page_id);
        match index {
            -1 => {
                let (b, i) = self.fetch_page(file_id, page_id);
                unsafe {
                    self.file_manager.read_page(file_id, page_id, std::slice::from_raw_parts_mut(b, PAGE_SIZE as usize), 0);
                }
                (b, i)
            },
            _ => {
                self.access(index);
                (self.addr[index as usize], index)
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

    pub fn write_back(&mut self, index: i32) {
        if self.dirty[index as usize] {
            let (f, p) = self.hash.get_keys(index);
            unsafe {
                self.file_manager.write_page(f, p, std::slice::from_raw_parts(self.addr[index as usize], PAGE_SIZE as usize), 0);
            }
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
            addr: [ptr::null_mut(); CAP],
            dirty: [false; CAP],
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

#[test]
fn test_file_system() {
    let mut bpm = BufPageManager::new();
    bpm.file_manager.create_file("d:/Rua/testfile.txt");
    bpm.file_manager.create_file("d:/Rua/testfile2.txt");
    let f1 = bpm.file_manager.open_file("d:/Rua/testfile.txt");
    let f2 = bpm.file_manager.open_file("d:/Rua/testfile2.txt");
    for page_id in 0..1000 {
        let (b, index) = bpm.alloc_page(f1, page_id, false);
        unsafe {
            let buf = BufPageManager::to_slice_mut(b);
            buf[0] = page_id as u8;
            buf[1] = f1 as u8;
        }
        bpm.mark_dirty(index);
        let (b, index) = bpm.alloc_page(f2, page_id, false);
        unsafe {
            let buf = BufPageManager::to_slice_mut(b);
            buf[0] = page_id as u8;
            buf[1] = f2 as u8;
        }
        bpm.mark_dirty(index);
    }

    println!("");

    for page_id in 0..1000 {
        let (b, index) = bpm.get_page(f1, page_id);
        unsafe {
            let buf = BufPageManager::to_slice(b);
            assert_eq!(buf[0], page_id as u8);
            assert_eq!(buf[1], f1 as u8);
        }
        bpm.access(index);

        let (b, index) = bpm.get_page(f2, page_id);

        unsafe {
            let buf = BufPageManager::to_slice(b);
            assert_eq!(buf[0], page_id as u8);
            assert_eq!(buf[1], f2 as u8);
        }
        bpm.access(index);
    }
    bpm.close();
}

#[test]
fn test_mem() {
    let b = unsafe {alloc(Layout::new::<[u8; PAGE_SIZE as usize]>())};
    let buf = unsafe {std::slice::from_raw_parts_mut(b, PAGE_SIZE as usize)};
    buf[0] = 10u8;
    buf[1] = 20u8;
    let a = b;
    let buf = unsafe {std::slice::from_raw_parts_mut(a, PAGE_SIZE as usize)};
    assert_eq!(buf[0], 10u8);
    assert_eq!(buf[1], 20u8);
}
