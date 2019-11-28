use std::mem::transmute;
use std::cmp::Ordering;
use crate::rm::record::*;
use crate::rm::table_handler::TableHandler;
use crate::utils::convert;

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
                Data::Numeric(d) => {
                    index_flags.push(4);
                    index.push(unsafe{transmute(*d)});
                }
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
                0 => Data::Str(index.th.get_string_(index.index[i])), 
                1 => Data::Int(unsafe{transmute(index.index[i])}),
                2 => Data::Float(unsafe{transmute(index.index[i])}),
                3 => Data::Date(unsafe{transmute(index.index[i])}),
                4 => Data::Numeric(unsafe{transmute(index.index[i])}),
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
            root: th.insert_btree_node(&BTreeNode::new(th), node_capacity as usize).to_u64(),
            node_capacity: node_capacity,
            index_col: index_col,
        }
    }

    // offset
    pub fn get_offset_root() -> usize {
        4 * 3 // [root node_capacity index_col]
    }

    pub fn get_offset_node_capacity() -> usize { // should not be used
        4 * 3 // [root node_capacity index_col]
        + 8 // root
    }

    pub fn get_index_col() -> usize { // should not be used
        4 * 3 // [root node_capacity index_col]
        + 8 // root
        + 4 // node_capacity
    }

    pub fn insert_record(&mut self, key: &RawIndex, data: u64) {
        let mut root = self.th.get_btree_node_(self.root);
        self.root = root.insert(key, data, self.node_capacity as usize, None, 0, self.root);
    }

    pub fn delete_record(&mut self, key: &RawIndex, data: u64) {
        let mut root = self.th.get_btree_node_(self.root);
        self.root = root.delete(key, data, self.node_capacity as usize, None, 0, self.root);
    }

    pub fn search_record(&self, key: &RawIndex) -> Option<Bucket> {
        let root = self.th.get_btree_node_(self.root);
        root.search(key)
    }

    pub fn traverse(&self) {
        let root = self.th.get_btree_node_(self.root);
        root.traverse(self.root);
    }

    pub fn first_bucket(&self) -> Option<Bucket> {
        let root = self.th.get_btree_node_(self.root);
        root.first_bucket()
    }

    pub fn last_bucket(&self) -> Option<Bucket> {
        let root = self.th.get_btree_node_(self.root);
        root.last_bucket()
    }

    pub fn get_height(&self) -> usize {
        let root = self.th.get_btree_node_(self.root);
        root.get_height()
    }
}

#[derive(Debug)]
pub enum BTreeNodeType {
    Internal,
    Leaf,
}

