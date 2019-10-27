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
    cache: Rc<RefCell<BufPageManager>>,
    used_page: RefCell<HashSet<(i32, i32)>>,
    columns: Vec<Option<ColumnType>>,
}

impl FileHandler {
    pub fn new(file_id: i32, cache: Rc<RefCell<BufPageManager>>) -> Self {
        let mut s = Self {
            fd: file_id,
            cache: cache,
            used_page: RefCell::new(HashSet::new()),
            columns: vec![None;MAX_COLUMN_NUMBER],
        };
        let header = unsafe{s.header_mut()};
        
        if header.has_used == 0 {
            header.has_used = 1;
            header.free_page = [0,0];
            header.least_unused_page = 1;
        }
        
        if header.has_set_column == u32::max_value() {
            s.read_columns();
        }

        s
    }

    unsafe fn page_mut<'b, T>(&self, page_id: u32, dirty: bool) -> &'b mut T {
        let (page, index) = self.cache.borrow_mut().get_page(self.fd, page_id as i32);
        self.used_page.borrow_mut().insert((page_id as i32, index));
        if dirty {
            self.cache.borrow_mut().mark_dirty(index);
        }

        convert(page)
    }

    pub fn get_fd(&self) -> i32 {
        self.fd
    }

    pub fn close(&self) {
        self.cache.borrow_mut().write_back_file(self.fd, &self.used_page.borrow());
    }

    pub fn set_columns(&mut self, columns: &Vec<ColumnType>) {
        assert!(columns.len() <= MAX_COLUMN_NUMBER);
        let header = unsafe{ self.header_mut() };
        assert_eq!(header.has_set_column, 0);
        header.has_set_column = u32::max_value();
        header.column_num = columns.len() as u32;

        for (i, column) in columns.iter().enumerate() {

            let mut new_column = column.clone();
            new_column.index = i as u32;
            self.columns[i] = Some(new_column);

            let header = unsafe{ self.header_mut() };

            set_used(&mut header.column_slot, i as u32);
            copy_bytes(&mut header.column_info[i].column_name, &column.name);
            header.column_info[i].column_name_len = column.name.len() as u8;
            header.column_info[i].can_be_null = column.can_be_null;
            header.column_info[i].is_primary = column.is_primary;
            header.column_info[i].has_index = column.has_index;
            header.column_info[i].is_foreign = column.is_foreign;
            if column.is_foreign {
                copy_bytes(&mut header.column_info[i].foreign_table_name, &column.foreign_table_name);
                header.column_info[i].foreign_table_name_len = column.foreign_table_name.len() as u8;
            }
            header.column_info[i].has_default = column.has_default;
            header.column_info[i].default_null = column.default_null;
            match column.data_type {
                Type::Int(d) => {
                    header.column_info[i].data_type = INT_TYPE;
                    if column.has_default && !column.default_null {
                        if let Some(v) = d {
                            header.column_info[i].default_value.int = v;
                        } else {
                            unreachable!();
                        }
                    }
                },
                Type::Float(d) => {
                    header.column_info[i].data_type = FLOAT_TYPE;
                    if column.has_default && !column.default_null {
                        if let Some(v) = d {
                            header.column_info[i].default_value.float = v;
                        } else {
                            unreachable!();
                        }
                    }
                },
                Type::Date(d) => {
                    header.column_info[i].data_type = DATE_TYPE;
                    if column.has_default && !column.default_null{
                        if let Some(v) = d {
                            header.column_info[i].default_value.date = v;
                        } else {
                            unreachable!();
                        }
                    }

                },
                Type::Str(len, ref d) => {
                    header.column_info[i].data_type = STR_TYPE;
                    assert!(len > 0);
                    header.column_info[i].max_len = len;
                    if column.has_default && !column.default_null {
                        if let Some(ref v) = d {
                            header.column_info[i].default_value.strp = self.alloc_string(v);
                        } else {
                            unreachable!();
                        }
                    }
                },
            }
        }
    }

