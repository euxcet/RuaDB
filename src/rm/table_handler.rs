use super::file_handler::*;
use super::record::*;
use super::in_file::*;
use super::pagedef::*;
use crate::index::in_file::*;
use crate::index::btree::*;
use crate::utils::convert;

use std::fmt;
use std::collections::HashMap;
use std::mem::size_of;

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

    pub fn delete(&self, ptr: &StrPointer) {
        self.fh.delete(ptr);
    }

    pub fn delete_(&self, ptr: u64) {
        self.fh.delete(&StrPointer::new(ptr));
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

    pub fn update_string(&self, ptr: &StrPointer, s: &String) {
        self.fh.update::<String, u32>(ptr, &s);
    }

    pub fn update_string_(&self, ptr: u64, s: &String) {
        self.fh.update::<String, u32>(&StrPointer::new(ptr), &s);
    }

    // for Record
    pub fn insert_record(&self, record: &Record) -> StrPointer {
        self.fh.insert::<RecordInFile, u32>(&RecordInFile::from(self, record))
    }

    pub fn insert_record_get_record_in_file(&self, record: &Record) -> (StrPointer, RecordInFile) {
        let rif = RecordInFile::from(self, record);
        let ptr = self.fh.insert::<RecordInFile, u32>(&rif);
        (ptr, rif)
    }

    pub fn get_record(&self, ptr: &StrPointer) -> (Record, RecordInFile) {
        let in_file = self.fh.get::<RecordInFile, u32>(ptr);
        (in_file.to_record(self), in_file)
    }

    pub fn get_record_(&self, ptr: u64) -> (Record, RecordInFile) {
        let in_file = self.fh.get::<RecordInFile, u32>(&StrPointer::new(ptr));
        (in_file.to_record(self), in_file)
    }

    pub fn update_record(&self, ptr: &StrPointer, record: &Record) {
        self.fh.update::<RecordInFile, u32>(ptr, &RecordInFile::from(self, record));
    }

    pub fn update_record_in_file(&self, ptr: &StrPointer, record: &RecordInFile) {
        self.fh.update::<RecordInFile, u32>(ptr, &record);
    }

    pub fn update_record_(&self, ptr: u64, record: &Record) {
        self.fh.update::<RecordInFile, u32>(&StrPointer::new(ptr), &RecordInFile::from(self, record));
    }

    pub fn delete_record_data_column(&self, ptr: &StrPointer, i: usize) {
        let (_, mut record_in_file) = self.get_record(ptr);
        let size_of_data = size_of::<ColumnDataInFile>();
        let data_str: String = record_in_file.record.drain(size_of_data * i .. size_of_data * (i + 1)).collect();
        let data_in_file = ColumnDataInFile::new(data_str.as_bytes());
        if data_in_file.get_type() == ColumnDataInFile::str_type() {
            self.delete(&StrPointer::new(data_in_file.data));
        }
        self.update_record_in_file(ptr, &record_in_file);
    }

    pub fn insert_record_data_column(&self, ptr: &StrPointer, ct: &ColumnType) {
        let (_, mut record_in_file) = self.get_record(ptr);
        let size_of_data = size_of::<ColumnDataInFile>();
        let cd = ColumnDataInFile::null_data(ct);
        record_in_file.record.push_str(cd.as_str());

        self.update_record_in_file(ptr, &record_in_file);
    }

    // for ColumnType
    pub fn __insert_column_type(&self, ct: &ColumnType) -> StrPointer {
        self.fh.insert::<ColumnTypeInFile, u32>(&ColumnTypeInFile::from(self, ct))
    }

    pub fn insert_column_types(&self, cts: &ColumnTypeVec) {
        let c_ptrs = cts.cols.iter().map(|ct| self.__insert_column_type(ct).to_u64()).collect();
        let ptr = self.insert_string(&unsafe{convert::vec_u64_to_string(&c_ptrs)});
        self.fh.set_column_types_ptr(ptr.to_u64());
    }

    fn __get_ptrs(&self, ptr: &StrPointer) -> Vec<u64> {
        let s = self.get_string(&ptr);
        let ptrs = unsafe{convert::string_to_vec_u64(&s)};
        ptrs
    }

    pub fn insert_column_type(&self, ct: &ColumnType) {
        let ptrs_ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let mut ptrs = self.__get_ptrs(&ptrs_ptr);
        let cp = self.__insert_column_type(ct);
        ptrs.push(cp.to_u64());
        self.update_string(&ptrs_ptr, &unsafe{convert::vec_u64_to_string(&ptrs)});
    }

    pub fn delete_column_type_from_index(&self, index: usize) {
        let ptrs_ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let mut ptrs = self.__get_ptrs(&ptrs_ptr);
        assert!(index < ptrs.len());
        self.delete(&StrPointer::new(ptrs[index]));
        ptrs.remove(index);
        self.update_string(&ptrs_ptr, &unsafe{convert::vec_u64_to_string(&ptrs)});
    }

    pub fn update_column_type_from_index(&self, index: usize, ct: &ColumnType) {
        let ptrs_ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let ptrs = self.__get_ptrs(&ptrs_ptr);
        assert!(index < ptrs.len());
        self.update_column_type(&StrPointer::new(ptrs[index]), ct);
    }

    pub fn update_table_name(&self, tb_name: &String) {
        let ptrs_ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let ptrs = self.__get_ptrs(&ptrs_ptr);
        for ptr in &ptrs {
            let p = &StrPointer::new(*ptr);
            let mut ct = self.get_column_type(&p);
            ct.tb_name = tb_name.clone();
            self.update_column_type(&p, &ct);
        }
    }

    pub fn get_column_types(&self) -> ColumnTypeVec {
        let ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let c_ptrs = unsafe{convert::string_to_vec_u64(&self.get_string(&ptr))};
        ColumnTypeVec {
            cols: c_ptrs.iter().map(|&p| self.get_column_type(&StrPointer::new(p))).collect(),
        }
    }

    pub fn get_primary_cols(&self) -> Option<ColumnTypeVec> {
        let btrees = self.get_btrees();
        let pri_tree = btrees.iter().find(|t| t.is_primary());
        match pri_tree {
            Some(pri_tree) => {
                let cts = self.get_column_types().cols;
                Some(ColumnTypeVec {
                    cols: pri_tree.index_col.iter().map(|i| cts[*i as usize].clone()).collect(),
                })
            },
            None => None,
        }
    }

    pub fn get_primary_column_index(&self) -> Option<Vec<u32>> {
        let btrees = self.get_btrees();
        let pri_tree = btrees.into_iter().find(|t| t.is_primary());
        match pri_tree {
            Some(pri_tree) => {
                Some(pri_tree.index_col)
            },
            None => None,
        }
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

    pub fn get_column_numbers(&self) -> usize {
        let ptr = StrPointer::new(self.fh.get_column_types_ptr());
        let s = self.get_string(&ptr);
        let c_ptrs = unsafe{ convert::string_to_vec_u64(&s) };
        c_ptrs.len()
    }

    pub fn get_column_type(&self, ptr: &StrPointer) -> ColumnType {
        self.fh.get::<ColumnTypeInFile, u32>(ptr).to_column_type(self)
    }

    pub fn get_column_type_(&self, ptr: u64) -> ColumnType {
        self.fh.get::<ColumnTypeInFile, u32>(&StrPointer::new(ptr)).to_column_type(self)
    }

    pub fn update_column_type(&self, ptr: &StrPointer, ct: &ColumnType) {
        self.fh.update::<ColumnTypeInFile, u32>(ptr, &ColumnTypeInFile::from(self, ct))
    }

    pub fn update_column_type_(&self, ptr: u64, ct: &ColumnType) {
        self.fh.update::<ColumnTypeInFile, u32>(&StrPointer::new(ptr), &ColumnTypeInFile::from(self, ct));
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
        self.fh.set_born_btree_ptr(self.__insert_btree(btree).to_u64());
    }

    pub fn get_born_btree(&self) -> BTree {
        let ptr = StrPointer::new(self.fh.get_born_btree_ptr());
        self.__get_btree(&ptr)
    }

    pub fn update_born_btree(&self, btree: &BTree) {
        let ptr = StrPointer::new(self.fh.get_born_btree_ptr());
        self.update_btree(&ptr, btree);
    }

    pub fn insert_btree(&self, btree: &BTree) {
        let ptrs_ptr = StrPointer::new(self.fh.get_btrees_ptr());
        let mut ptrs = self.__get_ptrs(&ptrs_ptr);
        let bp = self.__insert_btree(btree);
        ptrs.push(bp.to_u64());
        self.update_string(&ptrs_ptr, &unsafe{convert::vec_u64_to_string(&ptrs)});
    }

    pub fn delete_btree_from_index(&self, index: usize) {
        let ptrs_ptr = StrPointer::new(self.fh.get_btrees_ptr());
        let mut ptrs = self.__get_ptrs(&ptrs_ptr);
        assert!(index < ptrs.len());
        self.delete(&StrPointer::new(ptrs[index]));
        ptrs.remove(index);
        self.update_string(&ptrs_ptr, &unsafe{convert::vec_u64_to_string(&ptrs)});
    }

    pub fn get_btrees(&self) -> Vec<BTree> {
        let ptrs_ptr = StrPointer::new(self.fh.get_btrees_ptr());
        let ptrs = self.__get_ptrs(&ptrs_ptr);
        ptrs.iter().map(|&p| self.__get_btree(&StrPointer::new(p))).collect()
    }

    pub fn get_primary_btree_with_ptr(&self) -> Option<(StrPointer, BTree)> {
        let btrees = self.get_btrees_with_ptrs();
        btrees.into_iter().find(|(p, t)| t.is_primary())
    }

    pub fn get_primary_btree(&self) -> Option<BTree> {
        self.get_primary_btree_with_ptr().map(|(_, t)| t)
    }

    pub fn get_btrees_with_ptrs(&self) -> Vec<(StrPointer, BTree)> {
        let ptrs_ptr = StrPointer::new(self.fh.get_btrees_ptr());
        let ptrs = self.__get_ptrs(&ptrs_ptr);
        ptrs.iter().map(|&p| {
            let p = StrPointer::new(p);
            let t = self.__get_btree(&p);
            (p, t)
        }).collect()
    }

    pub fn __get_btree(&self, ptr: &StrPointer) -> BTree {
        self.fh.get::<BTreeInFile, u32>(ptr).to_btree(self)
    }

    pub fn get_btree_from_index(&self, index: usize) -> BTree {
        let ptrs_ptr = StrPointer::new(self.fh.get_btrees_ptr());
        let ptrs = self.__get_ptrs(&ptrs_ptr);
        assert!(index < ptrs.len());
        self.__get_btree(&StrPointer::new(ptrs[index]))
    }

    pub fn update_btree(&self, ptr: &StrPointer, btree: &BTree) {
        self.fh.update::<BTreeInFile, u32>(ptr, &BTreeInFile::from(self, btree))
    }

    pub fn update_btree_(&self, ptr: u64, btree: &BTree) {
        self.fh.update::<BTreeInFile, u32>(&StrPointer::new(ptr), &BTreeInFile::from(self, btree));
    }

    // for BTreeNode
    pub fn insert_btree_node(&self) -> StrPointer {
        self.fh.alloc(&vec![0u8; size_of::<BTreeNode>()], true)
    }

    pub fn get_btree_node(&self, ptr: &StrPointer) -> &mut BTreeNode {
        self.fh.get_mut(ptr)
    }

    pub fn get_btree_node_(&self, ptr: u64) -> &mut BTreeNode {
        self.fh.get_mut(&StrPointer::new(ptr))
    }

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

    pub fn update_index(&self, ptr: &StrPointer, index: &Index) {
        self.fh.update::<IndexInFile, u32>(ptr, &IndexInFile::from(self, index))
    }

    pub fn update_index_(&self, ptr: u64, index: &Index) {
        self.fh.update::<IndexInFile, u32>(&StrPointer::new(ptr), &IndexInFile::from(self, index));
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

    pub fn update_bucket(&self, ptr: &StrPointer, bucket: &Bucket) {
        self.fh.update::<BucketInFile, u32>(ptr, &BucketInFile::from(self, bucket))
    }

    pub fn update_bucket_(&self, ptr: u64, bucket: &Bucket) {
        self.fh.update::<BucketInFile, u32>(&StrPointer::new(ptr), &BucketInFile::from(self, bucket));
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