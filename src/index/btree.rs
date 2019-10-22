use std::vec;
use std::ptr;
use std::mem;

const BTREE_NODE_CAPACITY: usize = 3;

enum BTreeNodeType {
    Internal,
    Leaf,
}

static mut btree_node_id: i32 = 0;

struct BTreeNode {
    ty: BTreeNodeType,
    key: Vec<i32>,
    data: Vec<i32>,
    son: Vec<Box<BTreeNode>>,
    father: *mut BTreeNode,
    id: i32,
}

pub struct BTree {
    root: Box<BTreeNode>,
}


impl BTreeNode {

    fn left_sibling(&mut self) -> Option<*mut BTreeNode> {
        if self.father.is_null() {
            return None;
        }
        unsafe {
            for i in 1..(*self.father).son.len() {
                if (*self.father).son[i].id == self.id {
                    return Some(&mut *(*self.father).son[i - 1]);
                }
            }
        }
        return None;
    }

    fn right_sibling(&mut self) -> Option<*mut BTreeNode> {
        if self.father.is_null() {
            return None;
        }
        unsafe {
            for i in 0..(*self.father).son.len() - 1 {
                if (*self.father).son[i].id == self.id {
                    return Some(&mut *(*self.father).son[i + 1]);
                }
            }
        }
        return None;
    }

    pub fn new() -> Self {
        unsafe {
            btree_node_id += 1;
            BTreeNode {
                ty: BTreeNodeType::Leaf,
                key: Vec::new(),
                data: Vec::new(),
                son: Vec::new(),
                father: ptr::null_mut(),
                id: btree_node_id,
            }
        }
    }

    pub fn split_up(&mut self) {
        let mid = self.key.len() / 2;
        match self.ty {
            BTreeNodeType::Leaf => {
                let mut new_node = Box::new(BTreeNode::new());
                let mid_key = self.key[mid];

                new_node.key = self.key.split_off(mid);
                new_node.data = self.data.split_off(mid);
                new_node.father = self.father;
                unsafe {
                    for i in 0..=(*self.father).son.len() {
                        if (*self.father).son[i].id == self.id {
                            (*self.father).key.insert(i, mid_key);
                            (*self.father).son.insert(i + 1, new_node);
                            break;
                        }
                    }
                }
            }
            BTreeNodeType::Internal => {
                let mut new_node = Box::new(BTreeNode::new());
                let mid_key = self.key[mid];
                new_node.ty = BTreeNodeType::Internal;
                new_node.key = self.key.split_off(mid + 1);
                new_node.son = self.son.split_off(mid + 1);
                for i in 0..new_node.son.len() {
                    new_node.son[i].father = &mut *new_node;
                }
                new_node.father = self.father;
                self.key.pop();
                unsafe {
                    for i in 0..=(*self.father).son.len() {
                        if (*self.father).son[i].id == self.id {
                            (*self.father).key.insert(i, mid_key);
                            (*self.father).son.insert(i + 1, new_node);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn split(&mut self) {
        if self.key.len() <= BTREE_NODE_CAPACITY {
            return;
        }
        if self.father.is_null() { // split root
            let mut new_node = Box::new(std::mem::replace(self, BTreeNode::new()));
            new_node.father = self;

            // TODO: optimize this piece of code
            for i in 0..new_node.son.len() {
                new_node.son[i].father = &mut *new_node;
            }

            self.ty = BTreeNodeType::Internal;
            self.son.push(new_node);
            self.son[0].split_up();
        }
        else {
            self.split_up();
            unsafe {
                (*self.father).split();
            }
        }
    }

    pub fn insert(&mut self, key: i32, data: i32) {
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..=self.key.len() {
                    if i == self.key.len() {
                        self.key.push(key);
                        self.data.push(data);
                        break;
                    }
                    if self.key[i] >= key {
                        self.key.insert(i, key);
                        self.data.insert(i, data);
                        break;
                    }
                }
                self.split();
            }
            BTreeNodeType::Internal => {
                for i in 0..=self.key.len() {
                    if i == self.key.len() || self.key[i] > key {
                        self.son[i].insert(key, data);
                        break;
                    }
                }
            }
        }
    }

    /*
    TODO: merge nodes
    pub fn merge_borrow(&mut self, sibling: Option<*mut BTreeNode>, is_left: bool) -> bool {
        match sibling {
            Some(sibling) => {
                unsafe {
                    if (*sibling).key.len() > BTREE_NODE_CAPACITY / 2 {
                        let father = (*sibling).father;
                        let key = (*sibling).key.remove(0);
                        for i in 0..(*father).son.len() {
                            if is_left {
                                if (*father).son[i].id == (*sibling).id {
                                    (*father).key[i] = key;
                                }
                            }
                            else {
                                if (*father).son[i].id == self.id {
                                    (*father).key[i] = key;
                                }
                            }
                        }
                        return true;
                    }
                }
            },
            None => return false
        }
        false
    }

    pub fn merge_with(&mut self, sibling: Option<*mut BTreeNode>, is_left: bool) -> bool {
        true
    }

    pub fn merge(&mut self) {
        if self.key.len() >= BTREE_NODE_CAPACITY / 2 {
            return;
        }
        let left_sibling = self.left_sibling();
        let right_sibling = self.right_sibling();
        if self.merge_borrow(left_sibling, true) || self.merge_borrow(right_sibling, false) {
            return;
        }
        if self.merge_with(left_sibling, true) || self.merge_with(right_sibling, false) {
            unsafe {
                (*self.father).merge();
            }
            return;
        }
    }
    */

    pub fn delete_up(&mut self) {
        if self.key.is_empty() && self.son.is_empty() && !self.father.is_null() {
            unsafe {
                for i in 0..(*self.father).son.len() {
                    if (*self.father).son[i].id == self.id {
                        if i == 0 {
                            if !(*self.father).key.is_empty() {
                                (*self.father).key.remove(0);
                            }
                        }
                        else {
                            (*self.father).key.remove(i - 1);
                        }
                        (*self.father).son.remove(i);
                        break;
                    }
                }
                (*self.father).delete_up();
            }
        }
    }

    pub fn delete(&mut self, key: i32) {
        // println!("delete {:?} {:?}", self.key, self.data);
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..self.key.len() {
                    if self.key[i] == key {
                        self.key.remove(i);
                        self.data.remove(i);
                        break;
                    }
                }
                self.delete_up();
            }
            BTreeNodeType::Internal => {
                for i in 0..=self.key.len() {
                    if i == self.key.len() || self.key[i] > key {
                        self.son[i].delete(key);
                        break;
                    }
                }
            }
        }
    }

    pub fn search(&mut self, key: i32) -> Option<i32> {
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..self.key.len() {
                    if self.key[i] == key {
                        return Some(self.data[i]);
                    }
                }
                return None;
            }
            BTreeNodeType::Internal => {
                for i in 0..=self.key.len() {
                    if i == self.key.len() || self.key[i] > key {
                        return self.son[i].search(key);
                    }
                }
                return None;
            }
        }
    }
}

