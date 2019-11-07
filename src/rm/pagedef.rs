use std::cmp::Ordering;

pub const MAX_FIXED_STRING_NUMBER: usize = 30;
pub const MAX_FIXED_STRING_LENGTH: usize = 256;

pub const PAGE_SIZE: usize = 8192;
pub const PAGE_SIZE_IDX: i32 = 13;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct StrPointer {
    pub page: u32,
    pub len: u16,
    pub offset: u16,
}

impl StrPointer {
    pub fn new(p: u64) -> Self {
        StrPointer {
            page: (p & 0xffffffff) as u32,
            len: ((p >> 32) & 0xffff) as u16,
            offset: ((p >> 48) & 0xffff) as u16,
        }
    }

    pub fn to_u64(&self) -> u64 {
        (self.offset as u64) << 48 | (self.len as u64) << 32 | (self.page as u64)
    }
}

#[repr(C)]
pub struct PageHeader {
    pub next_free_page: u32,
    pub free_slot: u32,
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
    pub free_page: u32,
    pub least_unused_page: u32,
}

