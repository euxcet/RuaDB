use crate::logger::logger::RuaResult;
use crate::rm::record_manager::*;
use crate::settings;
use std::path::PathBuf;
use std::fs;
use std::cell::RefCell;
use std::rc::Rc;


pub struct SystemManager {
    root_dir: String,
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
            current_database: None,
        }
    }

    pub fn create_database(&self, db_name: String) -> RuaResult{
        assert!(db_name.len() > 0);
        let path: PathBuf = [self.root_dir.clone(), db_name].iter().collect();
        match fs::create_dir_all(path) {
            Ok(()) => {
                RuaResult::default()
            },
            _ => {
                RuaResult::err("database exists".to_string())
            },
        }
    }

    pub fn drop_database(&mut self, db_name: String) -> RuaResult {
        assert!(db_name.len() > 0);
        let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
        if let &Some(ref cdb) = &self.current_database {
            if cdb.as_str() == db_name {
                self.current_database = None;
            }
        }
        match fs::remove_dir_all(path) {
            Ok(()) => {
                RuaResult::default()
            },
            _ => {
                RuaResult::err("database doesn't exist".to_string())
            },
        }
    }

    pub fn use_database(&mut self, db_name: String) -> RuaResult {
        let path: PathBuf = [self.root_dir.clone(), db_name.clone()].iter().collect();
        if path.is_dir() {
            self.current_database = Some(db_name);
            RuaResult::ok(None, "change database".to_string())
        } else {
            RuaResult::err("database doesn't exist".to_string())
        }
    }

    pub fn show_databases(&self) -> RuaResult {
        let path: PathBuf = PathBuf::from(self.root_dir.clone());
        let mut t = vec![vec!["Databases".to_owned()]];
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                t.push(vec![entry.file_name().into_string().unwrap()]);
            }
        }
        let count = t.len() - 1;

        RuaResult::ok(Some(t), format!("{} row(s) in set", count))
    }
}






