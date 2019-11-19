use std::mem::transmute;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::Ordering;
use crate::rm::record::*;
use crate::rm::pagedef::StrPointer;
use crate::rm::table_handler::TableHandler;

#[derive(Clone, Debug)]
pub struct Index<'a> {
    pub th: &'a TableHandler,
    /*
        flags [0 .. 8]
        [data_type_bit0, data_type_bit1, data_byte_bit2, 0, 0, 0, 0, 0]
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

impl Index<'_> {
    pub fn from<'a>(th: &'a TableHandler, raw: &RawIndex) -> Index<'a> {
        let mut index_flags: Vec<u8> = Vec::new();
        let mut index: Vec<u64> = Vec::new();
        for data in &raw.index {
            match data {
                Data::Str(d) => {
                    index_flags.push(0);
                    index.push(th.insert_string(d).to_u64());
                },
                Data::Int(d) => {
                    index_flags.push(1);
                    index.push(unsafe{transmute(*d)});
                },
                Data::Float(d) => {
                    index_flags.push(2);
                    index.push(unsafe{transmute(*d)});
                },
                Data::Date(d) => {
                    index_flags.push(3);
                    index.push(unsafe{transmute(*d)});
                },
                _ => unreachable!(),
            }
        }
        Index {
            th: th,
            index_flags: index_flags,
            index: index,
        }
    }
}

#[derive(Debug)]
pub struct RawIndex {
    pub index: Vec<Data>,
}

impl RawIndex {
    pub fn from(index: &Index) -> Self {
        let mut data = Vec::new();
        for i in 0..index.index.len() {
            data.push(match index.index_flags[i] {
                0 => Data::Str(index.th.get_string(&StrPointer::new(index.index[i]))), 
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

impl PartialOrd for RawIndex {
    fn partial_cmp(&self, other: &RawIndex) -> Option<Ordering> {
        if self.index.len() != other.index.len() {
            None
        }
        else {
            for i in 0..self.index.len() {
                let res = self.index[i].partial_cmp(&other.index[i]);
                if res != Some(Ordering::Equal) {
                    return res;
                }
            }
            Some(Ordering::Equal)
        }
    }
}

impl PartialEq for RawIndex {
    fn eq(&self, other: &RawIndex) -> bool {
        if self.index.len() != other.index.len() {
            false
        }
        else {
            for i in 0..self.index.len() {
                if self.index[i] != other.index[i] {
                    return false;
                }
            }
            true
        }

    }
}

pub struct BTree<'a> {
    pub th: &'a TableHandler,
    pub root: u64,
    pub node_capacity: u32,
    pub index_col: Vec<u32>, // should be orderly
}

impl<'a> BTree<'a> {
    pub fn new(th: &'a TableHandler, node_capacity: u32, index_col: Vec<u32>) -> Self {
        Self {
            th: th,
            root: th.insert_btree_node(&BTreeNode::new(th)).to_u64(),
            node_capacity: node_capacity,
            index_col: index_col,
        }
    }

    pub fn insert_record(&mut self, key: &RawIndex, data: u64) {
        let mut root = self.th.get_btree_node(&StrPointer::new(self.root));
        self.root = root.insert(key, data, self.node_capacity as usize, None, self.root);
    }

    pub fn delete_record(&mut self, key: &RawIndex, data: u64) {
        let mut root = self.th.get_btree_node(&StrPointer::new(self.root));
        root.delete(key, data);
    }

    pub fn search_record(&mut self, key: &RawIndex) -> Option<Bucket> {
        let root = self.th.get_btree_node(&StrPointer::new(self.root));
        root.search(key)
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

impl Bucket {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }
}

pub struct BTreeNode<'a> {
    pub th: &'a TableHandler,
    pub ty: BTreeNodeType,
    pub key: Vec<u64>, // Vec<Index>
    pub son: Vec<u64>, // Vec<BTreeNode>
    pub bucket: Vec<u64>, // Vec<Bucket>
}

impl<'a> BTreeNode<'a> {
    pub fn new(th: &'a TableHandler) -> Self {
        BTreeNode {
            th: th,
            ty: BTreeNodeType::Leaf,
            key: Vec::new(),
            son: Vec::new(),
            bucket: Vec::new(),
        }
    }
    
    pub fn to_raw(&self, key: u64) -> RawIndex {
        RawIndex::from(&self.th.get_index(&StrPointer::new(key)))
    }

    pub fn split_up(&mut self, node_capacity: usize, father: &mut BTreeNode, self_ptr: &mut u64) {
        let mid = node_capacity / 2;
        let ptr = *self_ptr;
        match self.ty {
            BTreeNodeType::Leaf => {
                let mut new_node = BTreeNode::new(self.th);
                let mid_key = self.key[mid];
                /*
                println!("key {:?} {}", self.key, mid_key);
                println!("bucket {:?}", self.bucket);
                */
                new_node.key = self.key.split_off(mid);
                new_node.bucket = self.bucket.split_off(mid);
                /*
                println!("key {:?}", self.key);
                */

                let new_node_ptr = self.th.insert_btree_node(&new_node).to_u64();
                *self_ptr = self.th.insert_btree_node(self).to_u64();

                for i in 0..father.son.len() {
                    if father.son[i] == ptr {
                        father.key.insert(i, mid_key);
                        father.son.insert(i + 1, new_node_ptr);
                        father.son[i] = *self_ptr;
                        break;
                    }
                }
            }
            BTreeNodeType::Internal => {
                let mut new_node = BTreeNode::new(self.th);
                new_node.ty = BTreeNodeType::Internal;
                new_node.key = self.key.split_off(mid + 1);
                new_node.son = self.son.split_off(mid + 1);
                let new_node_ptr = self.th.insert_btree_node(&new_node).to_u64();
                let mid_key = self.key.pop();

                *self_ptr = self.th.insert_btree_node(self).to_u64();
                
                for i in 0..father.son.len() {
                    if father.son[i] == ptr {
                        father.key.insert(i, mid_key.unwrap());
                        father.son.insert(i + 1, new_node_ptr);
                        father.son[i] = *self_ptr;
                        break;
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, key: &RawIndex, data: u64, node_capacity: usize, father: Option<(&mut BTreeNode, &mut u64)>, self_ptr: u64) -> u64 {
        /*
        println!("{:?}   {:?}", self.ty, key);
        for i in 0..self.key.len() {
            let index_in_node = RawIndex::from(&self.th.get_index(&StrPointer::new(self.key[i])));
            println!("    index  {:?}", index_in_node);
        }
        println!("{:?}", self.key);
        println!("{:?}", self.bucket);
        println!("{:?}", self.son);
        println!("");
        println!("");
        */

        let mut self_ptr = self_ptr;
        match self.ty {
            BTreeNodeType::Leaf => {
                let key_ptr = self.th.insert_index(&Index::from(self.th, key)).to_u64();
                for i in 0..=self.key.len() {
                    if i == self.key.len() {
                        self.key.push(key_ptr);
                        let mut bucket = Bucket::new();
                        bucket.data.push(data);
                        self.bucket.push(self.th.insert_bucket(&bucket).to_u64());
                        break;
                    }
                    let index_in_node = RawIndex::from(&self.th.get_index(&StrPointer::new(self.key[i])));
                    if let Some(cmp) = index_in_node.partial_cmp(key) {
                        match cmp {
                            Ordering::Equal => {
                                let mut ptr = StrPointer::new(self.bucket[i]);
                                let mut bucket = self.th.get_bucket(&ptr);
                                bucket.data.push(data);
                                self.th.update_bucket(&mut ptr, &bucket);
                                self.bucket[i] = ptr.to_u64();
                                break;
                            },
                            Ordering::Greater => {
                                self.key.insert(i, key_ptr);
                                let mut bucket = Bucket::new();
                                bucket.data.push(data);
                                self.bucket.insert(i, self.th.insert_bucket(&bucket).to_u64());
                                break;
                            },
                            _ => {}
                        }
                    }
                }
                /*
                if self.key.len() <= node_capacity {
                    self.th.update_btree_node_(&mut self_ptr, self);
                }
                */
            }
            BTreeNodeType::Internal => {
                for i in 0..=self.key.len() {
                    if i == self.key.len() || self.to_raw(self.key[i]) > *key {
                        let mut son_node = self.th.get_btree_node(&StrPointer::new(self.son[i]));
                        let son_ptr = self.son[i];
                        self.son[i] = son_node.insert(key, data, node_capacity, Some((self, &mut self_ptr)), son_ptr);
                        break;
                    }
                }
            }
        }
        
        // split
        if self.key.len() > node_capacity {
            match father {
                Some(father) => {
                    self.split_up(node_capacity, father.0, &mut self_ptr);
                }
                None => {
                    let mut new_root = BTreeNode::new(self.th);
                    new_root.ty = BTreeNodeType::Internal;
                    new_root.son.push(self_ptr);
                    self.split_up(node_capacity, &mut new_root, &mut self_ptr);
                    self_ptr = self.th.insert_btree_node(&new_root).to_u64();
                }
            }
        }
        else {
            self.th.update_btree_node_(&mut self_ptr, self);
        }

        self_ptr
    }

    pub fn delete(&mut self, key: &RawIndex, data: u64) {
        unimplemented!();
    }
    
    pub fn search(&self, key: &RawIndex) -> Option<Bucket> {
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..self.key.len() {
                    if self.to_raw(self.key[i]) == *key {
                        return Some(self.th.get_bucket(&StrPointer::new(self.bucket[i])));
                    }
                }
            }
            BTreeNodeType::Internal => {
                for i in 0..=self.key.len() {
                    if i == self.key.len() || self.to_raw(self.key[i]) > *key {
                        return self.th.get_btree_node(&StrPointer::new(self.son[i])).search(key);
                    }
                }
            }
        }
        None
    }
}