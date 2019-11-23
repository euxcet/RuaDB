enum BTreeNodeType {
    Internal,
    Leaf,
}

pub struct BTree<Tk: PartialOrd + Copy + fmt::Debug, Td: Copy + fmt::Debug> {
    root: Rc<RefCell<BTreeNode<Tk, Td>>>,
    btree_node_id: i32,
}

struct BTreeNode<Tk: PartialOrd + Copy + fmt::Debug, Td: Copy + fmt::Debug> {
    ty: BTreeNodeType,
    key: Vec<Tk>,
    data: Vec<Td>,
    son: Vec<Rc<RefCell<BTreeNode<Tk, Td>>>>,
    father: Option<Rc<RefCell<BTreeNode<Tk, Td>>>>,
    btree: Option<Rc<RefCell<BTree<Tk, Td>>>>,
    id: i32,
}


impl<Tk: PartialOrd + Copy + fmt::Debug, Td: Copy + fmt::Debug> BTreeNode<Tk, Td> {

    fn left_sibling(&mut self) -> Option<Rc<RefCell<BTreeNode<Tk, Td>>>> {
        match self.father {
            Some(ref father) => {
                let father = father.borrow();
                let mut sibling = None;
                for i in 1..father.son.len() {
                    if father.son[i].borrow().id == self.id {
                        sibling = Some(father.son[i - 1].clone());
                    }
                }
                sibling
            }
            None => None
        }

        /*
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
        */
    }

    fn right_sibling(&mut self) -> Option<Rc<RefCell<BTreeNode<Tk, Td>>>> {
        match self.father {
            Some(ref father) => {
                let father = father.borrow();
                let mut sibling = None;
                for i in 0..father.son.len() - 1 {
                    if father.son[i].borrow().id == self.id {
                        sibling = Some(father.son[i + 1].clone());
                    }
                }
                sibling
            }
            None => None
        }
        /*
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
        */
    }

    pub fn new(btree: Option<Rc<RefCell<BTree<Tk, Td>>>>, id: i32) -> Self {
        BTreeNode {
            ty: BTreeNodeType::Leaf,
            key: Vec::new(),
            data: Vec::new(),
            son: Vec::new(),
            father: None,
            id: id,
            btree: btree,
        }
    }

    pub fn new_node(&mut self) -> Self {
        match self.btree {
            Some(ref btree) => {
                btree.borrow_mut().btree_node_id += 1;
                BTreeNode {
                    ty: BTreeNodeType::Leaf,
                    key: Vec::new(),
                    data: Vec::new(),
                    son: Vec::new(),
                    father: None,
                    id: btree.borrow_mut().btree_node_id,
                    btree: self.btree.clone(),
                }
            }
            None => {
                panic!();
            }
        }
    }

    pub fn split_up(&mut self) {
        let mid = self.key.len() / 2;
        match self.ty {
            BTreeNodeType::Leaf => {
                let new_node = Rc::new(RefCell::new(self.new_node()));

                let mid_key = self.key[mid];
                new_node.borrow_mut().key = self.key.split_off(mid);
                new_node.borrow_mut().data = self.data.split_off(mid);
                new_node.borrow_mut().father = self.father.clone();

                if let Some(ref father) = self.father {
                    let mut father = father.borrow_mut();
                    for i in 0..father.son.len() {
                        if father.son[i].borrow().id == self.id {
                            father.key.insert(i, mid_key);
                            father.son.insert(i + 1, new_node);
                            break;
                        }
                    }
                }
            }
            BTreeNodeType::Internal => {
                let new_node = Rc::new(RefCell::new(self.new_node()));
                new_node.borrow_mut().ty = BTreeNodeType::Internal;
                new_node.borrow_mut().key = self.key.split_off(mid + 1);
                new_node.borrow_mut().son = self.son.split_off(mid + 1);
                new_node.borrow_mut().father = self.father.clone();

                for son in &mut new_node.borrow_mut().son {
                    son.borrow_mut().father = Some(new_node.clone());
                }

                let mid_key = self.key.pop();
                if let Some(ref father) = self.father {
                    let mut father = father.borrow_mut();
                    for i in 0..father.son.len() {
                        if father.son[i].borrow().id == self.id {
                            father.key.insert(i, mid_key.unwrap());
                            father.son.insert(i + 1, new_node);
                            break;
                        }
                    }
                }
            }
        }
        /*
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
        */
    }

