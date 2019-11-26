use crate::rm::record_manager::RecordManager;
use crate::logger::logger::RuaResult;
use crate::sm::system_manager::SystemManager;
use crate::parser::ast::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Executor {
    sm: Rc<RefCell<SystemManager>>,
    rm: Rc<RefCell<RecordManager>>,
}

impl Executor {
    pub fn new() -> Self {
        let rm = Rc::new(RefCell::new(RecordManager::new()));
        let sm = Rc::new(RefCell::new(SystemManager::new(rm.clone())));
        Self {
            rm: rm,
            sm: sm,
        }
    }

    pub fn exe(&mut self, stmt: &Stmt) -> RuaResult {
        match stmt {
            Stmt::System(SystemStmt::ShowDatabases) => self.sm.borrow_mut().show_databases(),
            Stmt::Database(s) => {
                match s {
                    DatabaseStmt::CreateDatabase { db_name } => self.sm.borrow_mut().create_database(db_name.to_string()),
                    DatabaseStmt::DropDatabase { db_name } => self.sm.borrow_mut().create_database(db_name.to_string()),
                    DatabaseStmt::UseDatabase { db_name } => self.sm.borrow_mut().create_database(db_name.to_string()),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
