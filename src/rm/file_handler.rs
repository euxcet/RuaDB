use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::utils::bit::*;
use crate::utils::string::*;

use super::record::*;
use super::filesystem::bufmanager::buf_page_manager::BufPageManager;
use super::filesystem::pagedef::*;


pub struct FileHandler{
    fd: i32,
    // header: &'a mut FileHeader,
    // first_page_index: i32,
    cache: Rc<RefCell<BufPageManager>>,
    used_page: HashSet<i32>,
}

impl FileHandler {
    pub fn new(file_id: i32, cache: Rc<RefCell<BufPageManager>>) -> Self {
        let mut s = Self {
            fd: file_id,
            // header: unsafe{ convert(first_page) },
            // first_page_index: index,

            cache: cache,
            used_page: HashSet::new(),
        };
        let header = unsafe{s.header_mut()};
        
        if header.has_used == 0 {
            header.has_used = 1;
            header.free_record_page = 0;
            header.free_string_page = 0;
            header.least_unused_page = 1;
        }

        s
    }

    unsafe fn page_mut<'b, T>(&mut self, page_id: u32, dirty: bool) -> &'b mut T {
        let mut cache = self.cache.borrow_mut();
        let (page, index) = cache.get_page(self.fd, page_id as i32);
        self.used_page.insert(index);
        if dirty {
            cache.mark_dirty(index);
        }

        convert(page)
    }

    pub fn get_fd(&self) -> i32 {
        self.fd
    }

    pub fn close(&mut self) {
        self.cache.borrow_mut().write_back_file(self.fd, &self.used_page);
    }

    pub fn set_column(&mut self, columns: &Vec<ColumnType>) {
        let header = unsafe{ self.header_mut() };
        assert_eq!(header.has_used, 0);

        header.column_num = columns.len() as u32;

        for (i, column) in columns.iter().enumerate() {
            copy_bytes(&mut header.column_info[i].column_name, &column.name);
            header.column_info[i].can_be_null = column.can_be_null;
            header.column_info[i].is_primary = column.is_primary;
            header.column_info[i].has_index = column.has_index;
            header.column_info[i].is_foreign = column.is_foreign;
            if column.is_foreign {
                copy_bytes(&mut header.column_info[i].foreign_table_name, &column.foreign_table_name);
            }
            header.column_info[i].has_default = column.has_default;
            match column.data_type {
                Type::Int(d) => {
                    header.column_info[i].data_type = INT_TYPE;
                    if column.has_default {
                        if let Some(v) = d {
                            header.column_info[i].default_null = false;
                            header.column_info[i].default_value.int = v;
                        } else {
                            header.column_info[i].default_null = true;
                        }
                    }
                },
                Type::Float(d) => {
                    header.column_info[i].data_type = FLOAT_TYPE;
                    if column.has_default {
                        if let Some(v) = d {
                            header.column_info[i].default_null = false;
                            header.column_info[i].default_value.float = v;
                        } else {
                            header.column_info[i].default_null = true;
                        }
                    }
                },
                Type::Date(d) => {
                    header.column_info[i].data_type = DATE_TYPE;
                    if column.has_default {
                        if let Some(v) = d {
                            header.column_info[i].default_null = false;
                            header.column_info[i].default_value.date = v;
                        } else {
                            header.column_info[i].default_null = true;
                        }
                    }

                },
                Type::Str(len, ref d) => {
                    header.column_info[i].data_type = STR_TYPE;
                    assert!(len > 0);
                    header.column_info[i].max_len = len;
                    if column.has_default {
                        if let Some(ref v) = d {
                            header.column_info[i].default_null = false;
                            header.column_info[i].default_value.strp = self.alloc_string(v);
                        } else {
                            header.column_info[i].default_null = true;
                        }
                    }
                },
            }
        }
    }

