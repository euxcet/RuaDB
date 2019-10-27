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

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct StrPointer {
    pub page: u32,
    pub len: u16,
    pub offset: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union MemData {
    pub strp: StrPointer,
    pub int: i64,
    pub float: f64,
    pub date: u64,
}

impl std::fmt::Debug for MemData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemData( int: {} )", unsafe{self.int})
    }

}

impl Default for MemData {
    fn default() -> Self {
        Self { int: 0 }
    }
}

#[repr(C)]
pub struct PageHeader {
    pub next_free_page: u32,
    pub usage: u32, 
    pub free_slot: u32,
}

#[repr(C)]
pub struct MemRecord {
    pub rid: u32,
    pub is_null: u32,
    pub is_default: u32,
    pub data: [MemData; MAX_COLUMN_NUMBER],
}

#[repr(C)]
pub struct RecordPage {
    pub header: PageHeader,
    pub record: [MemRecord; MAX_PAGE_RECORD_NUMBER],
}

#[repr(C)]
pub struct StringSlice {
    pub bytes: [u8; MAX_FIXED_STRING_LENGTH],
    pub next: StrPointer,
}

#[repr(C)]
pub struct StringPage {
    pub header: PageHeader,
    pub strs: [StringSlice; MAX_FIXED_STRING_NUMBER],
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct FileHeader {
    pub has_used: u32,
    pub has_set_column: u32,

    // pub free_record_page: u32,
    // pub free_string_page: u32,
    pub free_page: [u32;2],

    pub least_unused_page: u32,

    pub column_num: u32,
    pub column_slot: u32,
    pub column_info: [MemColumnInfo; MAX_COLUMN_NUMBER],
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct MemColumnInfo {
    pub column_name_len: u8,
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
    pub foreign_table_name_len: u8,
}

#[derive(PartialEq, Debug)]
pub enum Data {
    Str(String),
    Int(i64),
    Float(f64),
    Date(u64),
} 

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Str(u32, Option<String>),
    Int(Option<i64>),
    Float(Option<f64>),
    Date(Option<u64>),
}

impl Default for Type {
    fn default() -> Self {
        Self::Int(None)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnType {
    pub name: String,
    pub index: u32,
    pub data_type: Type,
    pub can_be_null: bool,
    pub has_index: bool,
    pub has_default: bool,
    pub is_primary: bool,
    pub is_foreign: bool,
    pub default_null: bool,
    pub foreign_table_name: String,
}

#[derive(PartialEq, Debug)]
pub struct ColumnData {
    pub index: u32,
    pub default: bool,
    pub data: Option<Data>,
}

#[derive(PartialEq, Debug)]
pub struct Record {
    pub record: Vec<ColumnData>,
}

#[derive(Debug)]
pub struct InsertData {
    pub index: u32,
    pub is_null: bool,
    pub data: MemData,
}

pub unsafe fn to_slice_mut<'a>(data: *mut u8, len: usize) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(data, len)
}

pub unsafe fn to_slice<'a>(data: *const u8, len: usize) -> &'a [u8] {
    std::slice::from_raw_parts(data, len)
}

pub unsafe fn convert<'a, T>(data:*mut u8) -> &'a mut T {
    // (data as *mut T).as_mut().unwrap()
    transmute(data)
}

#[test]
fn check_size() {
    println!("size of <RecordPage> {}", size_of::<RecordPage>());
    println!("size of <PageHeader> {}", size_of::<PageHeader>());
    println!("size of <MemRecord> {}", size_of::<MemRecord>());
    println!("size of <MemData> {}", size_of::<MemData>());
    println!("size of <StrPointer> {}", size_of::<StrPointer>());
    assert!(size_of::<FileHeader>() <= PAGE_SIZE);
    assert!(size_of::<StringPage>() <= PAGE_SIZE);
    assert!(size_of::<RecordPage>() <= PAGE_SIZE);
}