    pub fn split(&mut self) {
        if self.key.len() <= BTREE_NODE_CAPACITY {
            return;
        }
        let father = self.father.clone();
        match father {
            Some(ref father) => {
                self.split_up();
                father.borrow_mut().split();
            }
            None => {
                let node = self.new_node();
                let new_node = Rc::new(RefCell::new(std::mem::replace(self, node)));

                if let Some(ref btree) = self.btree {
                    new_node.borrow_mut().father = Some(btree.borrow().root.clone());
                }

                for son in &new_node.borrow_mut().son {
                    son.borrow_mut().father = Some(new_node.clone());
                }

                self.ty = BTreeNodeType::Internal;
                self.son.push(new_node);
                self.son[0].borrow_mut().split_up();
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
                        self.son[i].borrow_mut().insert(key, data);
                        break;
                    }
                }
            }
        }
    }

    pub fn delete_up(&mut self) {
        if let Some(ref father) = self.father {
            if self.key.is_empty() && self.son.is_empty() {
                let mut father = father.borrow_mut();
                for i in 0..father.son.len() {
                    if father.son[i].borrow().id == self.id {
                        if i == 0 {
                            if !father.key.is_empty() {
                                father.key.remove(0);
                            }
                        }
                        else {
                            father.key.remove(i - 1);
                        }
                        father.son.remove(i);
                        break;
                    }
                }
                father.delete_up();
            }
        }
        /*
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
        */
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
                        self.son[i].borrow_mut().delete(key);
                        break;
                    }
                }
            }
        }
    }

    pub fn search(&self, key: Tk) -> Option<Td> {
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
                        let son = self.son[i].borrow();
                        return son.search(key);
                    }
                }
                return None;
            }
        }
    }
}

impl<Tk: PartialOrd + Copy + fmt::Debug, Td: Copy + fmt::Debug> BTree<Tk, Td> {
    fn new() -> Rc<RefCell<Self>> {
        let root = Rc::new(RefCell::new(BTreeNode::new(None, 0)));
        let btree = Rc::new(RefCell::new(BTree {
            root: root,
            btree_node_id: 0,
        }));
        btree.borrow_mut().root.borrow_mut().btree = Some(btree.clone());
        btree
    }

    fn insert_data(&mut self, key: Tk, data: Td) {
        self.root.borrow_mut().insert(key, data);
    }

    fn delete_data(&mut self, key: Tk) {
        self.root.borrow_mut().delete(key);
    }

    fn search_data(&mut self, key: Tk) -> Option<Td> {
        self.root.borrow().search(key)
    }
}

#[cfg(test)]
mod btree_tests {
    use rand::prelude::*;
    use rand::SeedableRng;
    use super::BTree;
    #[test]
    fn test_btree() {
        /*
        let mut btree = BTree::new();

        let mut data: Vec<(i32, i32)> = Vec::new();
        let mut vis: Vec<bool> = Vec::new();

        let size: usize = 20;
        let opt_cnt: usize = 20;
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
                    /*
                    btree.delete_data(data[pos].0);
                    vis[pos] = false;
                    */
                }
                else {
                    println!("insert {}", data[pos].0);
                    btree.borrow_mut().insert_data(data[pos].0, data[pos].1);
                    vis[pos] = true;
                }
            }
            else {
                if vis[pos] {
                    assert_eq!(btree.borrow_mut().search_data(data[pos].0), Some(data[pos].1));
                }
                else {
                    assert_eq!(btree.borrow_mut().search_data(data[pos].0), None);
                }
            }
        }
        */
    }
}

/*
                unsafe {
                    self.th.update_sub(&StrPointer::new(father_ptr), BTreeNode::get_offset_key(0), convert::vec_u64_to_string_len(&father.key, node_capacity).into_bytes());
                    self.th.update_sub(&StrPointer::new(father_ptr), BTreeNode::get_offset_son(0, node_capacity), convert::vec_u64_to_string_len(&father.son, node_capacity).into_bytes());
                }
                */