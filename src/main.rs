extern crate lalrpop_util;
extern crate config;
#[macro_use]
pub mod bytevec;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate prettytable;

// pub mod parser;
// pub mod ast;
// pub mod tok;
// pub mod sql;
mod utils;
mod index;
mod rm;
mod sm;
mod logger;
mod settings;
mod parser;
mod executor;

use settings::Settings;
use std::io;
use std::io::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use parser::sql;

fn initalize() {
    println!("Connected to server.");
    let settings = Settings::new();
    println!("{:?}", settings);
}

fn print_prompt() {
    print!("rua > ");
    io::stdout().flush().ok().expect("Could not flush stdout.");
}


fn main() {
    use crate::sm::system_manager::SystemManager;
    use crate::rm::record_manager::RecordManager;

    initalize();

    let logger = logger::logger::RuaLogger::new();
    let rm = Rc::new(RefCell::new(RecordManager::new()));
    let sm = Rc::new(RefCell::new(SystemManager::new(rm.clone())));
    let executor = executor::executor::Executor::new(rm.clone(), sm.clone());
    // let mut checker = executor::checker::Checker::new(rm.clone(), sm.clone());

    loop {
        print_prompt();
        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Failed to read line.");
        if input.trim() == "exit" {
            println!("bye!");
            break;
        }
        match sql::parse_sql(&input) {
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
