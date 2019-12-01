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
    pub fn new(rm: Rc<RefCell<RecordManager>>, sm:Rc<RefCell<SystemManager>>) -> Self {
        // let rm = Rc::new(RefCell::new(RecordManager::new()));
        // let sm = Rc::new(RefCell::new(SystemManager::new(rm.clone())));
        Self {
            rm: rm,
            sm: sm,
        }
    }

    fn process(&self, stmt: &Stmt, check: bool) -> RuaResult {
        self.sm.borrow_mut().check(check);
        match stmt {
            Stmt::System(SystemStmt::ShowDatabases) => self.sm.borrow_mut().show_databases(),
            Stmt::Database(ref s) => {
                match s {
                    DatabaseStmt::CreateDatabase { db_name } => self.sm.borrow_mut().create_database(&db_name),
                    DatabaseStmt::DropDatabase { db_name } => self.sm.borrow_mut().drop_database(&db_name),
                    DatabaseStmt::UseDatabase { db_name } => self.sm.borrow_mut().use_database(&db_name),
                    DatabaseStmt::ShowTables => self.sm.borrow_mut().show_tables(),
                }
            },
            Stmt::Table(ref s) => {
                match s {
                    TableStmt::CreateTable { tb_name, field_list } => self.sm.borrow_mut().create_table(&tb_name, &field_list),
                    TableStmt::DropTable { tb_name } => self.sm.borrow_mut().drop_table(&tb_name),
                    TableStmt::Desc { tb_name } => self.sm.borrow_mut().desc(&tb_name),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn execute(&self, stmt: &Stmt) -> RuaResult {
        self.process(stmt, false)
    }

    pub fn check(&self, stmt: &Stmt) -> RuaResult {
        self.process(stmt, true)
    }

}
