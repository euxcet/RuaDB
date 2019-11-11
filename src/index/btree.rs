use std::mem::transmute;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use crate::rm::record::*;
use crate::rm::pagedef::StrPointer;
use crate::rm::table_handler::TableHandler;

#[derive(Clone, fmt::Debug)]
pub struct Index<'a> {
    pub th: &'a TableHandler,
    /*
        flags [0 .. 8]
        [default, is_null, data_type_bit0, data_type_bit1, data_byte_bit2, 0, 0, 0]
        bit meaning
        0   Data::Str
        1   Data::Int
        2   Data::Float
        3   Data::Date
        4   Data::Numeric
    */
    pub index_flags: Vec<u8>,
    pub index: Vec<u64>,
}

/*
impl Index<'_> {
    pub fn from(record: &Record, index_col: &Vec<u32>) -> Self {
        unimplemented!();
    }
}
*/

pub struct RawIndex {
    pub index: Vec<Data>,
}

impl RawIndex {
    pub fn from(index: &Index) -> Self {
        let mut data = Vec::new();
        for i in 0..index.index.len() {
            data.push(match index.index_flags[i] {
                0 => unimplemented!(), 
                1 => Data::Int(unsafe{transmute(index.index[i])}),
                2 => Data::Float(unsafe{transmute(index.index[i])}),
                3 => Data::Date(unsafe{transmute(index.index[i])}),
                4 => unimplemented!(),
                _ => unreachable!(),
            });
        }
        Self {
            index: data,
        }
    }
}

impl PartialOrd for Index<'_> {
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        if self.index.len() != other.index.len() {
            None
        }
        else {
            for i in 0..self.index.len() {
                if self.index_flags[i] != other.index_flags[i] {
                    return None;
                }
            }
            for i in 0..self.index.len() {
                if ((self.index_flags[i] >> 2) & 7) > 0 { // Not a String
                    let res = self.index[i].partial_cmp(&other.index[i]);
                    if res != Some(Ordering::Equal) {
                        return res;
                    }
                }
                else { // String
                    let self_s = self.th.get_string(&StrPointer::new(self.index[i]));
                    let other_s = self.th.get_string(&StrPointer::new(other.index[i]));
                    let res = self_s.partial_cmp(&other_s);
                    if res != Some(Ordering::Equal) {
                        return res;
                    }
                }
            }
            Some(Ordering::Equal)
        }
    }
}

impl PartialEq for Index<'_> {
    fn eq(&self, other: &Index) -> bool {
        if self.index.len() != other.index.len() {
            false
        }
        else {
            for i in 0..self.index.len() {
                if self.index_flags[i] != other.index_flags[i] {
                    return false;
                }
            }
            for i in 0..self.index.len() {
                if ((self.index_flags[i] >> 2) & 7) > 0 { // Not a String
                    if self.index[i] != other.index[i] {
                        return false;
                    }
                }
                else { // String
                    if self.th.get_string(&StrPointer::new(self.index[i])) != self.th.get_string(&StrPointer::new(other.index[i])) {
                        return false;
                    }
                }
            }
            true
        }
    }
}

pub struct BTree<'a> {
    pub root: Rc<RefCell<BTreeNode<'a>>>,
    pub node_capacity: u32,
    pub index_col: Vec<u32>, // should be orderly
}

impl<'a> BTree<'a> {
    pub fn new(th: &'a TableHandler, node_capacity: u32, index_col: Vec<u32>) -> Self {
        Self {
            root: Rc::new(RefCell::new(BTreeNode::new(th))),
            node_capacity: node_capacity,
            index_col: index_col,
        }
    }

    pub fn insert_record(&mut self, key: &RawIndex, data: u64) {
        self.root.borrow_mut().insert(key, data);
    }

    pub fn delete_record(&mut self, key: &RawIndex, data: u64) {
        self.root.borrow_mut().delete(key, data);
    }

    pub fn search_record(&mut self, key: &RawIndex) -> Option<Bucket> {
        self.root.borrow().search(key)
    }
}

#[derive(Debug)]
pub enum BTreeNodeType {
    Internal,
    Leaf,
}

#[derive(Debug)]
pub struct Bucket {
    pub data: Vec<u64>,
}

pub struct BTreeNode<'a> {
    pub th: &'a TableHandler,
    pub ty: BTreeNodeType,
    pub key: Vec<u64>, // Vec<Index>
    pub son: Vec<u64>, // Vec<BTreeNode>
    pub bucket: Vec<u64>, // Vec<Bucket>
    // pub father: Option<Rc<RefCell<BTreeNode<'a>>>>,
}


impl<'a> BTreeNode<'a> {
    pub fn new(th: &'a TableHandler) -> Self {
        BTreeNode {
            th: th,
            ty: BTreeNodeType::Leaf,
            key: Vec::new(),
            son: Vec::new(),
            bucket: Vec::new(),
            // father: None,
        }
    }

    pub fn insert(&mut self, key: &RawIndex, data: u64) {
        unimplemented!();
    }

    pub fn delete(&mut self, key: &RawIndex, data: u64) {
        unimplemented!();
    }
    
    pub fn search(&self, key: &RawIndex) -> Option<Bucket> {
        unimplemented!();
    }
}