use crate::parser;
use crate::ast;
use crate::tok;
use crate::utils;

pub type ParseError<'input> = lalrpop_util::ParseError<usize, tok::Tok<'input>, tok::Error>;


pub fn parse_sql<'input>(input: &'input str) -> Result<Vec<Option<ast::Cmd>>, ParseError<'input>> {
    let tokenizer = tok::Tokenizer::new(input, 0);
    let sql = parser::lrsql::CmdListParser::new().parse(input, tokenizer)?;
    Ok(sql)
}

pub fn parse(sql: &String) {
    let root = "E:\\database\\";
    // let dbs = utils::file::get_database_list("E:\\database\\");
    // let tables = utils::file::get_database_list("E:\\database\\db1\\");

    // "CREATE TABLE test (col)"
    let res = parse_sql(&*sql).unwrap();

    for i in &res {
        match i {
            Some(cmd) => { // cmd ast::Cmd
                match cmd {
                    ast::Cmd::Stmt(x) => { // ast::Stmt
                        match x {
                            ast::Stmt::CreateDatabase{ db_name } => {
                                utils::file::create_table(root.to_owned() + db_name);
                                // println!("{}", db_name);
                            },
                            ast::Stmt::DropDatabase{ db_name } => {
                                utils::file::drop_table(root.to_owned() + db_name);
                            }
                            ast::Stmt::ShowDatabase{ db_name } => {
                                let dbs = utils::file::get_database_list(root);
                                for db in dbs {
                                    println!("{}", db);
                                    let tables = utils::file::get_table_list(&(root.to_owned() + &db + "\\"));
                                    for table in tables {
                                        println!(" - {}", table);
                                    }
                                }
                            }
                            _ => {},
                        }
                    },
                    ast::Cmd::Explain(_x) => println!("this is a explain"),
                    ast::Cmd::ExplainQueryPlan(_x) => println!("this is a explain query plan"),
                }
            },
            None => println!("None"),
        }
    }
}
