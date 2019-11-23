use std::mem::size_of;

pub const SLOT_PER_PAGE: usize = 30;
pub const SLOT_LENGTH: usize = 256;

pub const PAGE_SIZE: usize = 8192;
pub const PAGE_SIZE_IDX: i32 = 13;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct StrPointer {
    pub page: u32,
    pub offset: u32,
}

impl StrPointer {
    pub fn new(p: u64) -> Self {
        StrPointer {
            page: ((p >> 32) & 0xffffffff) as u32,
            offset: (p & 0xffffffff) as u32,
        }
    }

    pub fn to_u64(&self) -> u64 {
        ((self.page as u64) << 32) | (self.offset as u64)
    }

    pub fn is_null(&self) -> bool {
        self.page == 0
    }
}

#[repr(C)]
pub struct PageHeader {
    pub next_free_page: u32,
    pub free_slot: u32,
}

#[repr(C)]
pub struct StringSlice {
    pub len: u64,
    pub bytes: [u8; SLOT_LENGTH],
    pub next: StrPointer,
}

#[repr(C)]
pub struct StringPage {
    pub header: PageHeader,
    pub strs: [StringSlice; SLOT_PER_PAGE],
}

#[derive(Default, Debug)]
#[repr(C)]
pub struct FileHeader {
    pub has_used: u32,
    pub free_page: u32,
    pub least_unused_page: u32,
}


#[cfg(test)]
mod tests {
    use crate::rm::pagedef::*;

    #[test]
    fn page_size() {
        assert!(size_of::<StringPage>() <= PAGE_SIZE);
        assert!(size_of::<FileHeader>() <= PAGE_SIZE);
    }
}