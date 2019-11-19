use std::rc::Rc;
use std::cell::RefCell;
use std::mem::{transmute, size_of};
use crate::rm::pagedef::StrPointer;
use crate::rm::table_handler::TableHandler;
use crate::utils::convert;
use super::btree::*;

bytevec_decl! {
    pub struct BTreeInFile {
        root: u64, // StrPointer
        node_capacity: u32,
        index_col: String
        // index_type: String   necessary?
    }

    pub struct BTreeNodeInFile {
        /*
            flags [isLeaf, 0, 0, 0, 0, 0, 0, 0]
        */
        flags: u8,
        key: String,
        next: String
    }
}

bytevec_decl! {
    pub struct IndexInFile {
        pub index_type: String,
        pub index: String
    }
}

bytevec_decl! {
    pub struct BucketInFile {
        data: String
    }
}

impl IndexInFile {
    pub fn from(th: &TableHandler, index: &Index) -> Self {
        Self {
            index_type: unsafe{convert::vec_u8_to_string(&index.index_flags)},
            index: unsafe{convert::vec_u64_to_string(&index.index)},
        }
    }
    
    pub fn to_index<'a>(&self, th: &'a TableHandler) -> Index<'a> {
        Index {
            th: th,
            index_flags: convert::string_to_vec_u8(&self.index_type),
            index: unsafe{convert::string_to_vec_u64(&self.index)},
        }
    }
}

impl BTreeInFile {
    pub fn from(th: &TableHandler, btree: &BTree) -> Self {
        Self {
            root: btree.root,
            node_capacity: btree.node_capacity,
            index_col: unsafe{convert::vec_u32_to_string(&btree.index_col)},
        }
    }

    pub fn to_btree<'a>(&self, th: &'a TableHandler) -> BTree<'a> {
        BTree {
            th: th,
            root: self.root,
            node_capacity: self.node_capacity,
            index_col: unsafe{convert::string_to_vec_u32(&self.index_col)},
        }
    }
}

impl BTreeNodeInFile {
    pub fn from(th: &TableHandler, node: &BTreeNode) -> Self {
        Self {
            flags: match node.ty {
                BTreeNodeType::Leaf => 0,
                BTreeNodeType::Internal => 1,
            },
            key: unsafe{convert::vec_u64_to_string(&node.key)},
            next: match node.ty {
                BTreeNodeType::Leaf => { // from node.bucket
                    unsafe{convert::vec_u64_to_string(&node.bucket)}
                },
                BTreeNodeType::Internal => { // from node.son
                    unsafe{convert::vec_u64_to_string(&node.son)}
                },
            },
        }
    }
    
    pub fn to_btree_node<'a>(&self, th: &'a TableHandler) -> BTreeNode<'a> {
        BTreeNode {
            th: th, 
            ty: if self.flags & 1 > 0 {BTreeNodeType::Internal} else {BTreeNodeType::Leaf},
            key: unsafe{convert::string_to_vec_u64(&self.key)},
            son: if self.flags & 1 > 0 {unsafe{convert::string_to_vec_u64(&self.next)}} else {Vec::new()},
            bucket: if self.flags & 1 > 0 {Vec::new()} else {unsafe{convert::string_to_vec_u64(&self.next)}},
        }
    }
}

impl BucketInFile {
    pub fn from(th: &TableHandler, bucket: &Bucket) -> Self {
        Self {
            data: unsafe{convert::vec_u64_to_string(&bucket.data)},
        }
    }

