use std::vec;
use std::ptr;
use std::mem;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use crate::rm::record::*;
use crate::rm::pagedef::StrPointer;
use crate::rm::table_handler::TableHandler;

#[derive(Clone, fmt::Debug)]
pub struct Index<'a> {
    th: &'a TableHandler,
    index_type: Vec<u8>,
    index: Vec<u64>,
}

impl PartialOrd for Index<'_> {
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        if self.index.len() != other.index.len() {
            None
        }
        else {
            for i in 0..self.index.len() {
                if self.index_type[i] != other.index_type[i] {
                    return None;
                }
            }
            for i in 0..self.index.len() {
                if self.index_type[i] > 0 { // Not a String
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
                if self.index_type[i] != other.index_type[i] {
                    return false;
                }
            }
            for i in 0..self.index.len() {
                if self.index_type[i] > 0 { // Not a String
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
}

pub enum BTreeNodeType {
    Internal,
    Leaf,
}

pub struct Bucket {
    data: Vec<Data>,
}

pub struct BTreeNode<'a> {
    pub th: &'a TableHandler,
    pub ty: BTreeNodeType,
    pub key: Vec<Index<'a>>,
    pub son: Vec<StrPointer>,
    pub bucket: Vec<Bucket>,
    pub father: Option<Rc<RefCell<BTreeNode<'a>>>>,
}


impl<'a> BTreeNode<'a> {
    pub fn new(th: &'a TableHandler) -> Self {
        BTreeNode {
            th: th,
            ty: BTreeNodeType::Leaf,
            key: Vec::new(),
            son: Vec::new(),
            bucket: Vec::new(),
            father: None,
        }
    }
}