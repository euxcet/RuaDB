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

    pub fn up(&mut self) {
        let mid = self.key.len() / 2;
        match self.ty {
            BTreeNodeType::Leaf => {
                unsafe {
                    println!(" father {:?} {:?}", self.father, (*self.father).key);
                }
                println!("split leaf");
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
                    println!(" father {:?}", (*self.father).key);
                }
            }
            BTreeNodeType::Internal => {
                println!("split internal");
                println!("{:?}", self.key);
                println!("{:?}", self.data);
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
                    println!(" father {:?}", (*self.father).key);
                }
            }
        }
    }

    pub fn split(&mut self) {
        if self.key.len() <= BTREE_NODE_CAPACITY {
            return;
        }
        println!("split --- {:?}", self.key);
        if self.father == ptr::null_mut() { // split root
            let mut new_node = Box::new(std::mem::replace(self, BTreeNode::new()));
            new_node.father = self;
            for i in 0..new_node.son.len() {
                new_node.son[i].father = &mut *new_node;
            }

            self.ty = BTreeNodeType::Internal;
            self.son.push(new_node);
            self.son[0].up();
        }
        else {
            self.up();
            unsafe {
                (*self.father).split();
            }
        }
    }

    pub fn insert(&mut self, key: i32, data: i32) {
        println!("insert {:?} {:?}", self.key, self.son.len());
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

    pub fn search(&mut self, key: i32) -> Option<i32> {
        println!("{:?}", self.key);
        println!("{:?}", self.data);
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

        let size: usize = 300;
        for i in 0..size {
            data.push((i as i32, random()));
        }

        let mut rng = rand::thread_rng();
        data.shuffle(&mut rng);

        /*
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
        data.shuffle(&mut rng);
        */


        for i in 0..size {
            println!("{} {}", data[i].0, data[i].1);
            btree.insert_data(data[i].0, data[i].1);
            println!("");
            println!("");
        }

        /*
        for i in 0..size {
            println!("test {}", i);
            assert_eq!(btree.search_data(data[i].0), Some(data[i].1));
        }
        */
    }
}
