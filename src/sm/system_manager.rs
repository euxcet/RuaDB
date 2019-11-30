use crate::logger::logger::RuaResult;
use crate::rm::record_manager::*;
use crate::settings;
use crate::parser::ast::*;
use crate::rm::table_handler::TableHandler;

use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::cell::RefCell;
use std::rc::Rc;


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

    pub fn check(&mut self, check: bool) {
        self.check = check;
    }

    pub fn create_database(&self, db_name: &String) -> RuaResult {
        if self.check {
            assert!(db_name.len() > 0);
            let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
            if path.exists() {
                return RuaResult::err("database exists".to_string());
            }
            return RuaResult::default();
        } 

        let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
        assert!(fs::create_dir(path).is_ok());
        RuaResult::default()
    }

    pub fn drop_database(&mut self, db_name: &String) -> RuaResult {
        if self.check {
            assert!(db_name.len() > 0);
            let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
            if !path.is_dir() {
                return RuaResult::err("database doesn't exist".to_string());
            } 
            return RuaResult::default();
        } 

        let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
        if let Some(ref cdb) = self.current_database {
            if cdb == db_name {
                self.current_database = None;
            }
        }
        assert!(fs::remove_dir_all(path).is_ok());
        RuaResult::default()
    }

    pub fn use_database(&mut self, db_name: &String) -> RuaResult {
        if self.check {
            assert!(db_name.len() > 0);
            let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
            if !path.is_dir() {
                return RuaResult::err("database doesn't exist".to_string());
            }
            return RuaResult::default();
        } 

        self.current_database = Some(db_name.clone());
        RuaResult::ok(None, "database changed".to_string())
    }

    pub fn show_databases(&self) -> RuaResult {
        if self.check {
            return RuaResult::default();
        }

        let path: PathBuf = PathBuf::from(self.root_dir.clone());
        let mut t = vec![vec!["Databases".to_string()]];

        let databases: Vec<String> = fs::read_dir(path).unwrap()
            .filter(|e| e.as_ref().unwrap().path().is_dir())
            .map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        let count = databases.len();

        t.push(vec![databases.join("\n")]);

        RuaResult::ok(Some(t), format!("{} row(s) in set", count))
    }

    pub fn show_tables(&self) -> RuaResult {
        if self.check {
            if let Some(ref cdb) = self.current_database {
                let path: PathBuf = [self.root_dir.clone(), cdb.clone()].iter().collect();
                assert!(path.is_dir());
                return RuaResult::default();
            } else {
                return RuaResult::err("not use any database".to_string());
            }
        }

        let cdb = self.current_database.as_ref().unwrap();
        let path: PathBuf = [self.root_dir.clone(), cdb.clone()].iter().collect();
        let mut t = vec![vec!["Tables".to_string()]];

        let tables: Vec<String> = fs::read_dir(path).unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_file() && p.extension().unwrap() == "rua")
            .map(|p| p.file_stem().unwrap().to_str().unwrap().to_string()).collect();

        let count = tables.len();
        t.push(vec![tables.join("\n")]);

        RuaResult::ok(Some(t), format!("{} row(s) in set", count))
    }

    pub fn create_table(&self, tb_name: &String, field_list: &Vec<Field>) -> RuaResult {
        if self.check {
            if let Some(ref cdb) = self.current_database {
                let mut path: PathBuf = [self.root_dir.clone(), cdb.clone(), tb_name.clone()].iter().collect();
                path.set_extension("rua");
                if path.is_file() {
                    return RuaResult::err("table exists".to_string());
                }
            } else {
                return RuaResult::err("not use any database".to_string());
            }
        }
        RuaResult::default()
    }

    pub fn open_table(&self, tb_name: &String) -> Option<TableHandler>{
        if let Some(ref cdb) = self.current_database {
            let mut path: PathBuf = [self.root_dir.clone(), cdb.clone(), tb_name.clone()].iter().collect();
            path.set_extension("rua");
            let th = self.rm.borrow_mut().open_table(path.to_str().unwrap());
            Some(th)
        } else {
            return None;
        }
    }
}






