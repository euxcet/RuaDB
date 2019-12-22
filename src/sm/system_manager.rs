use crate::logger::logger::RuaResult;
use crate::rm::record::*;
use crate::rm::in_file::*;
use crate::rm::pagedef::*;
use crate::rm::record_manager::*;
use crate::rm::table_handler::TableHandler;
use crate::settings;
use crate::parser::ast::*;
use crate::index::btree::*;

use super::query_tree::*;
use super::check;

use std::path::PathBuf;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;


pub struct SystemManager {
    pub root_dir: String,
    pub check: bool,
    pub current_database: Option<String>,
    pub rm: Rc<RefCell<RecordManager>>,
}

impl SystemManager {
    pub fn new(rm: Rc<RefCell<RecordManager>>) -> Self {
        let settings = settings::Settings::new().unwrap();

        #[cfg(target_os = "macos")]
        let rd = settings.database.rd_macos;
        #[cfg(target_os = "windows")]
        let rd = settings.database.rd_windows;
        #[cfg(target_os = "linux")]
        let rd = settings.database.rd_linux;

        Self {
            rm: rm,
            root_dir: rd,
            check: false,
            current_database: None,
        }
    }

    pub fn set_check(&mut self, check: bool) {
        self.check = check;
    }

    pub fn get_database_path(&self, db_name: &String) -> PathBuf {
        let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
        path
    }

    pub fn get_table_path(&self, database: &String, tb_name: &String) -> PathBuf {
        let mut path: PathBuf = [self.root_dir.clone(), database.clone(), tb_name.clone()].iter().collect();
        path.set_extension("rua");
        path
    }

    pub fn get_tables(&self) -> Vec<String> {
        assert!(self.current_database.is_some());
        let dir: PathBuf = [self.root_dir.clone(), self.current_database.as_ref().unwrap().clone()].iter().collect();
        fs::read_dir(dir).unwrap().filter_map(
            |e| {
                let p = e.unwrap().path();
                if p.extension().unwrap() == "rua" {
                    Some(p.file_stem().unwrap().to_str().unwrap().to_string())
                } else {
                    None
                }
            }
        ).collect()
    }

    pub fn open_table(&self, tb_name: &String, create: bool) -> Option<TableHandler>{
        match &self.current_database {
            Some(database) => {
                let path = self.get_table_path(database, tb_name);
                let th = self.rm.borrow_mut().open_table(path.to_str().unwrap(), create);
                Some(th)
            },
            None => None,
        }
    }
    // TODO: Merge check and execute

    fn check_database_existence(&self, db_name: &String, should_exist: bool) -> RuaResult {
        assert!(db_name.len() > 0);
        let path = self.get_database_path(db_name);
        if path.exists() && !should_exist {
            RuaResult::err("database exists".to_string())
        }
        else if !path.exists() && should_exist {
            RuaResult::err("database doesn't exist".to_string())
        }
        else {
            RuaResult::default()
        }
    }

    fn check_table_existence(&self, tb_name: &String, should_exist: bool) -> RuaResult {
        if let Some(ref cdb) = self.current_database {
            let mut path: PathBuf = [self.root_dir.clone(), cdb.clone(), tb_name.clone()].iter().collect();
            path.set_extension("rua");
            if !path.is_file() && should_exist {
                RuaResult::err("table doesn't exist".to_string())
            }
            else if path.is_file() && !should_exist {
                RuaResult::err("table exists".to_string())
            }
            else {
                RuaResult::default()
            }
        }
        else {
            RuaResult::err("not use any database".to_string())
        }
    }

    pub fn create_database(&self, db_name: &String) -> RuaResult {
        if self.check {
            self.check_database_existence(db_name, false)
        } 
        else {
            let path = self.get_database_path(db_name);
            assert!(fs::create_dir(path).is_ok());
            RuaResult::ok(None, "database created".to_string())
        }
    }

    pub fn drop_database(&mut self, db_name: &String) -> RuaResult {
        if self.check {
            self.check_database_existence(db_name, true)
        } 
        else {
            let path = self.get_database_path(db_name);
            if self.current_database == Some(db_name.clone()) {
                self.current_database = None;
            }
            assert!(fs::remove_dir_all(path).is_ok());
            RuaResult::ok(None, "database dropped".to_string())
        }
    }

