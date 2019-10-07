#![allow(unreachable_patterns)]

// #[macro_use]
extern crate lalrpop_util;

pub mod parser;
pub mod ast;
pub mod tok;

pub mod sql;


fn main() {
    sql::parse();
}
