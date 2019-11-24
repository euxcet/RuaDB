extern crate lalrpop_util;
extern crate config;
#[macro_use]
pub mod bytevec;
#[macro_use]
extern crate serde_derive;

// pub mod parser;
// pub mod ast;
// pub mod tok;
// pub mod sql;
mod utils;
mod index;
mod rm;
mod settings;
mod parser;

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
}
