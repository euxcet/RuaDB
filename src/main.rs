#![allow(unreachable_patterns)]

// #[macro_use]
extern crate lalrpop_util;

pub mod parser;
pub mod ast;
pub mod tok;

pub type ParseError<'input> = lalrpop_util::ParseError<usize, tok::Tok<'input>, tok::Error>;


pub fn parse_sql<'input>(input: &'input str) -> Result<Vec<Option<ast::Cmd>>, ParseError<'input>> {
    let tokenizer = tok::Tokenizer::new(input, 0);
    let sql = parser::lrsql::CmdListParser::new().parse(input, tokenizer)?;
    Ok(sql)
}

fn main() {
    let res = parse_sql("CREATE TABLE test (col)").unwrap();
    for i in &res {
        match i {
            Some(cmd) => { // cmd ast::Cmd
                match cmd {
                    ast::Cmd::Stmt(x) => { // ast::Stmt
                        match x {
                            ast::Stmt::CreateTable{ tbl_name, .. } => {
                                println!("{}", tbl_name.name);
                            },
                            _ => {},
                        }
                    },
                    ast::Cmd::Explain(x) => println!("this is a explain"),
                    ast::Cmd::ExplainQueryPlan(x) => println!("this is a explain query plan"),
                }
            },
            None => println!("None"),
        }
    }
}