    fn alloc_string(&mut self, s: &str) -> StrPointer {
        let s_len = s.len();
        let slot_num = (s_len + MAX_FIXED_STRING_LENGTH - 1) / MAX_FIXED_STRING_LENGTH;
        let mut spt = StrPointer {
            len: 0,
            page: 0,
            offset: 0,
        };
        for i in (0..slot_num).rev() {
            let (page_id, offset) = self.alloc_string_slot();
            let sp = unsafe{self.sp_mut(page_id)}; 

            let len = if i == slot_num - 1 {
                s_len - (slot_num - 1) * MAX_FIXED_STRING_LENGTH
            } else {
                MAX_FIXED_STRING_LENGTH
            };
            sp.strs[offset as usize].next = spt;
            sp.strs[offset as usize].bytes[0..len].clone_from_slice(s[(i * MAX_FIXED_STRING_LENGTH) .. (i * MAX_FIXED_STRING_LENGTH + len)].as_bytes());

            spt = StrPointer { len: len as u16, page: page_id, offset: offset };
        }

        spt
    }

    fn alloc_string_slot(&mut self) -> (u32, u16) {
        let header = unsafe{self.header_mut()};
        let need_new_page = header.free_string_page == 0;
        if need_new_page {
            header.free_string_page = self.use_new_page();
        }
        let page_id = header.free_string_page;

        let sp = unsafe{self.sp_mut(page_id)};
        if need_new_page {
            sp.header.usage = USAGE_STR;
        }

        let fsi = get_free_index(sp.free_slot);
        assert!(fsi < MAX_FIXED_STRING_NUMBER as u32);
        set_used(&mut sp.free_slot, fsi);

        if all_used(sp.free_slot, MAX_FIXED_STRING_NUMBER) {
            unsafe{self.header_mut()}.free_string_page = sp.header.next_free_page;
        }

        (page_id, fsi as u16)
    }


    fn free_string_slot(&mut self, strp: &StrPointer) {

    }



    fn use_new_page(&mut self) -> u32 {
        let header = unsafe{self.header_mut()};
        header.least_unused_page += 1;
        header.least_unused_page - 1
    }

    pub fn create_record() {

    }

    pub fn get_record() {

    }

    pub fn delete_record() {

    }

    pub fn update_record() {

    }

    pub fn add_column() {

    }

    pub fn delete_column() {

    }

    unsafe fn header_mut<'b>(&mut self) -> &'b mut FileHeader { self.page_mut(0, true) }
    unsafe fn header<'b>(&mut self) -> &'b FileHeader { self.page_mut(0, false) } 
    unsafe fn sp_mut<'b>(&mut self, page_id: u32) -> &'b mut StringPage { self.page_mut(page_id, true) }
    unsafe fn sp<'b>(&mut self, page_id: u32) -> &'b StringPage { self.page_mut(page_id, false) }
    unsafe fn rp_mut<'b>(&mut self, page_id: u32) -> &'b mut RecordPage { self.page_mut(page_id, true) }
    unsafe fn rp<'b>(&mut self, page_id: u32) -> &'b RecordPage { self.page_mut(page_id, false) }
}


#[test]
fn bit_op_test() {
    assert_eq!(get_free_index(0xFFu32), 8);
    assert!(all_used(0xFFFFFFFFu32, MAX_FIXED_STRING_NUMBER));
    assert!(!all_used(0xFFFFFF8Fu32, MAX_FIXED_STRING_NUMBER));

    let mut a = 0x1u32;
    set_used(&mut a, 1);
    assert_eq!(a, 0x3u32);
}

#[test]
fn divide() {
    assert_eq!(6/5, 1);
}


/*
----------
stack_top 4 u32
column_num 4 u32
list
[
    record_type 1
    none 1
]
----------

----------
next_page 4
free 1 00000000 (bitmap)

record:
type 1
{
    int    8 i64
    float  8 f64
    date   8 u64
    str {
        len 2 u16
        content len
    }
}
----------
*/
