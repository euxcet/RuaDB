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
     // TODO: null index
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

    pub fn from_u64(index: u64) -> Self {
        Self {
            index: vec![Data::Int(unsafe{transmute(index)})],
        }
    }

    pub fn from_record(record: &Record, sub_col: &Vec<u32>) -> Self {
        Self {
            index: sub_col.iter().map(|i| record.cols[*i as usize].data.clone().unwrap()).collect()
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
    pub index_col: Vec<u32>, // should be orderly
    // default_name: 
    // "" : born btree,
    // "[pk_name_name]": primary btree,
    // "foreign_constraint_name tb_name": foreign index
    pub index_name: String, 
    // born, primary, index, foreign
    // 0, 1, 2, 3
    pub ty: u8, 
}

impl<'a> BTree<'a> {
    pub fn new(th: &'a TableHandler, index_col: Vec<u32>, index_name: &str, ty: u8) -> Self {
        Self {
            th: th,
            root: th.insert_btree_node().to_u64(),
            index_col: index_col,
            index_name: index_name.to_string(),
            ty: ty,
        }
    }

    pub fn is_primary(&self) -> bool { self.ty == Self::primary_ty() } 
    pub fn is_foreign(&self) -> bool { self.ty == Self::foreign_ty() }
    pub fn is_index(&self) -> bool { self.ty == Self::primary_ty() }
    pub fn born_ty() -> u8 {0}
    pub fn primary_ty() -> u8 {1}
    pub fn index_ty() -> u8 {2}
    pub fn foreign_ty() -> u8 {3}

    pub fn get_foreign_constraint_name(&self) -> &str {
        if !self.is_foreign() {
            panic!("not foreign btree");
        }
        self.index_name.split_whitespace().next().unwrap()
    }

    pub fn get_foreign_table_name(&self) -> &str {
        assert!(self.is_foreign());

        let mut iter = self.index_name.split_whitespace();
        iter.next();
        iter.next().unwrap()
    }

    pub fn set_foreign_index_name(&mut self, constraint: &str, table: &str) {
        assert!(self.is_foreign());
        self.index_name = format!("{} {}", constraint, table);
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

    pub fn insert_record(&mut self, key: &RawIndex, data: u64, allow_duplicate: bool) -> bool {
        let root = self.th.get_btree_node_(self.root);
        let result = root.insert(self.th, key, data, None, 0, self.root, allow_duplicate);
        self.root = result.0;
        result.1
    }

    pub fn delete_record(&mut self, key: &RawIndex, data: u64) {
        let root = self.th.get_btree_node_(self.root);
        self.root = root.delete(self.th, key, data, None, 0, self.root);
    }

    pub fn search_record(&self, key: &RawIndex) -> Option<Bucket> {
        let root = self.th.get_btree_node_(self.root);
        root.search(self.th, key)
    }

    pub fn first_bucket(&self) -> Option<Bucket> {
        let root = self.th.get_btree_node_(self.root);
        root.first_bucket(self.th)
    }

    pub fn last_bucket(&self) -> Option<Bucket> {
        let root = self.th.get_btree_node_(self.root);
        root.last_bucket(self.th)
    }

    pub fn clear(&self) {
        let root = self.th.get_btree_node_(self.root);
        root.clear(self.th, self.root)
    }
}

#[derive(Debug)]
pub enum BTreeNodeType {
    Leaf = 0isize,
    Internal = 1isize,
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

const BTREE_NODE_CAPACITY: usize = 4;

#[repr(C, packed)]
pub struct BTreeNode {
    pub ty: BTreeNodeType,
    pub key: [u64; BTREE_NODE_CAPACITY + 2],
    pub son: [u64; BTREE_NODE_CAPACITY + 2],
    pub bucket: [u64; BTREE_NODE_CAPACITY + 2],
}

impl BTreeNode {
    pub fn new() -> Self {
        BTreeNode {
            ty: BTreeNodeType::Leaf,
            key: [0; BTREE_NODE_CAPACITY + 2],
            son: [0; BTREE_NODE_CAPACITY + 2],
            bucket: [0; BTREE_NODE_CAPACITY + 2],
        }
    }

    pub fn memory_length() -> usize {
        use std::mem::size_of;
        size_of::<BTreeNode>()
    }

    pub fn to_raw(&self, th: &TableHandler, key: u64) -> RawIndex {
        RawIndex::from(&th.get_index_(key))
    }

    fn get_len(&self) -> usize {
        let mut res = 0;
        let mut l = 0;
        let mut r = BTREE_NODE_CAPACITY;
        while l <= r {
            let mid = (l + r) >> 1;
            if self.key[mid] > 0 {
                res = mid + 1;
                l = mid + 1;
            }
            else {
                if mid == 0 {
                    break;
                }
                r = mid - 1;
            }
        }
        res
    }

    fn lower_bound(&self, th: &TableHandler, key: &RawIndex, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        let mut res = len;
        let mut l = 0;
        let mut r = len - 1;
        while l <= r {
            let mid = (l + r) >> 1;
            if self.to_raw(th, self.key[mid]) >= *key {
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

    fn upper_bound(&self, th: &TableHandler, key: &RawIndex, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        let mut res = len;
        let mut l = 0;
        let mut r = len - 1;
        while l <= r {
            let mid = (l + r) >> 1;
            if self.to_raw(th, self.key[mid]) > *key {
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

    pub fn insert_array(a: &mut[u64], pos: usize, val: u64, len: usize) {
        for i in (pos..len).rev() {
            a[i + 1] = a[i];
        }
        a[pos] = val;
    }

    pub fn remove(a: &mut[u64], pos: usize, len: usize) -> u64 {
        let res = a[pos];
        for i in pos..len - 1 {
            a[i] = a[i + 1];
        }
        a[len - 1] = 0;
        res
    }

    pub fn split_off(a: &mut[u64], pos: usize, len: usize) -> [u64; BTREE_NODE_CAPACITY + 2] {
        let mut res = [0; BTREE_NODE_CAPACITY + 2];
        for i in pos..len {
            res[i - pos] = a[i];
            a[i] = 0;
        }
        res
    }

    pub fn pop(a: &mut[u64], len: usize) -> Option<u64> {
        if len == 0 {
            None
        }
        else {
            let res = a[len - 1];
            a[len - 1] = 0;
            Some(res)
        }
    }

    pub fn push(a: &mut[u64], val: u64, len: usize) {
        a[len] = val;
    }

    pub fn append(a: &mut[u64], len_a: usize, b: &mut[u64], len_b: usize) {
        for i in 0..len_b {
            a[len_a + i] = b[i];
            b[i] = 0;
        }
    }

    pub fn split(&mut self, th: &TableHandler, father: &mut BTreeNode, pos: usize) {
        let mid = BTREE_NODE_CAPACITY / 2;
        let len = self.get_len();
        let father_len = father.get_len();
        match self.ty {
            BTreeNodeType::Leaf => {
                let mid_key = self.key[mid];
                let new_node_ptr = th.insert_btree_node();
                let mut new_node = th.get_btree_node(&new_node_ptr);
                unsafe {
                    new_node.key = BTreeNode::split_off(&mut self.key, mid, len);
                    new_node.bucket = BTreeNode::split_off(&mut self.bucket, mid, len);
                    BTreeNode::insert_array(&mut father.key, pos, mid_key, father_len);
                    BTreeNode::insert_array(&mut father.son, pos + 1, new_node_ptr.to_u64(), father_len + 1);
                }
            }
            BTreeNodeType::Internal => {
                let new_node_ptr = th.insert_btree_node();
                let mut new_node = th.get_btree_node(&new_node_ptr);
                new_node.ty = BTreeNodeType::Internal;
                unsafe {
                    new_node.key = BTreeNode::split_off(&mut self.key, mid + 1, len);
                    new_node.son = BTreeNode::split_off(&mut self.son, mid + 1, len + 1);
                    let mid_key = BTreeNode::pop(&mut self.key, mid + 1);
                    BTreeNode::insert_array(&mut father.key, pos, mid_key.unwrap(), father_len);
                    BTreeNode::insert_array(&mut father.son, pos + 1, new_node_ptr.to_u64(), father_len + 1);
                }
            }
        }
    }

    pub fn insert(&mut self, th: &TableHandler, key: &RawIndex, data: u64, father: Option<&mut BTreeNode>, pos: usize, self_ptr: u64, allow_duplicate: bool) -> (u64, bool) {
        let len = self.get_len();
        let mut dup = false;
        match self.ty {
            BTreeNodeType::Leaf => {
                let key_ptr = th.insert_index(&Index::from(th, key)).to_u64();
                let i = self.lower_bound(th, key, len);
                if i == len {
                    BTreeNode::push(unsafe{&mut self.key}, key_ptr, len);
                    let prev_bucket = if len == 0 {0} else {self.bucket[len - 1]};
                    let next_bucket = if prev_bucket == 0 {0} else {th.get_bucket_(prev_bucket).next};
                    let mut bucket = Bucket::new();
                    bucket.data.push(data);
                    bucket.prev = prev_bucket;
                    bucket.next = next_bucket;
                    let ptr = th.insert_bucket(&bucket).to_u64();
                    unsafe {
                        BTreeNode::push(&mut self.bucket, ptr, len);
                        if prev_bucket != 0 {
                            th.update_sub_(prev_bucket, Bucket::get_offset_next(), convert::u64_to_vec_u8(ptr));
                        }
                        if next_bucket != 0 {
                            th.update_sub_(next_bucket, Bucket::get_offset_prev(), convert::u64_to_vec_u8(ptr));
                        }
                    }
                }
                else if let Some(cmp) = self.to_raw(th, self.key[i]).partial_cmp(key) {
                    match cmp {
                        Ordering::Equal => {
                            dup = true;
                            if allow_duplicate {
                                let mut bucket = th.get_bucket_(self.bucket[i]);
                                bucket.data.push(data);
                                th.update_bucket_(self.bucket[i], &bucket);
                            }
                        },
                        Ordering::Greater => {
                            let next_bucket = self.bucket[i];
                            let prev_bucket = th.get_bucket_(next_bucket).prev;
                            unsafe {
                                BTreeNode::insert_array(&mut self.key, i, key_ptr, len);
                            }
                            let mut bucket = Bucket::new();
                            bucket.data.push(data);
                            bucket.prev = prev_bucket;
                            bucket.next = next_bucket;
                            let ptr = th.insert_bucket(&bucket).to_u64();
                            unsafe {
                                BTreeNode::insert_array(&mut self.bucket, i, ptr, len);
                                if prev_bucket != 0 {
                                    th.update_sub_(prev_bucket, Bucket::get_offset_next(), convert::u64_to_vec_u8(ptr));
                                }
                                if next_bucket != 0 {
                                    th.update_sub_(next_bucket, Bucket::get_offset_prev(), convert::u64_to_vec_u8(ptr));
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
            BTreeNodeType::Internal => {
                let son_pos = self.upper_bound(th, key, len);
                let son_ptr = self.son[son_pos];
                let son_node = th.get_btree_node_(son_ptr);
                dup = son_node.insert(th, key, data, Some(self), son_pos, son_ptr, allow_duplicate).1;
            }
        }
        
        let len = self.get_len();
        // split
        if len > BTREE_NODE_CAPACITY {
            match father {
                Some(father) => {
                    self.split(th, father, pos);
                }
                None => {
                    let new_root_ptr = th.insert_btree_node();
                    let mut new_root = th.get_btree_node(&new_root_ptr);
                    new_root.ty = BTreeNodeType::Internal;
                    BTreeNode::push(unsafe{&mut new_root.son}, self_ptr, 0);
                    self.split(th, &mut new_root, pos);
                    return (new_root_ptr.to_u64(), dup);
                }
            }
        }

        (self_ptr, dup)
    }

    pub fn combine_internal(&mut self, th: &TableHandler, father: &mut BTreeNode, pos: usize) {
        let len = self.get_len();
        let father_len = father.get_len();
        if pos > 0 { // left sibling
            let sibling = th.get_btree_node_(father.son[pos - 1]);
            let sibling_len = sibling.get_len();
            if sibling_len > BTREE_NODE_CAPACITY / 2 {
                unsafe {
                    let key = BTreeNode::pop(&mut sibling.key, sibling_len).unwrap();
                    BTreeNode::insert_array(&mut self.key, 0, father.key[pos - 1], len);
                    let son = BTreeNode::pop(&mut sibling.son, sibling_len + 1).unwrap();
                    BTreeNode::insert_array(&mut self.son, 0, son, len + 1);
                    father.key[pos - 1] = key;
                }
            }
            else {
                unsafe {
                    BTreeNode::push(&mut sibling.key, father.key[pos - 1], sibling_len);
                    BTreeNode::append(&mut sibling.key, sibling_len + 1, &mut self.key, len);
                    BTreeNode::append(&mut sibling.son, sibling_len + 1, &mut self.son, len + 1);
                    BTreeNode::remove(&mut father.key, pos - 1, father_len);
                    BTreeNode::remove(&mut father.son, pos, father_len + 1);
                }
            }
        }
        else if pos < father_len { // right sibling
            let sibling = th.get_btree_node_(father.son[pos + 1]);
            let sibling_len = sibling.get_len();
            if sibling_len > BTREE_NODE_CAPACITY / 2 {
                unsafe {
                    let key = BTreeNode::remove(&mut sibling.key, 0, sibling_len);
                    BTreeNode::push(&mut self.key, father.key[pos], len);
                    let son = BTreeNode::remove(&mut sibling.son, 0, sibling_len + 1);
                    BTreeNode::push(&mut self.son, son, len + 1);
                    father.key[pos] = key;
                }
            }
            else {
                unsafe {
                    BTreeNode::push(&mut self.key, father.key[pos], len);
                    BTreeNode::append(&mut self.key, len + 1, &mut sibling.key, sibling_len);
                    BTreeNode::append(&mut self.son, len + 1, &mut sibling.son, sibling_len + 1);
                    BTreeNode::remove(&mut father.key, pos, father_len);
                    BTreeNode::remove(&mut father.son, pos + 1, father_len + 1);
                }
            }
        }
    }

    pub fn combine_leaf(&mut self, th: &TableHandler, father: &mut BTreeNode, pos: usize) {
        let len = self.get_len();
        let father_len = father.get_len();
        if pos > 0 { // left sibling
            let sibling = th.get_btree_node_(father.son[pos - 1]);
            let sibling_len = sibling.get_len();
            if sibling_len > BTREE_NODE_CAPACITY / 2 {
                unsafe {
                    let key = BTreeNode::pop(&mut sibling.key, sibling_len).unwrap();
                    BTreeNode::insert_array(&mut self.key, 0, key, len);
                    let bucket = BTreeNode::pop(&mut sibling.bucket, sibling_len).unwrap();
                    BTreeNode::insert_array(&mut self.bucket, 0, bucket, len);
                    father.key[pos - 1] = key;
                }
            }
            else {
                unsafe {
                    BTreeNode::append(&mut sibling.key, sibling_len, &mut self.key, len);
                    BTreeNode::append(&mut sibling.bucket, sibling_len, &mut self.bucket, len);
                    BTreeNode::remove(&mut father.key, pos - 1, father_len);
                    BTreeNode::remove(&mut father.son, pos, father_len + 1);
                }
            }
        }
        else if pos < father_len { // right sibling
            let sibling = th.get_btree_node_(father.son[pos + 1]);
            let sibling_len = sibling.get_len();
            if sibling_len > BTREE_NODE_CAPACITY / 2 {
                unsafe {
                    let key = BTreeNode::remove(&mut sibling.key, 0, sibling_len);
                    BTreeNode::push(&mut self.key, key, len);
                    let bucket = BTreeNode::remove(&mut sibling.bucket, 0, sibling_len);
                    BTreeNode::push(&mut self.bucket, bucket, len);
                    father.key[pos] = key;
                }
            }
            else {
                unsafe {
                    BTreeNode::append(&mut self.key, len, &mut sibling.key, sibling_len);
                    BTreeNode::append(&mut self.bucket, len, &mut sibling.bucket, sibling_len);
                    BTreeNode::remove(&mut father.key, pos, father_len);
                    BTreeNode::remove(&mut father.son, pos + 1, father_len + 1);
                }
            }
        }
    }

    pub fn delete(&mut self, th: &TableHandler, key: &RawIndex, data: u64, father: Option<&mut BTreeNode>, pos: usize, self_ptr: u64) -> u64 {
        let mut self_ptr = self_ptr;
        let len = self.get_len();
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..len {
                    if self.to_raw(th, self.key[i]) == *key {
                        let mut bucket = th.get_bucket_(self.bucket[i]);
                        for j in 0..bucket.data.len() {
                            if bucket.data[j] == data {
                                bucket.data.remove(j);
                                break;
                            }
                        }
                        if bucket.data.is_empty() {
                            let prev_bucket = bucket.prev;
                            let next_bucket = bucket.next;
                            unsafe {
                                if prev_bucket != 0 {
                                    th.update_sub_(prev_bucket, Bucket::get_offset_next(), convert::u64_to_vec_u8(next_bucket));
                                }
                                if next_bucket != 0 {
                                    th.update_sub_(next_bucket, Bucket::get_offset_prev(), convert::u64_to_vec_u8(prev_bucket));
                                }
                            }
                            unsafe {
                                BTreeNode::remove(&mut self.key, i, len);
                                BTreeNode::remove(&mut self.bucket, i, len);
                            }
                        }
                        else {
                            th.update_bucket_(self.bucket[i], &bucket);
                        }
                        break;
                    }
                }
            }
            BTreeNodeType::Internal => {
                let son_pos = self.upper_bound(th, key, len);
                let son_ptr = self.son[son_pos];
                let son_node = th.get_btree_node_(son_ptr);
                son_node.delete(th, key, data, Some(self), son_pos, son_ptr);
            }
        }

        let len = self.get_len();
        // combine
        if len < BTREE_NODE_CAPACITY / 2 && father.is_some() {
            match self.ty {
                BTreeNodeType::Leaf => {
                    self.combine_leaf(th, father.unwrap(), pos);
                }
                BTreeNodeType::Internal => {
                    self.combine_internal(th, father.unwrap(), pos);
                }
            }
        }
        else if father.is_none() && self.key[0] == 0 && self.son[0] != 0 && self.son[1] == 0 { // delete root
            self_ptr = self.son[0];
        }

        self_ptr
    }

    pub fn search(&self, th: &TableHandler, key: &RawIndex) -> Option<Bucket> {
        let len = self.get_len();
        match self.ty {
            BTreeNodeType::Leaf => {
                let i = self.lower_bound(th, key, len);
                if i < len && self.to_raw(th, self.key[i]) == *key {
                    return Some(th.get_bucket_(self.bucket[i]));
                }
            }
            BTreeNodeType::Internal => {
                let son_pos = self.upper_bound(th, key, len);
                return th.get_btree_node_(self.son[son_pos]).search(th, key);
            }
        }
        None
    }

    pub fn first_bucket(&self, th: &TableHandler) -> Option<Bucket> {
        match self.ty {
            BTreeNodeType::Leaf => {
                if self.bucket[0] == 0 {
                    None
                }
                else {
                    Some(th.get_bucket_(self.bucket[0]))
                }
            }
            BTreeNodeType::Internal => {
                th.get_btree_node_(self.son[0]).first_bucket(th)
            }
        }
    }

    pub fn last_bucket(&self, th: &TableHandler) -> Option<Bucket> {
        match self.ty {
            BTreeNodeType::Leaf => {
                if self.bucket[0] == 0 {
                    None
                }
                else {
                    Some(th.get_bucket_(self.bucket[self.get_len() - 1]))
                }
            }
            BTreeNodeType::Internal => {
                th.get_btree_node_(self.son[self.get_len()]).first_bucket(th)
            }
        }
    }

    pub fn clear(&self, th: &TableHandler, self_ptr: u64) {
        let len = self.get_len();
        match self.ty {
            BTreeNodeType::Leaf => {}
            BTreeNodeType::Internal => {
                for i in 0..=len {
                    th.get_btree_node_(self.son[i]).clear(th, self.son[i])
                }
            }
        }
        th.delete_(self_ptr);
    }
}