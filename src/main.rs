// #[macro_use]
extern crate lalrpop_util;

// pub mod parser;
// pub mod ast;
// pub mod tok;
// pub mod sql;
pub mod utils;
pub mod index;
pub mod rm;

use std::io;
use std::io::prelude::*;




fn initalize() {
    println!("Connected to server.");
}

fn print_prompt() {
    print!("rua > ");
    io::stdout().flush().ok().expect("Could not flush stdout.");
}

fn main() {
    initalize();

    /*
    loop {
        print_prompt();
        let mut sql = String::new();
        io::stdin().read_line(&mut sql).expect("Failed to read line.");
        if sql.trim() == "exit" {
            println!("bye!");
            break;
        }
        sql::parse(&sql);
    }
    */
}
