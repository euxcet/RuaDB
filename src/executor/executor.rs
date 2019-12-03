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
        self.sm.borrow_mut().set_check(check);
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
                    TableStmt::Insert { tb_name, value_lists } => self.sm.borrow_mut().insert(&tb_name, &value_lists),
                    TableStmt::Select { table_list, selector, where_clause } => self.sm.borrow_mut().select(&table_list, &selector, &where_clause),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::logger;
    use crate::executor;
    use crate::parser::sql;

    #[test]
    pub fn sql_select() {
        let logger = logger::logger::RuaLogger::new();
        let rm = Rc::new(RefCell::new(RecordManager::new()));
        let sm = Rc::new(RefCell::new(SystemManager::new(rm.clone())));
        let executor = executor::executor::Executor::new(rm.clone(), sm.clone());

        let cmds = vec![
            String::from("create database sql_select;"),
            String::from("use sql_select;"),
            String::from("create table test(id int(4) default 3);"),
            String::from("insert into test values (10);"),
            String::from("desc test;"),
            String::from("drop database sql_select;"),
            // String::from("select * from sql_select;"),
        ];

        for cmd in cmds {
            match sql::parse_sql(&cmd) {
                Ok(sql) => {
                    for stmt in &sql.stmt_list {
                        let res = executor.check(stmt);
                        if res.is_ok() {
                            let res = executor.execute(stmt);
                            logger.log(&res);
                        } else {
                            logger.log(&res);
                        }
                    }
                },
                Err(e) => {
                    println!("Invalid syntax");
                }
            }
        }
    }
}