    pub fn use_database(&mut self, db_name: &String) -> RuaResult {
        if self.check {
            self.check_database_existence(db_name, true)
        } 
        else {
            self.current_database = Some(db_name.clone());
            RuaResult::ok(None, "database changed".to_string())
        }
    }

    pub fn show_databases(&self) -> RuaResult {
        if self.check {
            RuaResult::default()
        }
        else {
            let path: PathBuf = PathBuf::from(self.root_dir.clone());
            let mut res = vec![vec!["Databases".to_string()]];
            let databases: Vec<String> = fs::read_dir(path).unwrap()
                .filter(|e| e.as_ref().unwrap().path().is_dir())
                .map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
            let count = databases.len();
            res.push(vec![databases.join("\n")]);
            RuaResult::ok(Some(res), format!("{} row(s) in set", count))
        }
    }

    pub fn show_tables(&self) -> RuaResult {
        if self.check {
            match &self.current_database {
                Some(database) => self.check_database_existence(database, true),
                None => RuaResult::err("not use any database".to_string())
            }
        }
        else {
            let database = self.current_database.as_ref().unwrap();
            let path = self.get_database_path(database);
            let mut res = vec![vec![format!("Tables_in_{}", database)]];
            let tables: Vec<String> = self.get_tables();
            let count = tables.len();
            res.push(vec![tables.join("\n")]);
            RuaResult::ok(Some(res), format!("{} row(s) in set", count))
        }
    }

    pub fn create_table(&self, tb_name: &String, field_list: &Vec<Field>) -> RuaResult {
        if self.check {
            let res = self.check_table_existence(tb_name, false);
            if res.is_err() {
                res
            } else {
                if check::check_create_table(field_list, &self) {
                    RuaResult::default()
                } else {
                    RuaResult::err("invalid field".to_string())
                }
            }
        }
        else {
            let (columns, primary_cols, foreign_indexes) = ColumnTypeVec::from_fields(field_list, tb_name);
            let th = self.open_table(tb_name, true).unwrap();
            th.insert_column_types(&columns);
            th.init_btrees();
            th.insert_born_btree(&BTree::new(&th, vec![], "", 0));
            if primary_cols.len() > 0 {
                th.insert_btree(&BTree::new(&th, primary_cols, "PRIMARY", BTree::primary_ty()));
            }

            for (ft_name, foreign_index) in foreign_indexes {
                th.insert_btree(&BTree::new(&th, foreign_index, format!("FOREIGN_{}", ft_name).as_str(), BTree::foreign_ty()));
            }

            th.close();
            RuaResult::ok(None, "table created".to_string())
        }
    }

    pub fn drop_table(&self, tb_name: &String) -> RuaResult {
        if self.check {
            let res = self.check_table_existence(tb_name, false);
            if res.is_err() {
                res
            } else {
                if check::check_drop_table(tb_name, &self) {
                    RuaResult::default()
                } else {
                    RuaResult::err("invalid drop table".to_string())
                }
            }
        }
        else {
            let database = self.current_database.as_ref().unwrap();
            let path = self.get_table_path(database, tb_name);
            assert!(fs::remove_file(path).is_ok());
            RuaResult::ok(None, "table dropped".to_string())
        }
    }

    pub fn desc(&self, tb_name: &String) -> RuaResult {
        if self.check {
            self.check_table_existence(tb_name, true)
        }
        else {
            let th = self.open_table(tb_name, false).unwrap();
            let cts = th.get_column_types();
            th.close();
            let title = vec!["Field", "Type", "Null", "Key", "Default", "Foreign"].iter().map(|x| x.to_string()).collect();
            let print_content = cts.print();
            RuaResult::ok(Some(vec![title, print_content]), format!("{} row(s) in set", cts.cols.len()))
        }
    }

