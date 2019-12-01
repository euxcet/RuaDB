use super::file_handler::*;
use super::record::*;
use super::in_file::*;
use super::pagedef::*;
use crate::index::in_file::*;
use crate::index::btree::*;
use crate::utils::convert;

use std::fmt;
use std::collections::HashMap;

pub struct TableHandler {
    // TODO: support multiple filehandlers
    pub fh: FileHandler,
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

    pub fn update_string(&self, ptr: &mut StrPointer, s: &String) {
        self.fh.update::<String, u32>(ptr, &s);
    }

    // for Record
    pub fn insert_record(&self, record: &Record) -> StrPointer {
        self.fh.insert::<RecordInFile, u32>(&RecordInFile::from(self, record))
    }

    pub fn get_record(&self, ptr: &StrPointer) -> (Record, RecordInFile) {
        let in_file = self.fh.get::<RecordInFile, u32>(ptr);
        (in_file.to_record(self), in_file)
    }

    pub fn update_record(&self, ptr: &mut StrPointer, record: &Record) {
        self.fh.update::<RecordInFile, u32>(ptr, &RecordInFile::from(self, record));
    }

    // for ColumnType
    pub fn insert_column_type(&self, ct: &ColumnType) -> StrPointer {
        self.fh.insert::<ColumnTypeInFile, u32>(&ColumnTypeInFile::from(self, ct))
    }

    pub fn insert_column_types(&self, cts: &Vec<ColumnType>) {
        let mut c_ptrs = Vec::new();
        for ct in cts {
            c_ptrs.push(self.insert_column_type(ct).to_u64());
        }
        let ptr = self.insert_string(&unsafe{convert::vec_u64_to_string(&c_ptrs)});
        self.fh.set_column_types_ptr(ptr.to_u64());
    }

    pub fn get_column_types(&self) -> Vec<ColumnType> {
        let ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let s = self.get_string(&ptr);
        let c_ptrs = unsafe{convert::string_to_vec_u64(&s)};
        c_ptrs.iter().map(|&p| self.get_column_type(&StrPointer::new(p))).collect()
    }

    pub fn get_column_types_as_hashmap(&self) -> HashMap<String, ColumnType> {
        let ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let s = self.get_string(&ptr);
        let c_ptrs = unsafe{ convert::string_to_vec_u64(&s) };

        c_ptrs.iter().map(|&p| {
            let c = self.get_column_type(&StrPointer::new(p));
            (c.name.clone(), c)
        }).collect()
    }

    pub fn get_column_type(&self, ptr: &StrPointer) -> ColumnType {
        self.fh.get::<ColumnTypeInFile, u32>(ptr).to_column_type(self)
    }

    pub fn update_column_type(&self, ptr: &mut StrPointer, ct: &ColumnType) {
        self.fh.update::<ColumnTypeInFile, u32>(ptr, &ColumnTypeInFile::from(self, ct))
    }

    // for BTree
    // TODO: use u8 instead of string, use iterator instead of copy
    pub fn __insert_btree(&self, btree: &BTree) -> StrPointer {
        self.fh.insert::<BTreeInFile, u32>(&BTreeInFile::from(self, btree))
    }

    pub fn init_btrees(&self) {
        let dummy: Vec<u64> = Vec::new();
        let ptr = self.insert_string(&unsafe{convert::vec_u64_to_string(&dummy)});
        self.fh.set_btrees_ptr(ptr.to_u64());
    }

    pub fn insert_born_btree(&self, btree: &BTree) {
        let p = self.__insert_btree(btree);
        self.fh.set_born_btree_ptr(p.to_u64());
    }

    pub fn get_born_btree(&self, btree: &BTree) -> (StrPointer, BTree) {
        let p = StrPointer::new(self.fh.get_born_btree_ptr());
        (p, self.__get_btree(&p))
    }

    pub fn insert_btree(&self, btree: &BTree) {
        // TODO update
        let mut ptrs = self.__get_btree_ptrs();

        let mut ptrs_ptr = StrPointer::new(self.fh.get_btrees_ptr());
        self.fh.free(&mut ptrs_ptr);

        let btree = self.__insert_btree(btree);
        ptrs.push(btree.to_u64());
        let new_btrees_ptr = self.insert_string(&unsafe{convert::vec_u64_to_string(&ptrs)});
        self.fh.set_btrees_ptr(new_btrees_ptr.to_u64());
    }

    fn __get_btree_ptrs(&self) -> Vec<u64> {
        let ptr = StrPointer::new(self.fh.get_btrees_ptr());
        let s = self.get_string(&ptr);
        let b_ptrs = unsafe{convert::string_to_vec_u64(&s)};
        b_ptrs
    }

    pub fn get_btrees(&self) -> Vec<BTree> {
        let ptrs = self.__get_btree_ptrs();
        ptrs.iter().map(|&p| self.__get_btree(&StrPointer::new(p))).collect()
    }

    pub fn __get_btree(&self, ptr: &StrPointer) -> BTree {
        self.fh.get::<BTreeInFile, u32>(ptr).to_btree(self)
    }

    fn get_btree_from_index(&self, index: usize) -> (StrPointer, BTree) {
        let ptrs = self.__get_btree_ptrs();
        assert!(index < ptrs.len());
        let p = StrPointer::new(ptrs[index]);

        (p, self.__get_btree(&p))
    }

    pub fn update_btree(&self, ptr: &mut StrPointer, btree: &BTree) {
        self.fh.update::<BTreeInFile, u32>(ptr, &BTreeInFile::from(self, btree))
    }

    // for BTreeNode
    pub fn insert_btree_node(&self, node: &BTreeNode, node_capacity: usize) -> StrPointer {
        self.fh.insert::<BTreeNodeInFile, u32>(&BTreeNodeInFile::from(self, node, node_capacity))
    }

    pub fn get_btree_node(&self, ptr: &StrPointer) -> BTreeNode {
        self.fh.get::<BTreeNodeInFile, u32>(ptr).to_btree_node(self)
    }

    pub fn update_btree_node(&self, ptr: &mut StrPointer, node: &BTreeNode, node_capacity: usize) {
        self.fh.update::<BTreeNodeInFile, u32>(ptr, &BTreeNodeInFile::from(self, node, node_capacity))
    }

    pub fn update_btree_node_(&self, ptr: &mut u64, node: &BTreeNode, node_capacity: usize) {
        let mut s_ptr = StrPointer::new(*ptr);
        self.fh.update::<BTreeNodeInFile, u32>(&mut s_ptr, &BTreeNodeInFile::from(self, node, node_capacity));
        *ptr = s_ptr.to_u64();
    }

    // for index
    pub fn insert_index(&self, index: &Index) -> StrPointer {
        self.fh.insert::<IndexInFile, u32>(&IndexInFile::from(self, index))
    }

    pub fn get_index(&self, ptr: &StrPointer) -> Index {
        self.fh.get::<IndexInFile, u32>(ptr).to_index(self)
    }

    pub fn update_index(&self, ptr: &mut StrPointer, index: &Index) {
        self.fh.update::<IndexInFile, u32>(ptr, &IndexInFile::from(self, index))
    }

    // for bucket
    pub fn insert_bucket(&self, bucket: &Bucket) -> StrPointer {
        self.fh.insert::<BucketInFile, u32>(&BucketInFile::from(self, bucket))
    }

    pub fn get_bucket(&self, ptr: &StrPointer) -> Bucket {
        self.fh.get::<BucketInFile, u32>(ptr).to_bucket(self)
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
        self.fh.update_sub(ptr, offset, data);
    }
}