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
        let mut sm = self.sm.borrow_mut();
        sm.set_check(check);
        match stmt {
            Stmt::System(SystemStmt::ShowDatabases) => sm.show_databases(),
            Stmt::Database(ref s) => {
                match s {
                    DatabaseStmt::CreateDatabase { db_name } => sm.create_database(&db_name),
                    DatabaseStmt::DropDatabase { db_name } => sm.drop_database(&db_name),
                    DatabaseStmt::UseDatabase { db_name } => sm.use_database(&db_name),
                    DatabaseStmt::ShowTables => sm.show_tables(),
                }
            },
            Stmt::Table(ref s) => {
                match s {
                    TableStmt::CreateTable { tb_name, field_list } => sm.create_table(&tb_name, &field_list),
                    TableStmt::DropTable { tb_name } => sm.drop_table(&tb_name),
                    TableStmt::Desc { tb_name } => sm.desc(&tb_name),
                    TableStmt::Insert { tb_name, value_lists } => sm.insert(&tb_name, &value_lists),
                    TableStmt::Select { table_list, selector, where_clause } => sm.select(&table_list, &selector, &where_clause),
                    TableStmt::Delete { tb_name, where_clause } => sm.delete(&tb_name, &where_clause),
                    TableStmt::Update { tb_name, set_clause, where_clause } => sm.update(&tb_name, &set_clause, &where_clause),
                }
            },
            Stmt::Index(ref s) => {
                match s {
                    IndexStmt::CreateIndex { idx_name, tb_name, column_list } => sm.create_index(&idx_name, &tb_name, &column_list),
                    IndexStmt::DropIndex { idx_name, tb_name } => sm.drop_index(&idx_name, &tb_name),
                    IndexStmt::AlterAddIndex { idx_name, tb_name, column_list } => sm.create_index(&idx_name, &tb_name, &column_list),
                    IndexStmt::AlterDropIndex { idx_name, tb_name } => sm.drop_index(&idx_name, &tb_name),
                }
            },
            Stmt::Alter(ref s) => {
                match s {
                    AlterStmt::AddColumn { tb_name, field } => sm.add_column(tb_name, field),
                    AlterStmt::DropColumn { tb_name, col_name } => sm.drop_column(tb_name, col_name),
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

            String::from("create table test(id int(4) default 3, fuck varchar(10));"),
            String::from("insert into test values (-10, \"123124\");"),
            String::from("insert into test values (40, \"224\");"),
            String::from("insert into test values (3, \"23124\");"),
            String::from("insert into test values (4, \"12324\");"),

            String::from("create table b_test(id int(4), rua float);"),
            String::from("insert into b_test values (1, 1.23);"),
            String::from("insert into b_test values (2, 3.0);"),
            String::from("insert into b_test values (3, 42.12);"),
            String::from("insert into b_test values (4, -4.2);"),

            String::from("select * from test where id > 0;"),
            String::from("select * from test where id < -1;"),
            String::from("select * from test where id > 12;"),
            String::from("select * from test where id > 12 and id < 32;"),
            String::from("select * from b_test;"),
            String::from("select * from test, b_test where test.id >= b_test.id;"),
            String::from("select fuck from test;"),

            String::from("select * from test;"),
            String::from("delete from test where id > 5;"),
            String::from("select * from test;"),
            String::from("delete from test where id < 4;"),
            String::from("select * from test;"),

            String::from("insert into test values (-10, \"123124\");"),
            String::from("insert into test values (40, \"224\");"),
            String::from("insert into test values (3, \"23124\");"),

            String::from("update test set id = 10 where id > 3;"),
            String::from("select * from test;"),
            
            String::from("desc test;"),
            String::from("drop database sql_select;"),
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
                Err(_) => {
                    println!("Invalid syntax");
                }
            }
        }
    }
}