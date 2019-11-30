use super::file_handler::*;
use super::record::*;
use super::in_file::*;
use super::pagedef::*;
use crate::index::in_file::*;
use crate::index::btree::*;
use std::fmt;

pub struct TableHandler {
    // TODO: support multiple filehandlers
    fh: FileHandler,
}

impl fmt::Debug for TableHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TableHandler")
    }
}

impl TableHandler {
    pub fn new(fh: FileHandler) -> Self {
        TableHandler {
            fh: fh,
        }
    }

    pub fn close(&self) {
        self.fh.close();
    }

    pub fn delete(&self, ptr: &mut StrPointer) {
        self.fh.delete(ptr);
    }

    // for String
    pub fn insert_string(&self, s: &String) -> StrPointer {
        self.fh.insert::<String, u32>(&s)
    }

    pub fn get_string(&self, ptr: &StrPointer) -> String {
        self.fh.get::<String, u32>(ptr)
    }

    pub fn get_string_(&self, ptr: u64) -> String {
        self.fh.get::<String, u32>(&StrPointer::new(ptr))
    }

    pub fn update_string(&self, ptr: &mut StrPointer, s: &String) {
        self.fh.update::<String, u32>(ptr, &s);
    }

    pub fn update_string_(&self, ptr: &mut u64, s: &String) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<String, u32>(&mut s_ptr, &s);
        *ptr = s_ptr.to_u64();
    }

    // for Record
    pub fn insert_record(&self, record: &Record) -> StrPointer {
        self.fh.insert::<RecordInFile, u32>(&RecordInFile::from(self, record))
    }

    pub fn get_record(&self, ptr: &StrPointer) -> (Record, RecordInFile) {
        let in_file = self.fh.get::<RecordInFile, u32>(ptr);
        (in_file.to_record(self), in_file)
    }

    pub fn get_record_(&self, ptr: u64) -> (Record, RecordInFile) {
        let in_file = self.fh.get::<RecordInFile, u32>(&StrPointer::new(ptr));
        (in_file.to_record(self), in_file)
    }

    pub fn update_record(&self, ptr: &mut StrPointer, record: &Record) {
        self.fh.update::<RecordInFile, u32>(ptr, &RecordInFile::from(self, record));
    }

    pub fn update_record_(&self, ptr: &mut u64, record: &Record) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<RecordInFile, u32>(&mut s_ptr, &RecordInFile::from(self, record));
        *ptr = s_ptr.to_u64();
    }

    // for ColumnType
    pub fn insert_column_type(&self, ct: &ColumnType) -> StrPointer {
        self.fh.insert::<ColumnTypeInFile, u32>(&ColumnTypeInFile::from(self, ct))
    }

    pub fn get_column_type(&self, ptr: &StrPointer) -> ColumnType {
        self.fh.get::<ColumnTypeInFile, u32>(ptr).to_column_type(self)
    }

    pub fn get_column_type_(&self, ptr: u64) -> ColumnType {
        self.fh.get::<ColumnTypeInFile, u32>(&StrPointer::new(ptr)).to_column_type(self)
    }

    pub fn update_column_type(&self, ptr: &mut StrPointer, ct: &ColumnType) {
        self.fh.update::<ColumnTypeInFile, u32>(ptr, &ColumnTypeInFile::from(self, ct))
    }

    pub fn update_column_type_(&self, ptr: &mut u64, ct: &ColumnType) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<ColumnTypeInFile, u32>(&mut s_ptr, &ColumnTypeInFile::from(self, ct));
        *ptr = s_ptr.to_u64();
    }

    // for BTree
    pub fn insert_btree(&self, btree: &BTree) -> StrPointer {
        self.fh.insert::<BTreeInFile, u32>(&BTreeInFile::from(self, btree))
    }

    pub fn get_btree(&self, ptr: &StrPointer) -> BTree {
        self.fh.get::<BTreeInFile, u32>(ptr).to_btree(self)
    }

    pub fn get_btree_(&self, ptr: u64) -> BTree {
        self.fh.get::<BTreeInFile, u32>(&StrPointer::new(ptr)).to_btree(self)
    }

    pub fn update_btree(&self, ptr: &mut StrPointer, btree: &BTree) {
        self.fh.update::<BTreeInFile, u32>(ptr, &BTreeInFile::from(self, btree))
    }

    pub fn update_btree_(&self, ptr: &mut u64, btree: &BTree) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<BTreeInFile, u32>(&mut s_ptr, &BTreeInFile::from(self, btree));
        *ptr = s_ptr.to_u64();
    }

    // for BTreeNode
    pub fn insert_btree_node(&self) -> StrPointer {
        self.fh.alloc(&vec![0u8; BTreeNode::memory_length()], true)
    }

    pub fn get_btree_node(&self, ptr: &StrPointer) -> &mut BTreeNode {
        self.fh.get_btree_node(ptr)
        // self.fh.get::<BTreeNodeInFile, u32>(ptr).to_btree_node(self)
    }

    pub fn get_btree_node_(&self, ptr: u64) -> &mut BTreeNode {
        self.fh.get_btree_node(&StrPointer::new(ptr))
    }

    /*
    pub fn get_btree_node_(&self, ptr: u64) -> BTreeNode {
        self.fh.get::<BTreeNodeInFile, u32>(&StrPointer::new(ptr)).to_btree_node(self)
    }

    pub fn update_btree_node(&self, ptr: &mut StrPointer, node: &BTreeNode, node_capacity: usize) {
        self.fh.update::<BTreeNodeInFile, u32>(ptr, &BTreeNodeInFile::from(self, node, node_capacity))
    }

    pub fn update_btree_node_(&self, ptr: &mut u64, node: &BTreeNode, node_capacity: usize) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<BTreeNodeInFile, u32>(&mut s_ptr, &BTreeNodeInFile::from(self, node, node_capacity));
        *ptr = s_ptr.to_u64();
    }
    */

    // for index
    pub fn insert_index(&self, index: &Index) -> StrPointer {
        self.fh.insert::<IndexInFile, u32>(&IndexInFile::from(self, index))
    }

    pub fn get_index(&self, ptr: &StrPointer) -> Index {
        self.fh.get::<IndexInFile, u32>(ptr).to_index(self)
    }

    pub fn get_index_(&self, ptr: u64) -> Index {
        self.fh.get::<IndexInFile, u32>(&StrPointer::new(ptr)).to_index(self)
    }

    pub fn update_index(&self, ptr: &mut StrPointer, index: &Index) {
        self.fh.update::<IndexInFile, u32>(ptr, &IndexInFile::from(self, index))
    }

    pub fn update_index_(&self, ptr: &mut u64, index: &Index) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<IndexInFile, u32>(&mut s_ptr, &IndexInFile::from(self, index));
        *ptr = s_ptr.to_u64();
    }


    // for bucket
    pub fn insert_bucket(&self, bucket: &Bucket) -> StrPointer {
        self.fh.insert::<BucketInFile, u32>(&BucketInFile::from(self, bucket))
    }

    pub fn get_bucket(&self, ptr: &StrPointer) -> Bucket {
        self.fh.get::<BucketInFile, u32>(ptr).to_bucket(self)
    }

    pub fn get_bucket_(&self, ptr: u64) -> Bucket {
        self.fh.get::<BucketInFile, u32>(&StrPointer::new(ptr)).to_bucket(self)
    }

    pub fn update_bucket(&self, ptr: &mut StrPointer, bucket: &Bucket) {
        self.fh.update::<BucketInFile, u32>(ptr, &BucketInFile::from(self, bucket))
    }

    pub fn update_bucket_(&self, ptr: &mut u64, bucket: &Bucket) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<BucketInFile, u32>(&mut s_ptr, &BucketInFile::from(self, bucket));
        *ptr = s_ptr.to_u64();
    }

    // for all
    pub fn update_sub(&self, ptr: &StrPointer, offset: usize, data: Vec<u8>) {
        if ptr.to_u64() != 0 {
            self.fh.update_sub(ptr, offset, data);
        }
    }

    pub fn update_sub_(&self, ptr: u64, offset: usize, data: Vec<u8>) {
        if ptr != 0 {
            self.fh.update_sub(&StrPointer::new(ptr), offset, data);
        }
    }
}