use std::mem::{transmute, size_of};
use super::pagedef::*;

pub const MAX_COLUMN_NUMBER: usize = 32;
pub const MAX_PAGE_RECORD_NUMBER: usize = 24;
pub const MAX_FIXED_STRING_NUMBER: usize = 30;
pub const MAX_FIXED_STRING_LENGTH: usize = 256;
pub const MAX_COLUMN_NAME_LENGTH: usize = 32;
pub const INT_TYPE: u8 = 1;
pub const FLOAT_TYPE: u8 = 2;
pub const DATE_TYPE: u8 = 3;
pub const STR_TYPE: u8 = 4;

pub const USAGE_RECORD: u32 = 1;
pub const USAGE_STR: u32 = 2;

#[derive(Copy, Clone)]
pub struct StrPointer {
    pub len: u16,
    pub page: u32,
    pub offset: u16,
}

pub union MemData {
    pub strp: StrPointer,
    pub int: i64,
    pub float: f64,
    pub date: u64,
}

impl Default for MemData {
    fn default() -> Self {
        Self { int: 0 }
    }
}

pub struct PageHeader {
    pub next_free_page: u32,
    pub usage: u32, // 0 for record, 1 for string <= 256, 2 for string > 256
}

pub struct MemRecord {
    pub rid: u32,
    pub is_null: u32,
    pub data: [MemData; MAX_COLUMN_NUMBER],
}

pub struct RecordPage {
    pub header: PageHeader,
    pub free_slot: u32, // 0 for free, 1 for use
    pub record: [MemRecord; MAX_PAGE_RECORD_NUMBER],
}

pub struct StringSlice {
    pub bytes: [u8; MAX_FIXED_STRING_LENGTH],
    pub next: StrPointer,
}

pub struct StringPage {
    pub header: PageHeader,
    pub free_slot: u32, // 0 for free, 1 for use
    pub strs: [StringSlice; MAX_FIXED_STRING_NUMBER],
}

#[derive(Default)]
pub struct FileHeader {
    pub has_used: i32,

    pub free_record_page: u32,
    pub free_string_page: u32,

    pub least_unused_page: u32,

    pub column_num: u32,
    pub column_info: [MemColumnInfo; MAX_COLUMN_NUMBER],
}

#[derive(Default)]
pub struct MemColumnInfo {
    pub column_name: [u8; MAX_COLUMN_NAME_LENGTH],
    pub data_type: u8,
    pub max_len: u32,
    pub can_be_null: bool, 
    pub is_primary: bool,
    pub has_index: bool,
    pub has_default: bool,
    pub default_null: bool,
    pub default_value: MemData,
    pub is_foreign: bool, 
    pub foreign_table_name: [u8; MAX_COLUMN_NAME_LENGTH],
}


pub enum Data {
    Str(Option<String>),
    Int(Option<i64>),
    Float(Option<f64>),
    Date(Option<u64>),
} 

pub enum Type {
    Str(u32, Option<String>),
    Int(Option<i64>),
    Float(Option<f64>),
    Date(Option<u64>),
    // Str(bool, String, u32),
    // Int(bool, i64),
    // Float(bool, f64),
    // Date(bool, u64),
}

pub struct ColumnType {
    pub name: String,
    pub data_type: Type,
    pub can_be_null: bool,
    pub is_primary: bool,
    pub has_index: bool,
    pub has_default: bool,
    pub is_foreign: bool,
    pub foreign_table_name: String,
}

pub struct ColumnData {
    name: String,
    data: Data,
}

pub struct Record {
    record: Vec<ColumnData>,
}

pub unsafe fn to_slice_mut<'a>(data: *mut u8, len: usize) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(data, len)
}

pub unsafe fn to_slice<'a>(data: *const u8, len: usize) -> &'a [u8] {
    std::slice::from_raw_parts(data, len)
}

pub unsafe fn convert<'a, T>(data:*mut u8) -> &'a mut T {
    (data as *mut T).as_mut().unwrap()
}

#[test]
fn check_size() {
    assert!(size_of::<FileHeader>() <= PAGE_SIZE);
    assert!(size_of::<StringPage>() <= PAGE_SIZE);
    assert!(size_of::<RecordPage>() <= PAGE_SIZE);
}

