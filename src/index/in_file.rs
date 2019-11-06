use std::mem::{transmute, size_of};
bytevec_decl! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct BTree_in_file {
        root: i32
    }
}

/*
bytevec_decl! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct BTreeNode_in_file {
        ty: u8,
        father: i32,
        son: String, // vec<i32>
        key: vec<String>
    }
}
*/

#[test]
fn alloc_btree() {
    println!("{}", size_of::<f64>());
    println!("{}", size_of::<i64>());
    println!("{}", size_of::<u64>());
}


/*
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

*/