    fn alloc_string(&self, s: &str) -> StrPointer {
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
            copy_bytes(&mut sp.strs[offset as usize].bytes, &s[(i * MAX_FIXED_STRING_LENGTH) .. (i * MAX_FIXED_STRING_LENGTH + len)]);

            spt = StrPointer { len: len as u16, page: page_id, offset: offset };
        }

        spt
    }

    fn alloc_string_slot(&self) -> (u32, u16) {
        self.alloc_slot(USAGE_STR)
    }

    fn alloc_record_slot(&self) -> (u32, u16) {
        self.alloc_slot(USAGE_RECORD)
    }

    fn alloc_slot(&self, usage: u32) -> (u32, u16) {
        assert!(usage == USAGE_STR || usage == USAGE_RECORD);

        let i = (usage - 1) as usize;
        let max_number = if usage == USAGE_STR {
            MAX_FIXED_STRING_NUMBER
        } else {
            MAX_PAGE_RECORD_NUMBER
        };

        let header = unsafe{self.header_mut()};
        let need_new_page = header.free_page[i] == 0;

        if need_new_page {
            header.free_page[i] = self.use_new_page();
        }
        let page_id = header.free_page[i];

        let ph = unsafe{self.ph_mut(page_id)};
        if need_new_page {
            ph.usage = usage;
        }

        let fsi = get_free_index(ph.free_slot);

        assert!(fsi < max_number as u32);
        set_used(&mut ph.free_slot, fsi);

        if all_used(ph.free_slot, max_number) {
            unsafe{self.header_mut()}.free_page[i] = ph.next_free_page;
        }

        (page_id, fsi as u16)
    }

    fn free_string_slot(&self, strp: &mut StrPointer) {
        let this_page = strp.page;
        let offset = strp.offset;

        let sp = unsafe{self.sp_mut(this_page)};
        assert!(sp.header.usage == USAGE_STR);
        assert!((strp.offset as usize) < MAX_FIXED_STRING_NUMBER);

        set_free(&mut sp.header.free_slot, strp.offset as u32);
        *strp = sp.strs[offset as usize].next;

        if free_num(sp.header.free_slot, MAX_FIXED_STRING_NUMBER) == 1 {
            let header = unsafe{self.header_mut()};
            let stack_top = header.free_page[1];
            header.free_page[1] = this_page;
            unsafe{self.sp_mut(this_page)}.header.next_free_page = stack_top;
        }
    }

    fn free_string(&self, strp: &mut StrPointer) {
        loop {
            if strp.len == 0 {
                break;
            }
            self.free_string_slot(strp);
        }
    }

    fn use_new_page(&self) -> u32 {
        let header = unsafe{self.header_mut()};
        header.least_unused_page += 1;
        header.least_unused_page - 1
    }

    pub fn get_string(&self, strp: &StrPointer) -> String {
        let mut res = String::new();
        let mut t = strp.clone();
        loop {
            if t.len == 0 {
                break;
            }
            let sp = unsafe{self.sp(t.page)};
            res.push_str(from_bytes(&sp.strs[t.offset as usize].bytes, t.len as usize).as_str());
            t = sp.strs[t.offset as usize].next.clone();
        }
        
        res
    }
    
    pub fn get_columns(&self) -> Vec<ColumnType> {
        self.columns.clone().into_iter().filter_map(|c| c).collect()
    }

    fn read_columns(&mut self) {
        let header = unsafe{self.header()};
        assert_eq!(header.has_set_column, u32::max_value());
        assert_eq!(header.column_num, used_num(header.column_slot, MAX_COLUMN_NUMBER));

        for i in 0..MAX_COLUMN_NUMBER {
            println!("{}", i);
            let header = unsafe{self.header()};
            if is_free(header.column_slot, i) {
                continue;
            }
            let ci = &header.column_info[i];
            let data_type = match ci.data_type {
                INT_TYPE => {
                    if ci.has_default && !ci.default_null {
                        Type::Int(Some(unsafe{ci.default_value.int}))
                    } else {
                        Type::Int(None)
                    }
                },
                FLOAT_TYPE => {
                    if ci.has_default && !ci.default_null {
                        Type::Float(Some(unsafe{ci.default_value.float}))
                    } else {
                        Type::Float(None)
                    }
                },
                DATE_TYPE => {
                    if ci.has_default && !ci.default_null {
                        Type::Date(Some(unsafe{ci.default_value.date}))
                    } else {
                        Type::Date(None)
                    }
                },
                STR_TYPE => {
                    if ci.has_default && !ci.default_null {
                        Type::Str(ci.max_len, Some(self.get_string(unsafe{ &ci.default_value.strp })))
                    } else {
                        Type::Str(ci.max_len, None)
                    }
                },
                _ => {
                    unreachable!();
                }
            };

            let header = unsafe{self.header()};
            let ci = &header.column_info[i];

            let column_type = ColumnType {
                name: from_bytes(&ci.column_name, ci.column_name_len as usize),
                index: i as u32,
                data_type: data_type,
                can_be_null: ci.can_be_null,
                is_primary: ci.is_primary,
                has_index: ci.has_index,
                default_null: ci.default_null,
                has_default: ci.has_default,
                is_foreign: ci.is_foreign,
                foreign_table_name: from_bytes(&ci.foreign_table_name, ci.foreign_table_name_len as usize),
            };

            self.columns[i] = Some(column_type);
        }
    }

    fn check_column(&self, c: &ColumnData) {
        assert!(self.columns[c.index as usize].is_some());
        if c.default {
            assert!(self.columns[c.index as usize].as_ref().unwrap().has_default);
        }
    }

    fn get_insert_data(&self, c: &ColumnData) -> InsertData {
        let ct = self.columns[c.index as usize].as_ref().unwrap();

        let mut id = InsertData {
            index: ct.index,
            is_null: false,
            data: MemData{int: 0},
        };

        if c.default {
            if ct.default_null {
                if ct.can_be_null {
                    id.is_null = true;
                } else {
                    unreachable!();
                }
            } else {
                match ct.data_type {
                    Type::Int(Some(x)) => {
                        id.data.int = x;
                    },

                    Type::Float(Some(x)) => {
                        id.data.float = x;
                    },
                    Type::Date(Some(x)) => {
                        id.data.date = x;
                    },
                    Type::Str(_, Some(ref x)) => {
                        id.data.strp = self.alloc_string(x);
                    },
                    _ => {
                        unreachable!();
                    },
                }
            }
        } else {
            if let Some(ref d) = c.data {
                match d {
                    &Data::Int(x) => {
                        id.data.int = x;
                    },
                    &Data::Float(x) => {
                        id.data.float = x;
                    }
                    &Data::Date(x) => {
                        id.data.date = x;
                    },
                    &Data::Str(ref x) => {
                        id.data.strp = self.alloc_string(x);
                    }
                }
            } else {
                if ct.can_be_null {
                    id.is_null = true;
                } else {
                    unreachable!();
                }
            }
        }

        id
    }

    pub fn create_record(&self, r: &Record) -> u32 {
        let record = &r.record;
        let mut v: Vec<InsertData> = Vec::new();
        for c in record {
            self.check_column(c);
        }
        for c in record {
            v.push(self.get_insert_data(c));
        }

        let (page, offset) = self.alloc_record_slot();
        let rid = Self::get_rid(page, offset as u32);

        let rp = unsafe{self.rp_mut(page)};
        let mut mem_record = &mut rp.record[offset as usize];

        mem_record.rid = rid;
        mem_record.is_null = 0;
        for id in &v {
            self.write_record_column(id, &mut mem_record);
        }

        rid
    }

    fn get_rid(page: u32, offset: u32) -> u32 {
        (page - 1) * MAX_PAGE_RECORD_NUMBER as u32 + offset as u32
    }

    fn get_pos(rid: u32) -> (u32, u32) {
        (rid / MAX_PAGE_RECORD_NUMBER as u32 + 1, rid % MAX_PAGE_RECORD_NUMBER as u32)
    }

    fn write_record_column(&self, id: &InsertData, mem_record: &mut MemRecord) {
        let i = id.index;
        if id.is_null {
            set_used(&mut mem_record.is_null, i);
        }
        mem_record.data[i as usize] = id.data;
    }

    fn read_record_column(&self, is_null: bool, mem_data: MemData, column_type: &ColumnType) -> ColumnData {
        let mut cd = ColumnData {
            index: column_type.index,
            default: false,
            data: None,
        };
        if !is_null {
            match &column_type.data_type {
                &Type::Str(_, _) => {
                    cd.data = Some(Data::Str(self.get_string(&unsafe{mem_data.strp})));
                },
                &Type::Int(_) => {
                    cd.data = Some(Data::Int(unsafe{mem_data.int}));
                },
                &Type::Float(_) => {
                    cd.data = Some(Data::Float(unsafe{mem_data.float}));
                },
                &Type::Date(_) => {
                    cd.data = Some(Data::Date(unsafe{mem_data.date}));
                },
            }
        }

        cd
    }

    pub fn get_record(&self, rid: u32) -> Record {
        let (page, offset) = Self::get_pos(rid);
        let mut record: Vec<ColumnData> = Vec::new();

        for c in self.columns.iter().filter_map(|c| c.as_ref()) {
            let rp = unsafe{self.rp(page)};
            let mem_record = &rp.record[offset as usize];
            let i = c.index as usize;
            record.push(self.read_record_column(is_one(mem_record.is_null, i), mem_record.data[i], &c));
        }

        Record {
            record: record,
        }
    }

    pub fn delete_record(&self, rid: u32) {
        let (page, offset) = Self::get_pos(rid);
        let rp = unsafe{self.rp_mut(page)};
        set_free(&mut rp.header.free_slot, offset);

        for c in self.columns.iter().filter_map(|c| c.as_ref()) {
            let rp = unsafe{self.rp_mut(page)};
            match &c.data_type {
                &Type::Str(_, _) => {
                    self.free_string(&mut unsafe{rp.record[offset as usize].data[c.index as usize].strp});
                },
                _ => {},
            }
        }
        

        if free_num(rp.header.free_slot, MAX_PAGE_RECORD_NUMBER) == 1 {
            let header = unsafe{self.header_mut()};
            let stack_top = header.free_page[0];
            header.free_page[0] = page;
            unsafe{self.rp_mut(page)}.header.next_free_page = stack_top;
        }
    }

    pub fn update_record(&self, rid: u32, column_data: &ColumnData) {
        let (page, offset) = Self::get_pos(rid);
        let id = self.get_insert_data(column_data);
        let rp = unsafe{self.rp_mut(page)};
        self.write_record_column(&id, &mut rp.record[offset as usize]);
    }


    unsafe fn header_mut<'b>(&self) -> &'b mut FileHeader { self.page_mut(0, true) }
    unsafe fn header<'b>(&self) -> &'b FileHeader { self.page_mut(0, false) } 
    unsafe fn sp_mut<'b>(&self, page_id: u32) -> &'b mut StringPage { self.page_mut(page_id, true) }
    unsafe fn sp<'b>(&self, page_id: u32) -> &'b StringPage { self.page_mut(page_id, false) }
    unsafe fn rp_mut<'b>(&self, page_id: u32) -> &'b mut RecordPage { self.page_mut(page_id, true) }
    unsafe fn rp<'b>(&self, page_id: u32) -> &'b RecordPage { self.page_mut(page_id, false) }
    unsafe fn ph_mut<'b>(&self, page_id: u32) -> &'b mut PageHeader { self.page_mut(page_id, false) }
    unsafe fn ph<'b>(&self, page_id: u32) -> &'b PageHeader { self.page_mut(page_id, false) }
}


#[test]
#[warn(exceeding_bitshifts)]
fn bit_op_test() {
    assert_eq!(get_free_index(0xFFu32), 8);
    assert!(all_used(0xFFFFFFFFu32, MAX_FIXED_STRING_NUMBER));
    assert!(!all_used(0xFFFFFF8Fu32, MAX_FIXED_STRING_NUMBER));

    let mut a = 0x1u32;
    set_used(&mut a, 1);
    assert_eq!(a, 0x3u32);
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
