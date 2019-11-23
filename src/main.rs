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
pub mod utils;
pub mod index;
pub mod rm;
mod settings;

use settings::Settings;

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

    let settings = Settings::new();
    println!("{:?}", settings);

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