    pub fn insert(&self, tb_name: &String, value_lists: &Vec<Vec<Value>>) -> RuaResult {
        if self.check {
            let res = self.check_table_existence(tb_name, true);
            if res.is_err() {
                res
            } else {
                if check::check_insert_value(tb_name, value_lists, &self) {
                    RuaResult::default()
                } else {
                    RuaResult::err("invalid insert values".to_string())
                }
            }
        }
        else {
            let th = self.open_table(tb_name, false).unwrap();
            let cts = th.get_column_types();
            let records: Vec<Record> = value_lists.iter().map(|v| Record::from_value_lists(v, &cts.cols)).collect();
            let ptrs: Vec<(StrPointer, RecordInFile)> = records.iter().map(|record| th.insert_record_get_record_in_file(record)).collect();

            let mut born_btree = th.get_born_btree();
            let mut btrees = th.get_btrees_with_ptrs();
            for (ptr, rif) in ptrs {
                born_btree.insert_record(&RawIndex::from_u64(ptr.to_u64()), ptr.to_u64(), true);
                // TODO: take advantage of cache
                for (_, btree) in &mut btrees {
                    btree.insert_record(&RawIndex::from(&rif.get_index(&th, &btree.index_col)), ptr.to_u64(), true);
                }
            }

            th.update_born_btree(&born_btree);
            for (p, btree) in &btrees {
                th.update_btree(p, btree);
            }

            th.close();
            RuaResult::ok(None, format!("{} rows affected", records.len()))
        }
    }

    pub fn select(&self, table_list: &Vec<Name>, selector: &Selector, where_clause: &Option<Vec<WhereClause>>) -> RuaResult {
        if self.check {
            let repeat = !check::check_no_repeat(table_list);
            if repeat {
                return RuaResult::err("a single table cannot be selected twice".to_string())
            }
            let res = table_list.iter().map(|tb_name| self.check_table_existence(tb_name, true)).fold(RuaResult::default(), |s, v| s & v);
            if res.is_err() {
                res
            } else {
                let name_cols = table_list.iter()
                                .map(|tb_name| {
                                    let th = self.open_table(tb_name, false).unwrap();
                                    let map = th.get_column_types_as_hashmap();
                                    th.close();
                                    (tb_name, map)
                                }).collect();
                let valid = check::check_select(&name_cols, selector, where_clause);
                if !valid {
                    RuaResult::err("invalid select".to_string())
                } else {
                    RuaResult::default()
                }
            }
        }
        else {
            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(table_list, selector, where_clause);
            let record_list = tree.query();
            let record_num = record_list.record.len();
            if record_num == 0 {
                RuaResult::ok(None, format!("Empty set"))
            }
            else if record_num == 1 {
                let print_content = record_list.print();
                RuaResult::ok(Some(print_content), format!("{} row in set", record_num))
            }
            else {
                let print_content = record_list.print();
                RuaResult::ok(Some(print_content), format!("{} rows in set", record_num))
            }
        }
    }

    pub fn delete(&self, tb_name: &String, where_clause: &Option<Vec<WhereClause>>) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let th = self.open_table(tb_name, false).unwrap();
                let map = th.get_column_types_as_hashmap();
                th.close();

