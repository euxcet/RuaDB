use std::vec;
use std::ptr;
use std::mem;

const BTREE_NODE_CAPACITY: usize = 3;

enum BTreeNodeType {
    Internal,
    Leaf,
}

pub struct BTree<Tk: PartialOrd + Copy, Td> {
    root: Box<BTreeNode<Tk, Td>>,
    btree_node_id: i32,
}

struct BTreeNode<Tk: PartialOrd + Copy, Td> {
    ty: BTreeNodeType,
    key: Vec<Tk>,
    data: Vec<Td>,
    son: Vec<Box<BTreeNode<Tk, Td>>>,
    father: *mut BTreeNode<Tk, Td>,
    btree: *mut BTree<Tk, Td>,
    id: i32,
}


impl<Tk: PartialOrd + Copy, Td> BTreeNode<Tk, Td> {

    fn left_sibling(&mut self) -> Option<*mut BTreeNode<Tk, Td>> {
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

    fn right_sibling(&mut self) -> Option<*mut BTreeNode<Tk, Td>> {
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

    pub fn new(btree: *mut BTree<Tk, Td>, id: i32) -> Self {
        BTreeNode {
            ty: BTreeNodeType::Leaf,
            key: Vec::new(),
            data: Vec::new(),
            son: Vec::new(),
            father: ptr::null_mut(),
            id: id,
            btree: btree,
        }
    }

    pub fn new_node(&mut self) -> Self {
        unsafe {
            (*self.btree).btree_node_id += 1;
            BTreeNode {
                ty: BTreeNodeType::Leaf,
                key: Vec::new(),
                data: Vec::new(),
                son: Vec::new(),
                father: ptr::null_mut(),
                id: (*self.btree).btree_node_id,
                btree: self.btree,
            }
        }
    }

    pub fn split_up(&mut self) {
        let mid = self.key.len() / 2;
        match self.ty {
            BTreeNodeType::Leaf => {
                let mut new_node = Box::new(self.new_node());
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
                let mut new_node = Box::new(self.new_node());
                new_node.ty = BTreeNodeType::Internal;
                new_node.key = self.key.split_off(mid + 1);
                new_node.son = self.son.split_off(mid + 1);
                for i in 0..new_node.son.len() {
                    new_node.son[i].father = &mut *new_node;
                }
                new_node.father = self.father;
                let mid_key = self.key.pop();
                unsafe {
                    for i in 0..=(*self.father).son.len() {
                        if (*self.father).son[i].id == self.id {
                            (*self.father).key.insert(i, mid_key.unwrap());
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
            let node = self.new_node();
            let mut new_node = Box::new(std::mem::replace(self, node));
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

    pub fn insert(&mut self, key: Tk, data: Td) {
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

    pub fn delete(&mut self, key: Tk) {
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

    pub fn search(&mut self, key: Tk) -> Option<&Td> {
        match self.ty {
            BTreeNodeType::Leaf => {
                for i in 0..self.key.len() {
                    if self.key[i] == key {
                        return Some(&self.data[i]);
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

impl<Tk: PartialOrd + Copy, Td> BTree<Tk, Td> {

    fn new() -> Box<Self> {
        let mut btree = Box::new(BTree {
            root: Box::new(BTreeNode::new(ptr::null_mut(), 0)),
            btree_node_id: 0,
        });
        btree.root.btree = &mut *btree;
        btree
    }

    fn insert_data(&mut self, key: Tk, data: Td) {
        self.root.insert(key, data);
    }

    fn delete_data(&mut self, key: Tk) {
        self.root.delete(key);
    }

    fn search_data(&mut self, key: Tk) -> Option<&Td> {
        self.root.search(key)
    }

}

#[cfg(test)]
mod btree_tests {
    use rand::prelude::*;
    use rand::SeedableRng;
    use super::BTree;
    #[test]
    fn test_btree() {
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

        for _ in 0..opt_cnt {
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
                    assert_eq!(btree.search_data(data[pos].0), Some(&data[pos].1));
                }
                else {
                    assert_eq!(btree.search_data(data[pos].0), None);
                }
            }
        }
    }
}