impl BTree {
    fn new() -> Self {
        BTree {
            root: Box::new(BTreeNode::new()),
        }
    }

    fn insert_data(&mut self, key: i32, data: i32) {
        self.root.insert(key, data);
    }

    fn delete_data(&mut self, key: i32) {
        self.root.delete(key);
    }

    fn search_data(&mut self, key: i32) -> Option<i32> {
        self.root.search(key)
    }

}

// #[cfg(test)]


#[cfg(test)]
mod btree_tests {
    use rand::prelude::*;
    use rand::SeedableRng;
    use super::BTree;
    use super::BTreeNode;
    #[test]
    fn test_insert() {
        let mut btree = BTree::new();

        let mut data: Vec<(i32, i32)> = Vec::new();
        let mut vis: Vec<bool> = Vec::new();

        let size: usize = 2000;
        let opt_cnt: usize = 200000;
        let seedable: bool = true;

        for i in 0..size {
            data.push((i as i32, random()));
            vis.push(false);
        }

        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);

        let mut opt_type: bool;
        let mut pos: usize;

        for i in 0..opt_cnt {
            if seedable {
                opt_type = rng.gen::<bool>();
                pos = rng.gen::<usize>() % size;
            }
            else {
                opt_type = random::<bool>();
                pos = random::<usize>() % size;
            }
            if opt_type {
                if vis[pos] {
                    btree.delete_data(data[pos].0);
                    vis[pos] = false;
                }
                else {
                    btree.insert_data(data[pos].0, data[pos].1);
                    vis[pos] = true;
                }
            }
            else {
                if vis[pos] {
                    assert_eq!(btree.search_data(data[pos].0), Some(data[pos].1));
                }
                else {
                    assert_eq!(btree.search_data(data[pos].0), None);
                }
            }
        }
    }
}
