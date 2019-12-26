extern crate lalrpop_util;
extern crate config;
#[macro_use]
pub mod bytevec;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate prettytable;

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
    
    // let mut checker = executor::checker::Checker::new(rm.clone(), sm.clone());
    let logger = logger::logger::RuaLogger::new();
    loop {
        print_prompt();
    }
}