    pub fn to_bucket(&self, th: &TableHandler) -> Bucket {
        Bucket {
            data: unsafe{convert::string_to_vec_u64(&self.data)},
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::random;
    use crate::rm::record_manager::*;
    use crate::rm::record::*;
    use super::*;

    fn gen_random_columns(gen: &mut random::Generator, number: usize, max_string_length: usize) -> Vec<ColumnType> {
        let mut columns = Vec::new();
        for i in 0..number {
            let ty_rand = gen.gen::<u8>() % 4;
            let has_default = gen.gen::<bool>();
            let ty: Type = match ty_rand {
                0 => Type::Int(if has_default {Some(gen.gen::<i64>())} else {None}),
                1 => Type::Float(if has_default {Some(gen.gen::<f64>())} else {None}),
                2 => Type::Date(if has_default {Some(gen.gen::<u64>())} else {None}),
                3 => Type::Str(if has_default {Some(gen.gen_string_s(max_string_length))} else {None}),
                _ => unreachable!()
            };

            columns.push(
                ColumnType {
                    index: i as u32,
                    name: gen.gen_string(max_string_length),
                    data_type: ty,
                    has_default: has_default,
                    default_null: !has_default,
                    .. Default::default()
                }
            );
        }
        columns
    }

    fn gen_record(gen: &mut random::Generator, columns: &Vec<ColumnType>, max_string_length: usize) -> Record {
        let mut record = Vec::new();
        for c in columns.iter() {
            let default = if c.has_default {gen.gen()} else {false};
            record.push(ColumnData {
                index: c.index,
                data: if default {
                    match &c.data_type {
                        &Type::Int(Some(x)) => Some(Data::Int(x)),
                        &Type::Float(Some(x)) => Some(Data::Float(x)),
                        &Type::Date(Some(x)) => Some(Data::Date(x)),
                        &Type::Str(Some(ref x)) => Some(Data::Str(x.clone())),
                        _ => unreachable!(),
                    }
                } else {
                    match &c.data_type {
                        &Type::Int(_) => Some(Data::Int(gen.gen::<i64>())),
                        &Type::Float(_) => Some(Data::Float(gen.gen::<f64>())),
                        &Type::Date(_) => Some(Data::Date(gen.gen::<u64>())),
                        &Type::Str(_) => Some(Data::Str(gen.gen_string_s(max_string_length as usize))),
                    }
                },
                default: default,
            });
        }
        Record {
            record: record
        }
    }

    #[test]
    fn alloc_btree() {
        let mut gen = random::Generator::new(true);
        const MAX_STRING_LENGTH: usize = 10;

        const MAX_RECORD_NUMBER: usize = 1000;

        let mut r = RecordManager::new();
        r.create_table("alloc_btree_test.rua");

        let columns = gen_random_columns(&mut gen, 10, MAX_STRING_LENGTH);
        let th = r.open_table("alloc_btree_test.rua");
        for c in &columns {
            th.insert_column_type(c);
        }
        th.close();

        let mut ptrs = Vec::new();

        let th = r.open_table("alloc_btree_test.rua");
        for _ in 0..MAX_RECORD_NUMBER {
            let record = gen_record(&mut gen, &columns, MAX_STRING_LENGTH);
            let insert_times: usize = gen.gen_range(1, 5);
            for _ in 0..insert_times {
                ptrs.push(th.insert_record(&record));
            }
        }
        th.close();

        let th = r.open_table("alloc_btree_test.rua");
        let btree = BTree::new(&th, 20, vec![0]);
        let btree_ptr = th.insert_btree(&btree);
        th.close();


        let th = r.open_table("alloc_btree_test.rua");

        let mut btree_ = th.get_btree(&btree_ptr);

        for i in 0..ptrs.len() {
            let record = th.get_record(&ptrs[i]);
            let index = RawIndex::from(&record.1.get_index(&th, &btree.index_col));
            btree_.insert_record(&index, ptrs[i].to_u64());
        }

        for i in 0..ptrs.len() {
            let record = th.get_record(&ptrs[i]);
            let index = RawIndex::from(&record.1.get_index(&th, &btree.index_col));
            assert!(btree_.search_record(&index).unwrap().data.contains(&ptrs[i].to_u64()));
        }

        th.close();
    }
}
