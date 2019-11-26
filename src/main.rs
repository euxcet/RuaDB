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
    initalize();
    let rua = logger::logger::RuaLogger::new();
    let mut exe = executor::Executor::new();

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
                    let res = exe.exe(stmt);
                    rua.rua(&res);
                }
            },
            Err(e) => {
                println!("Invalid syntax");
            }
        }
    }
}
