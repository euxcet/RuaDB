use crate::rm::record_manager::RecordManager;
use crate::logger::logger::RuaResult;
use crate::sm::system_manager::SystemManager;
use crate::parser::ast::*;
use crate::logger::logger::*;
use crate::parser::sql;
use std::cell::RefCell;
use std::rc::Rc;
use std::io;

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
                    AlterStmt::AddColumn { tb_name, field } => sm.add_column(&tb_name, &field),
                    AlterStmt::DropColumn { tb_name, col_name } => sm.drop_column(&tb_name, &col_name),
                    AlterStmt::ChangeColumn { tb_name, col_name, field } => sm.change_column(&tb_name, &col_name, &field),
                    AlterStmt::RenameTable { tb_name, new_name } => sm.rename_table(&tb_name, &new_name),
                    AlterStmt::AddPrimaryKey { tb_name, column_list } => sm.add_primary_key(&tb_name, &column_list),
                    AlterStmt::DropPrimaryKey { tb_name } => sm.drop_primary_key(&tb_name),
                    AlterStmt::AddConstraintPrimaryKey { tb_name, pk_name, column_list } => sm.add_constraint_primary_key(&tb_name, &pk_name, &column_list),
                    AlterStmt::DropConstraintPrimaryKey { tb_name, pk_name } => sm.drop_constraint_primary_key(&tb_name, &pk_name),
                    AlterStmt::AddConstraintForeignKey { tb_name, fk_name, column_list, foreign_tb_name, foreign_column_list } => sm.add_constraint_foreign_key(&tb_name, &fk_name, &column_list, &foreign_tb_name, &foreign_column_list),
                    AlterStmt::DropConstraintForeignKey { tb_name, fk_name } => sm.drop_constraint_foreign_key(&tb_name, &fk_name),
                }
            },
            Stmt::Copy(ref s) => {
                match s {
                    CopyStmt { tb_name, path } => sm.copy(&tb_name, &path),
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

    pub fn process_string(&self, input: &String, logger: &RuaLogger) {
        match sql::parse_sql(&input) {
            Ok(sql) => {
                for stmt in &sql.stmt_list {
                    let res = self.check(stmt);
                    if res.is_ok() {
                        logger.log(&self.execute(stmt));
                    }
                    else {
                        logger.log(&res);
                    }
                }
            },
            Err(e) => {
                logger.log(&RuaResult::err(String::from("Invalid syntax")));
            }
        }
    }

    pub fn process_string_list(&self, cmds: &Vec<String>, logger: &RuaLogger) {
        for cmd in cmds {
            self.process_string(&cmd, logger);
        }
    }

    pub fn process_from_stdin(&self, logger: &RuaLogger) -> bool {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line.");
        if input.trim() == "exit" {
            println!("bye!");
            true
        }
        else {
            self.process_string(&input, logger);
            false
        }
    }

    pub fn process_from_file(&self, path: &str, logger: &RuaLogger) {
        use std::error::Error;
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use std::path::Path;

        let path = Path::new(path);
        let display = path.display();
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            self.process_string(&line, logger);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Executor;
    use crate::logger;

    #[test]
    pub fn sql_select() {
        let executor = Executor::new();
        let logger = logger::logger::RuaLogger::new();
        // executor.process_from_file("sql/bug.rsql", &logger);
        executor.process_from_file("sql/small.rsql", &logger);
    }
}