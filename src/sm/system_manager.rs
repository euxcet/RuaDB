use crate::logger::logger::RuaResult;
use crate::rm::record::*;
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
    root_dir: String,
    check: bool,
    current_database: Option<String>,
    rm: Rc<RefCell<RecordManager>>,
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
            let tables: Vec<String> = fs::read_dir(path).unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.is_file() && p.extension().unwrap() == "rua")
                .map(|p| p.file_stem().unwrap().to_str().unwrap().to_string()).collect();
            let count = tables.len();
            res.push(vec![tables.join("\n")]);
            RuaResult::ok(Some(res), format!("{} row(s) in set", count))
        }
    }

    // TODO: foreign key
    pub fn create_table(&self, tb_name: &String, field_list: &Vec<Field>) -> RuaResult {
        if self.check {
            let res = self.check_table_existence(tb_name, false);
            if res.is_ok() {
                if check::check_field_list(field_list, &self) {
                    RuaResult::default()
                } else {
                    RuaResult::err("invalid field".to_string())
                }
            } else {
                res
            }
        }
        else {
            let columns = ColumnTypeVec::from_fields(field_list, tb_name);
            let primary_index = columns.get_primary_index();

            let th = self.open_table(tb_name, true).unwrap();
            th.insert_column_types(&columns);
            th.init_btrees();
            th.insert_born_btree(&BTree::new(&th, vec![], ""));
            if primary_index.len() > 0 {
                th.insert_btree(&BTree::new(&th, primary_index, "primary"));
            }
            th.close();
            RuaResult::ok(None, "table created".to_string())
        }
    }

    // TODO: foreign key
    pub fn drop_table(&self, tb_name: &String) -> RuaResult {
        if self.check {
            self.check_table_existence(tb_name, true)
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
            let title = vec!["Field", "Type", "Null", "Key", "Default"].iter().map(|x| x.to_string()).collect();
            let print_content = cts.print();
            RuaResult::ok(Some(vec![title, print_content]), format!("{} row(s) in set", cts.cols.len()))
        }
    }

    pub fn insert(&self, tb_name: &String, value_lists: &Vec<Vec<Value>>) -> RuaResult {
        if self.check {
            let res = self.check_table_existence(tb_name, true);
            let th = self.open_table(tb_name, false).unwrap();
            let cts = th.get_column_types();
            th.close();
            if res.is_ok() {
                if check::check_insert_value(value_lists, &cts) {
                    RuaResult::default()
                } else {
                    RuaResult::err("invalid insert values".to_string())
                }
            } else {
                res
            }
        }
        else {
            // let database = self.current_database.as_ref().unwrap();
            // let th = self.rm.borrow_mut().open_table(self.get_table_path(database, tb_name).to_str().unwrap(), false);
            let th = self.open_table(tb_name, false).unwrap();
            let records: Vec<Record> = value_lists.iter().map(|v| Record::from_value_lists(v)).collect();
            let ptrs: Vec<StrPointer> = records.iter().map(|record| th.insert_record(record)).collect();

            // TODO: insert into every btree
            let mut born_btree = th.get_born_btree();
            for ptr in ptrs {
                born_btree.insert_record(&RawIndex::from_u64(ptr.to_u64()), ptr.to_u64());
            }
            th.update_born_btree(&born_btree);
            th.close();
            RuaResult::ok(None, format!("{} row affected", records.len()))
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

                let valid = check::check_delete(tb_name, &map, where_clause);
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

            // TODO: delete in every btree
            let th = self.open_table(tb_name, false).unwrap();
            let mut born_btree = th.get_born_btree();
            for ptr in &record_list.ptrs {
                born_btree.delete_record(&RawIndex::from_u64(ptr.to_u64()), ptr.to_u64());
            }
            th.update_born_btree(&born_btree);
            th.close();
            RuaResult::default()
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

                let valid = check::check_update(tb_name, &map, set_clause, where_clause);
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
            let mut record_list = tree.query();

            let th = self.open_table(tb_name, false).unwrap();
            for pair in record_list.record.iter_mut().zip(record_list.ptrs.iter()) {
                let (record, ptr) = pair;
                record.set_(set_clause, &record_list.ty);
                th.update_record(ptr, record);
            }
            th.close();
            // TODO: update in every related btree
            RuaResult::default()
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
            let th = self.open_table(tb_name, false).unwrap();
            let map = th.get_column_types_as_hashmap();
            let index_col: Vec<u32> = column_list.iter().map(|column_name| map.get(column_name).unwrap().index).collect();
            let mut btree = BTree::new(&th, index_col.clone(), idx_name);

            let database = self.current_database.as_ref().unwrap();
            let mut tree = QueryTree::new(&self.root_dir, database, self.rm.clone());
            tree.build(&vec![tb_name.clone()], &Selector::All, &None);

            let record_list = tree.query();
            for ptr in &record_list.ptrs {
                let (_, record_in_file) = th.get_record(ptr);
                let record_index = record_in_file.get_index(&th, &index_col);
                btree.insert_record(&RawIndex::from(&record_index), ptr.to_u64());
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
}