#[derive(Debug)]
pub struct Bucket {
    pub prev: u64,
    pub next: u64,
    pub data: Vec<u64>,
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            prev: 0,
            next: 0,
            data: Vec::new(),
        }
    }

    // offset
    pub fn get_offset_prev() -> usize {
        4 * 3 // [prev next data]
    }

    pub fn get_offset_next() -> usize {
        4 * 3 // [prev next data]
        + 8 // prev
    }

    pub fn get_offset_data(pos: usize) -> usize {
        4 * 3 // [prev next data]
        + 8 // prev
        + 8 // next
        + pos * 8
    }

    pub fn prev_bucket(&self, th: &TableHandler) -> Option<Bucket> {
        if self.prev == 0 {
            None
        }
        else {
            Some(th.get_bucket_(self.prev))
        }
    }

    pub fn next_bucket(&self, th: &TableHandler) -> Option<Bucket> {
        if self.next == 0 {
            None
        }
        else {
            Some(th.get_bucket_(self.next))
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

    // offset
    pub fn get_offset_ty() -> usize {
        4 * 3 // [flags key next]
    }

    pub fn get_offset_key(pos: usize) -> usize {
        4 * 3 // [flags key next]
        + 1 // flags
        + pos * 8 // key
    }

    pub fn get_offset_son(pos: usize, node_capacity: usize) -> usize {
        4 * 3 // [flags key next]
        + 1 // flags
        + (node_capacity + 1) * 8 // key
        + pos * 8 // son
    }

    pub fn get_offset_bucket(pos: usize, node_capacity: usize) -> usize {
        4 * 3 // [flags key next]
        + 1 // flags
        + (node_capacity + 1) * 8 // key
        + pos * 8 // bucket
    }
    
    pub fn to_raw(&self, key: u64) -> RawIndex {
        RawIndex::from(&self.th.get_index_(key))
    }

    fn lower_bound(&self, key: &RawIndex) -> usize {
        if self.key.is_empty() {
            return 0;
        }
        let mut res = self.key.len();
        let mut l = 0;
        let mut r = self.key.len() - 1;
        while l <= r {
            let mid = (l + r) >> 1;
            if self.to_raw(self.key[mid]) >= *key {
                res = mid;
                if mid == 0 {
                    break;
                }
                r = mid - 1;
            }
            else {
                l = mid + 1;
            }
        }
        res
    }

    fn upper_bound(&self, key: &RawIndex) -> usize {
        if self.key.is_empty() {
            return 0;
        }
        let mut res = self.key.len();
        let mut l = 0;
        let mut r = self.key.len() - 1;
        while l <= r {
            let mid = (l + r) >> 1;
            if self.to_raw(self.key[mid]) > *key {
                res = mid;
                if mid == 0 {
                    break;
                }
                r = mid - 1;
            }
            else {
                l = mid + 1;
            }
        }
        res
    }

    pub fn split(&mut self, node_capacity: usize, father: &mut BTreeNode, father_ptr: u64, pos: usize, self_ptr: &mut u64) {
        let mid = node_capacity / 2;
        match self.ty {
            BTreeNodeType::Leaf => {
                let mid_key = self.key[mid];
                let new_node = BTreeNode {
                    th: self.th,
                    ty: BTreeNodeType::Leaf,
                    key: self.key.split_off(mid),
                    son: Vec::new(),
                    bucket: self.bucket.split_off(mid),
                };
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&self.bucket, node_capacity + 1).into_bytes());
                }
                let new_node_ptr = self.th.insert_btree_node(&new_node, node_capacity).to_u64();
                father.key.insert(pos, mid_key);
                father.son.insert(pos + 1, new_node_ptr);
                if father.key.len() <= node_capacity {
                    unsafe {
                        self.th.update_sub_(father_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&father.key, node_capacity + 1).into_bytes());
                        self.th.update_sub_(father_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&father.son, node_capacity + 1).into_bytes());
                    }
                }
            }
            BTreeNodeType::Internal => {
                let new_node = BTreeNode {
                    th: self.th,
                    ty: BTreeNodeType::Internal,
                    key: self.key.split_off(mid + 1),
                    son: self.son.split_off(mid + 1),
                    bucket: Vec::new(),
                };
                let mid_key = self.key.pop();
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&self.son, node_capacity + 1).into_bytes());
                }
                let new_node_ptr = self.th.insert_btree_node(&new_node, node_capacity).to_u64();
                father.key.insert(pos, mid_key.unwrap());
                father.son.insert(pos + 1, new_node_ptr);
                if father.key.len() <= node_capacity {
                    unsafe {
                        self.th.update_sub_(father_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&father.key, node_capacity + 1).into_bytes());
                        self.th.update_sub_(father_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&father.son, node_capacity + 1).into_bytes());
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, key: &RawIndex, data: u64, node_capacity: usize, father: Option<(&mut BTreeNode, u64)>, pos: usize, self_ptr: u64) -> u64 {
        let mut self_ptr = self_ptr;
        let mut modified = false;
        match self.ty {
            BTreeNodeType::Leaf => {
                let key_ptr = self.th.insert_index(&Index::from(self.th, key)).to_u64();
                let i = self.lower_bound(key);
                if i == self.key.len() {
                    modified = true;
                    self.key.push(key_ptr);
                    let prev_bucket = *self.bucket.last().unwrap_or(&0u64);
                    let next_bucket = if prev_bucket == 0 {0} else {self.th.get_bucket_(prev_bucket).next};

                    let mut bucket = Bucket::new();
                    bucket.data.push(data);
                    bucket.prev = prev_bucket;
                    bucket.next = next_bucket;

                    let ptr = self.th.insert_bucket(&bucket).to_u64();
                    unsafe {
                        self.th.update_sub_(prev_bucket, Bucket::get_offset_next(), convert::u64_to_vec_u8(ptr));
                        self.th.update_sub_(next_bucket, Bucket::get_offset_prev(), convert::u64_to_vec_u8(ptr));
                    }
                    self.bucket.push(ptr);
                }
                else if let Some(cmp) = self.to_raw(self.key[i]).partial_cmp(key) {
                    match cmp {
                        Ordering::Equal => {
                            let mut bucket = self.th.get_bucket_(self.bucket[i]);
                            bucket.data.push(data);
                            self.th.update_bucket_(&mut self.bucket[i], &bucket);
                            unsafe {
                                self.th.update_sub_(self_ptr, BTreeNode::get_offset_bucket(i, node_capacity), convert::u64_to_vec_u8(self.bucket[i]));
                            }
                        },
                        Ordering::Greater => {
                            modified = true;
                            let next_bucket = self.bucket[i];
                            let prev_bucket = self.th.get_bucket_(next_bucket).prev;

                            self.key.insert(i, key_ptr);
                            let mut bucket = Bucket::new();
                            bucket.data.push(data);
                            bucket.prev = prev_bucket;
                            bucket.next = next_bucket;

                            let ptr = self.th.insert_bucket(&bucket).to_u64();
                            unsafe {
                                self.th.update_sub_(prev_bucket, Bucket::get_offset_next(), convert::u64_to_vec_u8(ptr));
                                self.th.update_sub_(next_bucket, Bucket::get_offset_prev(), convert::u64_to_vec_u8(ptr));
                            }
                            self.bucket.insert(i, ptr);
                        },
                        _ => {}
                    }
                }
            }
            BTreeNodeType::Internal => {
                let son_pos = self.upper_bound(key);
                let son_ptr = self.son[son_pos];
                let mut son_node = self.th.get_btree_node_(son_ptr);
                son_node.insert(key, data, node_capacity, Some((self, self_ptr)), son_pos, son_ptr);
            }
        }
        
        // split
        if self.key.len() > node_capacity {
            match father {
                Some(father) => {
                    self.split(node_capacity, father.0, father.1, pos, &mut self_ptr);
                }
                None => {
                    let mut new_root = BTreeNode::new(self.th);
                    new_root.ty = BTreeNodeType::Internal;
                    new_root.son.push(self_ptr);
                    let new_root_ptr = self.th.insert_btree_node(&new_root, node_capacity).to_u64();
                    self.split(node_capacity, &mut new_root, new_root_ptr, pos, &mut self_ptr);
                    self_ptr = new_root_ptr;
                }
            }
        }
        else if modified {
            match self.ty {
                BTreeNodeType::Leaf => {
                    unsafe {
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&self.bucket, node_capacity + 1).into_bytes());
                    }
                }
                BTreeNodeType::Internal => {
                    unsafe {
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&self.son, node_capacity + 1).into_bytes());
                    }
                }
            }
        }

        self_ptr
    }

    pub fn combine_internal(&mut self, node_capacity: usize, father: &mut BTreeNode, father_ptr: u64, pos: usize, self_ptr: &mut u64) {
        if pos > 0 { // left sibling
            let mut sibling = self.th.get_btree_node_(father.son[pos - 1]);
            if sibling.key.len() > node_capacity / 2 {
                let key = sibling.key.pop().unwrap();
                self.key.insert(0, father.key[pos - 1]);
                let son = sibling.son.pop().unwrap();
                self.son.insert(0, son);
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&self.son, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&sibling.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&sibling.son, node_capacity + 1).into_bytes());
                }
                father.key[pos - 1] = key;
            }
            else {
                sibling.key.push(father.key[pos - 1]);
                sibling.key.append(&mut self.key);
                sibling.son.append(&mut self.son);
                unsafe {
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&sibling.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&sibling.son, node_capacity + 1).into_bytes());
                }
                father.key.remove(pos - 1);
                father.son.remove(pos);
            }
        }
        else if pos < father.son.len() - 1 { // right sibling
            let mut sibling = self.th.get_btree_node_(father.son[pos + 1]);
            if sibling.key.len() > node_capacity / 2 {
                let key = sibling.key.remove(0);
                self.key.push(father.key[pos]);
                let son = sibling.son.remove(0);
                self.son.push(son);
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&self.son, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos + 1], BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&sibling.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos + 1], BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&sibling.son, node_capacity + 1).into_bytes());
                }
                father.key[pos] = key;
            }
            else {
                self.key.push(father.key[pos]);
                self.key.append(&mut sibling.key);
                self.son.append(&mut sibling.son);
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&self.son, node_capacity + 1).into_bytes());
                }
                father.key.remove(pos);
                father.son.remove(pos + 1);
            }
        }
        if father.key.len() >= node_capacity / 2 {
            unsafe {
                self.th.update_sub_(father_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&father.key, node_capacity + 1).into_bytes());
                self.th.update_sub_(father_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&father.son, node_capacity + 1).into_bytes());
            }
        }
    }

    pub fn combine_leaf(&mut self, node_capacity: usize, father: &mut BTreeNode, father_ptr: u64, pos: usize, self_ptr: &mut u64) {
        if pos > 0 { // left sibling
            let mut sibling = self.th.get_btree_node_(father.son[pos - 1]);
            if sibling.key.len() > node_capacity / 2 {
                let key = sibling.key.pop().unwrap();
                self.key.insert(0, key);
                self.bucket.insert(0, sibling.bucket.pop().unwrap());
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&self.bucket, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&sibling.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&sibling.bucket, node_capacity + 1).into_bytes());
                }
                father.key[pos - 1] = key;
            }
            else {
                sibling.key.append(&mut self.key);
                sibling.bucket.append(&mut self.bucket);
                unsafe {
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&sibling.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos - 1], BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&sibling.bucket, node_capacity + 1).into_bytes());
                }
                father.key.remove(pos - 1);
                father.son.remove(pos);
            }
        }
        else if pos < father.son.len() - 1 { // right sibling
            let mut sibling = self.th.get_btree_node_(father.son[pos + 1]);
            if sibling.key.len() > node_capacity / 2 {
                let key = sibling.key.remove(0);
                self.key.push(key);
                self.bucket.push(sibling.bucket.remove(0));
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&self.bucket, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos + 1], BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&sibling.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(father.son[pos + 1], BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&sibling.bucket, node_capacity + 1).into_bytes());
                }
                father.key[pos] = key;
            }
            else {
                self.key.append(&mut sibling.key);
                self.bucket.append(&mut sibling.bucket);
                unsafe {
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                    self.th.update_sub_(*self_ptr, BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&self.bucket, node_capacity + 1).into_bytes());
                }
                father.key.remove(pos);
                father.son.remove(pos + 1);
            }
        }
        if father.key.len() >= node_capacity / 2 {
            unsafe {
                self.th.update_sub_(father_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&father.key, node_capacity + 1).into_bytes());
                self.th.update_sub_(father_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&father.son, node_capacity + 1).into_bytes());
            }
        }
    }

    pub fn delete(&mut self, key: &RawIndex, data: u64, node_capacity: usize, father: Option<(&mut BTreeNode, &mut u64)>, pos: usize, self_ptr: u64) -> u64 {
        let mut self_ptr = self_ptr;
        let mut modified = false;
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..self.key.len() {
                    if self.to_raw(self.key[i]) == *key {
                        let mut bucket = self.th.get_bucket_(self.bucket[i]);
                        for j in 0..bucket.data.len() {
                            if bucket.data[j] == data {
                                bucket.data.remove(j);
                                break;
                            }
                        }
                        if bucket.data.is_empty() {
                            modified = true;
                            let prev_bucket = if i > 0 {self.bucket[i - 1]} else {0u64};
                            let next_bucket = if i + 1 < self.bucket.len() {self.bucket[i + 1]} else {0u64};
                            let prev_bucket = if prev_bucket == 0 && next_bucket != 0 {self.th.get_bucket_(next_bucket).prev} else {prev_bucket};
                            let next_bucket = if next_bucket == 0 && prev_bucket != 0 {self.th.get_bucket_(prev_bucket).next} else {next_bucket};
                            unsafe {
                                if prev_bucket != 0 {
                                    self.th.update_sub_(prev_bucket, Bucket::get_offset_next(), convert::u64_to_vec_u8(next_bucket));
                                }
                                if next_bucket != 0 {
                                    self.th.update_sub_(next_bucket, Bucket::get_offset_prev(), convert::u64_to_vec_u8(prev_bucket));
                                }
                            }
                            self.key.remove(i);
                            self.bucket.remove(i);
                        }
                        else {
                            self.th.update_bucket_(&mut self.bucket[i], &bucket);
                            unsafe {
                                self.th.update_sub_(self_ptr, BTreeNode::get_offset_bucket(i, node_capacity), convert::u64_to_vec_u8(self.bucket[i]));
                            }
                        }
                        break;
                    }
                }
            }
            BTreeNodeType::Internal => {
                let son_pos = self.upper_bound(key);
                let son_ptr = self.son[son_pos];
                let mut son_node = self.th.get_btree_node_(son_ptr);
                son_node.delete(key, data, node_capacity, Some((self, &mut self_ptr)), son_pos, son_ptr);
            }
        }

        // combine
        if self.key.len() < node_capacity / 2 && father.is_some() {
            let father = father.unwrap();
            match self.ty {
                BTreeNodeType::Leaf => {
                    self.combine_leaf(node_capacity, father.0, *father.1, pos, &mut self_ptr);
                }
                BTreeNodeType::Internal => {
                    self.combine_internal(node_capacity, father.0, *father.1, pos, &mut self_ptr);
                }
            }
        }
        else if father.is_none() && self.key.is_empty() && !self.son.is_empty() { // delete root
            self_ptr = self.son[0];
        }
        else if modified || father.is_none() {
            match self.ty {
                BTreeNodeType::Leaf => {
                    unsafe {
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_bucket(0, node_capacity), convert::vec_u64_to_string_len(&self.bucket, node_capacity + 1).into_bytes());
                    }
                }
                BTreeNodeType::Internal => {
                    unsafe {
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&self.key, node_capacity + 1).into_bytes());
                        self.th.update_sub_(self_ptr, BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&self.son, node_capacity + 1).into_bytes());
                    }
                }
            }
        }

        self_ptr
    }

    pub fn search(&self, key: &RawIndex) -> Option<Bucket> {
        match self.ty {
            BTreeNodeType::Leaf => {
                let i = self.lower_bound(key);
                if i < self.key.len() && self.to_raw(self.key[i]) == *key {
                    return Some(self.th.get_bucket_(self.bucket[i]));
                }
            }
            BTreeNodeType::Internal => {
                let son_pos = self.upper_bound(key);
                return self.th.get_btree_node_(self.son[son_pos]).search(key);
            }
        }
        None
    }

    pub fn get_height(&self) -> usize {
        match self.ty {
            BTreeNodeType::Leaf => {
                0
            }
            BTreeNodeType::Internal => {
                self.th.get_btree_node_(self.son[0]).get_height() + 1
            }
        }
    }

    pub fn traverse(&self, self_ptr: u64) {
        match self.ty {
            BTreeNodeType::Leaf => {
                assert_eq!(self.key.len(), self.bucket.len());
            }
            BTreeNodeType::Internal => {
                assert_eq!(self.key.len() + 1, self.son.len());
                for i in 0..self.son.len() {
                    self.th.get_btree_node_(self.son[i]).traverse(self.son[i]);
                }
            }
        }
    }

    pub fn first_bucket(&self) -> Option<Bucket> {
        match self.ty {
            BTreeNodeType::Leaf => {
                self.bucket.first().map(|x| self.th.get_bucket_(*x))
            }
            BTreeNodeType::Internal => {
                self.th.get_btree_node_(*self.son.first().unwrap()).first_bucket()
            }
        }
    }

    pub fn last_bucket(&self) -> Option<Bucket> {
        match self.ty {
            BTreeNodeType::Leaf => {
                self.bucket.last().map(|x| self.th.get_bucket_(*x))
            }
            BTreeNodeType::Internal => {
                self.th.get_btree_node_(*self.son.last().unwrap()).first_bucket()
            }
        }
    }
}