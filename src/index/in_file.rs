use std::rc::Rc;
use std::cell::RefCell;
use std::mem::{transmute, size_of};
use crate::rm::pagedef::StrPointer;
use crate::rm::table_handler::TableHandler;
use super::btree::*;
bytevec_decl! {
    pub struct BTreeInFile {
        root: u64, // StrPointer
        node_capacity: u32
        // index_type: String   necessary?
    }

    pub struct BTreeNodeInFile {
        /*
            flags [isLeaf, 0, 0, 0, 0, 0, 0, 0]
        */
        flags: u8,
        index: String,
        next: String
    }

    pub struct BucketInFile {
        data: String
    }
}

/*
    ColumnDataInFile represents the specific arrangement BTreeNodeInFile::index
    It's not stored in the file separately
*/
pub struct IndexInFile {
    index_type: Vec<u8>,
    index: Vec<u64>,
}

impl BTreeInFile {
    pub fn from(node_capacity: u32) -> Self {
        Self {
            // TODO: insert root
            root: StrPointer::new(0).to_u64(),
            node_capacity: node_capacity,
        }
    }

    pub fn to_btree<'a>(&self, th: &'a TableHandler) -> BTree<'a> {
        BTree {
            root: Rc::new(RefCell::new(
                // TODO get root
                BTreeNode::new(th)
            )),
            node_capacity: self.node_capacity,
        }
    }
}

impl BTreeNodeInFile {
    pub fn from(node: &BTreeNode) -> Self {
        Self {
            flags: match node.ty {
                BTreeNodeType::Leaf => 0,
                BTreeNodeType::Internal => 1,
            },
            index: String::new(), // TODO
            next: String::new(), // TODO
        }
    }
    
    pub fn to_btree_node<'a>(&self, th: &'a TableHandler) -> BTreeNode<'a> {
        let ty = if self.flags & 1 > 0 {BTreeNodeType::Internal} else {BTreeNodeType::Leaf};
        BTreeNode {
            th: th,
            ty: ty,
            key: Vec::new(), // TODO from self.index
            son: Vec::new(), // TODO from self.next
            bucket: Vec::new(), // TODO from self.next
            father: None,
        }
    }
}

impl BucketInFile {
}

/*
pub struct BTree<Tk: PartialOrd + Copy + fmt::Debug> {
    root: Rc<RefCell<BTreeNode<Tk>>>,
    node_capacity: u8,
}
*/




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