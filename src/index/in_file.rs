use crate::rm::table_handler::TableHandler;
use crate::utils::convert;
use super::btree::*;

bytevec_decl! {
    pub struct BTreeInFile {
        root: u64, // StrPointer
        index_col: String
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
        prev: u64,
        next: u64,
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
            index_col: unsafe{convert::vec_u32_to_string(&btree.index_col)},
        }
    }

    pub fn to_btree<'a>(&self, th: &'a TableHandler) -> BTree<'a> {
        BTree {
            th: th,
            root: self.root,
            index_col: unsafe{convert::string_to_vec_u32(&self.index_col)},
        }
    }
}

impl BucketInFile {
    pub fn from(th: &TableHandler, bucket: &Bucket) -> Self {
        Self {
            prev: bucket.prev,
            next: bucket.next,
            data: unsafe{convert::vec_u64_to_string(&bucket.data)},
        }
    }

    pub fn to_bucket(&self, th: &TableHandler) -> Bucket {
        Bucket {
            prev: self.prev,
            next: self.next,
            data: unsafe{convert::string_to_vec_u64(&self.data)},
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};
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
                4 => Type::Numeric(if has_default {Some(gen.gen::<i64>())} else {None}),
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
        let mut cols = Vec::new();
        for c in columns.iter() {
            let default = if c.has_default {gen.gen()} else {false};
            cols.push(ColumnData {
                index: c.index,
                data: if default {
                    match &c.data_type {
                        &Type::Int(Some(x)) => Some(Data::Int(x)),
                        &Type::Float(Some(x)) => Some(Data::Float(x)),
                        &Type::Date(Some(x)) => Some(Data::Date(x)),
                        &Type::Str(Some(ref x)) => Some(Data::Str(x.clone())),
                        &Type::Numeric(Some(x)) => Some(Data::Numeric(x)),
                        _ => unreachable!(),
                    }
                } else {
                    match &c.data_type {
                        &Type::Int(_) => Some(Data::Int(gen.gen::<i64>())),
                        &Type::Float(_) => Some(Data::Float(gen.gen::<f64>())),
                        &Type::Date(_) => Some(Data::Date(gen.gen::<u64>())),
                        &Type::Str(_) => Some(Data::Str(gen.gen_string_s(max_string_length as usize))),
                        &Type::Numeric(_) => Some(Data::Numeric(gen.gen::<i64>())),
                    }
                },
                default: default,
            });
        }
        Record {
            cols: cols
        }
    }

    #[test]
    fn alloc_btree() {
        let start_time = SystemTime::now();
        let mut gen = random::Generator::new(true);
        const MAX_STRING_LENGTH: usize = 10;
        const MAX_RECORD_NUMBER: usize = 1000;
        use crate::settings;

        let settings = settings::Settings::new().unwrap();

        #[cfg(target_os = "macos")]
        let rd = settings.database.rd_macos;
        #[cfg(target_os = "windows")]
        let rd = settings.database.rd_windows;
        #[cfg(target_os = "linux")]
        let rd = settings.database.rd_linux;

        let mut r = RecordManager::new();
        r.create_table(&(rd.clone() + "alloc_btree_test.rua"));

        let columns = gen_random_columns(&mut gen, 10, MAX_STRING_LENGTH);
        let th = r.open_table(&(rd.clone() + "alloc_btree_test.rua"), false);
        for c in &columns {
            th.insert_column_type(c);
        }
        th.close();

        let mut ptrs = Vec::new();

        let th = r.open_table(&(rd.clone() + "alloc_btree_test.rua"), false);
        for _ in 0..MAX_RECORD_NUMBER {
            let record = gen_record(&mut gen, &columns, MAX_STRING_LENGTH);
            let insert_times: usize = gen.gen_range(1, 2);
            for _ in 0..insert_times {
                ptrs.push(th.insert_record(&record));
            }
        }
        th.close();
        println!("insert records {:?}", SystemTime::now().duration_since(start_time).unwrap().as_millis());

        let th = r.open_table(&(rd.clone() + "alloc_btree_test.rua"), false);
        let btree = BTree::new(&th, vec![0]);
        let btree_ptr = th.__insert_btree(&btree);
        th.close();

        let th = r.open_table(&(rd.clone() + "alloc_btree_test.rua"), false);

        let mut btree_ = th.__get_btree(&btree_ptr);

        for i in 0..ptrs.len() {
            let record = th.get_record(&ptrs[i]);
            let index = RawIndex::from(&record.1.get_index(&th, &btree.index_col));
            btree_.insert_record(&index, ptrs[i].to_u64());
        }
        th.update_btree(&btree_ptr, &btree_);
        th.close();

        println!("btree insert {:?}", SystemTime::now().duration_since(start_time).unwrap().as_millis());

        let th = r.open_table(&(rd.clone() + "alloc_btree_test.rua"), false);
        let btree_ = th.__get_btree(&btree_ptr);
        for i in 0..ptrs.len() {
            let record = th.get_record(&ptrs[i]);
            let index = RawIndex::from(&record.1.get_index(&th, &btree.index_col));
            assert!(btree_.search_record(&index).unwrap().data.contains(&ptrs[i].to_u64()));
        }
        th.update_btree(&btree_ptr, &btree_);
        th.close();

        println!("btree search {:?}", SystemTime::now().duration_since(start_time).unwrap().as_millis());

        let th = r.open_table(&(rd.clone() + "alloc_btree_test.rua"), false);
        let mut btree_ = th.__get_btree(&btree_ptr);
        for i in 0..ptrs.len() {
            let record = th.get_record(&ptrs[i]);
            let index = RawIndex::from(&record.1.get_index(&th, &btree.index_col));
            btree_.delete_record(&index, ptrs[i].to_u64());
            let result = btree_.search_record(&index);
            if result.is_some() {
                assert!(!result.unwrap().data.contains(&ptrs[i].to_u64()));
            }
        }
        th.update_btree(&btree_ptr, &btree_);
        th.close();

        println!("btree delete {:?}", SystemTime::now().duration_since(start_time).unwrap().as_millis());
    }
}
