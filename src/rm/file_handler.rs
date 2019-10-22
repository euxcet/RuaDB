use std::rc::Rc;
use std::cell::RefCell;

use super::record::*;
use super::filesystem::bufmanager::buf_page_manager::BufPageManager;

pub struct FileHandler<'a>{
    file_id: i32,

    first_page: &'a [u8],
    first_page_index: i32,

    stack_top_page_id: i32,
    column_type: Vec<ColumnType>,

    bpm: Option<Rc<RefCell<BufPageManager>>>,
}

pub struct FileHeader {
    first_page_index: u32,
    column_num: u32,
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

impl<'a> FileHandler<'a> {
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

    pub fn new() {

    }

    pub fn close() {

    }
}