                let valid = check::check_delete(tb_name, &map, where_clause, &self);
                if !valid {
                    RuaResult::err("invalid delete".to_string())
                } else {
                    RuaResult::default()
                }
            }
        }
        else {
            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(&vec![tb_name.clone()], &Selector::All, where_clause);
            let record_list = tree.query();

            let th = self.open_table(tb_name, false).unwrap();
            let mut born_btree = th.get_born_btree();
            let mut btrees = th.get_btrees_with_ptrs();
            for (ptr, record) in record_list.ptrs.iter().zip(record_list.record.iter()) {
                born_btree.delete_record(&RawIndex::from_u64(ptr.to_u64()), ptr.to_u64());
                for (_, btree) in &mut btrees {
                    btree.delete_record(&RawIndex::from_record(record, &btree.index_col), ptr.to_u64());
                }
                th.delete(&ptr);
            }
            th.update_born_btree(&born_btree);
            for (p, btree) in &btrees {
                th.update_btree(p, btree);
            }

            th.close();
            RuaResult::ok(None, format!("{} rows affected", record_list.ptrs.len()))
        }
    }

    pub fn update(&self, tb_name: &String, set_clause: &Vec<SetClause>, where_clause: &Option<Vec<WhereClause>>) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let th = self.open_table(tb_name, false).unwrap();
                let map = th.get_column_types_as_hashmap();
                th.close();

                let valid = check::check_update(tb_name, &map, set_clause, where_clause, self);
                if !valid {
                    RuaResult::err("invalid update".to_string())
                } else {
                    RuaResult::default()
                }
            }
        }
        else {
            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(&vec![tb_name.clone()], &Selector::All, where_clause);
            let record_list = tree.query();

            let th = self.open_table(tb_name, false).unwrap();
            let mut affected_btrees = th.get_affected_btrees_with_ptrs(&set_clause.iter().map(|s| &s.col_name).collect());

            let l = record_list.ptrs.len();

            for (ptr, mut record) in record_list.ptrs.into_iter().zip(record_list.record.into_iter()) {
                let origin_record = record.clone();
                record.set_(set_clause, &record_list.ty);

                for (_, btree) in &mut affected_btrees {
                    btree.delete_record(&RawIndex::from_record(&origin_record, &btree.index_col), ptr.to_u64());
                    btree.insert_record(&RawIndex::from_record(&record, &btree.index_col), ptr.to_u64(), true);
                }
                th.update_record(&ptr, &record);
            }

            for (p, btree) in &affected_btrees {
                th.update_btree(p, btree);
            }

            th.close();
            RuaResult::ok(None, format!("{} rows affected", l))
        }
    }

    pub fn create_index(&self, idx_name: &String, tb_name: &String, column_list: &Vec<String>) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let th = self.open_table(tb_name, false).unwrap();
                let map = th.get_column_types_as_hashmap();
                let btrees = th.get_btrees();
                th.close();

                let valid = check::check_create_index(idx_name, &map, column_list, &btrees);
                if !valid {
                    RuaResult::err("invalid create index".to_string())
                } else {
                    RuaResult::default()
                }
            }
        }
        else {
            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(&vec![tb_name.clone()], &Selector::All, &None);
            let record_list = tree.query();

            let th = self.open_table(tb_name, false).unwrap();
            let map = th.get_column_types_as_hashmap();
            let index_col: Vec<u32> = column_list.iter().map(|column_name| map.get(column_name).unwrap().index).collect();
            let mut btree = BTree::new(&th, index_col.clone(), idx_name, BTree::index_ty());

            for ptr in &record_list.ptrs {
                let (_, record_in_file) = th.get_record(ptr);
                let record_index = record_in_file.get_index(&th, &index_col);
                btree.insert_record(&RawIndex::from(&record_index), ptr.to_u64(), true);
            }
            th.insert_btree(&btree);
            th.close();
            RuaResult::ok(None, "index created".to_string())
        }
    }

    pub fn drop_index(&self, idx_name: &String, tb_name: &String) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let th = self.open_table(tb_name, false).unwrap();
                let btrees = th.get_btrees();
                th.close();

                let valid = check::check_drop_index(idx_name, &btrees);
                if !valid {
                    RuaResult::err("invalid drop index".to_string())
                } else {
                    RuaResult::default()
                }
            }
        } else {
            let th = self.open_table(tb_name, false).unwrap();
            let btrees = th.get_btrees();
            let i = btrees.iter().position(|t| &t.index_name == idx_name).unwrap();
            btrees[i].clear();
            th.delete_btree_from_index(i);
            th.close();
            RuaResult::ok(None, "index created".to_string())
        }
    }

    pub fn add_column(&self, tb_name: &String, field: &Field) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let th = self.open_table(tb_name, false).unwrap();
                let map = th.get_column_types_as_hashmap();
                th.close();

                let valid = check::check_add_column(&map, field);
                if !valid {
                    RuaResult::err("invalid add column".to_string())
                } else {
                    RuaResult::default()
                }
            }
        } else {
            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(&vec![tb_name.clone()], &Selector::All, &None);

            let th = self.open_table(tb_name, false).unwrap();
            let index = th.get_column_numbers() as u32;
            let new_column = ColumnType::from_field(tb_name, index, field); 
            th.insert_column_type(&new_column);

            let record_list = tree.query();
            for ptr in &record_list.ptrs {
                th.insert_record_data_column(ptr, &new_column);
            }

            th.close();
            RuaResult::ok(None, "column added".to_string())
        }
    }

    pub fn drop_column(&self, tb_name: &String, col_name: &String) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let valid = check::check_drop_column(tb_name, col_name, self);
                if !valid {
                    RuaResult::err("invalid drop column".to_string())
                } else {
                    RuaResult::default()
                }
            }
        } else {
            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(&vec![tb_name.clone()], &Selector::All, &None);

            let th = self.open_table(tb_name, false).unwrap();
            let index = th.get_column_types().cols.iter().position(|ct| &ct.name == col_name).unwrap();

            th.delete_column_type_from_index(index);

            let mut deleted_btree_index: Vec<usize> = Vec::new();
            let mut btrees = th.get_btrees_with_ptrs();
            for (i, (ptr, btree)) in (0..).zip(btrees.iter_mut()) {
                if btree.index_col == vec![index as u32] {
                    deleted_btree_index.push(i);
                } else {
                    let mut update = false;
                    for ci in &mut btree.index_col {
                        if *ci > index as u32 {
                            *ci -= 1;
                            update = true;
                        }
                    }
                    if update {
                        th.update_btree(&ptr, &btree);
                    }
                }
            } 

            while let Some(i) = deleted_btree_index.pop() {
                th.delete_btree_from_index(i);
            }

            let record_list = tree.query();
            for ptr in &record_list.ptrs {
                th.delete_record_data_column(ptr, index);
            }

            th.close();
            RuaResult::ok(None, "column deleted".to_string())
        }
    }

    pub fn change_column(&self, tb_name: &String, col_name: &Name, field: &Field) -> RuaResult {
        // TODO: ensure column type can be converted
        // TODO: out of range value
        unimplemented!();
    }

    pub fn rename_table(&self, tb_name: &String, new_name: &String) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let non_exist = self.check_table_existence(new_name, false);
                if non_exist.is_err() {
                    non_exist
                } else {
                    RuaResult::default()
                }
            }
        } else {
            let th = self.open_table(tb_name, false).unwrap();
            th.update_table_name(new_name);
            let path = self.get_table_path(self.current_database.as_ref().unwrap(), tb_name);
            let new_path = self.get_table_path(self.current_database.as_ref().unwrap(), new_name);
            th.close();

            for table in self.get_tables() {
                if &table != tb_name {
                    let fth = self.open_table(&table, false).unwrap();
                    for (ptr, mut ct) in fth.get_column_types_with_ptrs() {
                        if &ct.foreign_table_name == tb_name {
                            ct.foreign_table_name = new_name.clone();
                            fth.update_column_type(&ptr, &ct);
                        }
                    }
                    for (ptr, mut bt) in fth.get_btrees_with_ptrs() {
                        if bt.is_foreign() && bt.index_name == format!("FOREIGN_{}", tb_name) {
                            bt.index_name = format!("FOREIGN_{}", new_name);
                            fth.update_btree(&ptr, &bt);
                        }
                    }
                    fth.close();
                }
            }
            fs::rename(path, new_path).ok();

            RuaResult::default()
        }
    }

    pub fn add_primary_key(&self, tb_name: &String, column_list: &Vec<String>) -> RuaResult {
        if self.check {
            let exist = self.check_table_existence(tb_name, true);
            if exist.is_err() {
                exist
            } else {
                let th = self.open_table(tb_name, false).unwrap();
                let map = th.get_column_types_as_hashmap();
                let valid = check::check_add_primary_key(&map, column_list);
                if !valid {
                    RuaResult::err("invalid add primary key".to_string())
                } else {
                    RuaResult::default()
                }
            }
        } else {
            // TODO: add index and primary key
            RuaResult::default()
        }
    }
}