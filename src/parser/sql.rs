use super::ast;
use super::tok;
use super::lrsql;

use crate::utils;

pub type ParseError<'input> = lalrpop_util::ParseError<usize, tok::Tok<'input>, tok::Error>;

pub fn parse_sql<'input>(input: &'input str) -> Result<ast::Sql, ParseError<'input>> {
    let tokenizer = tok::Tokenizer::new(input, 0);
    let sql = lrsql::SqlParser::new().parse(input, tokenizer)?;
    Ok(sql)
}

// pub fn parse(input: &str) -> Option<ast::Sql> {
//     let res = parse_sql(input);
//     if let Ok(sql) = res {
//         Some(sql)
//     } else {
//         println!("Invalid syntax");
//         None
//     }
// }


#[cfg(test)]
mod tests {
    #[test]
    fn parser_test() {
        use crate::parser::sql::parse_sql;
        let correct = vec![
            "show databases;",
            "create database test1;",
            "create database test1;show databases;",
            "drop database test1;",
            "use test1;",
            "show tables;",
            "create table tb (c1 date, c2 int(1) not null default 1, c3 varchar(1) default \"default\", c4 float not null, primary key (c1, c2), foreign key (c5) references tb2 (c1));",
            "drop table tb;",
            "desc tb;",
            "insert into tb values (\"1\", 1), (\"2\", 2);",
            "delete from tb where id = 1;",
            "delete from tb;",
            "delete from tb where id = number;",
            "delete from tb where id = number and id = 1;",
            "update tb set id = 1;",
            "update tb set id = 1 where id = number;",
            "select * from tb;",
            "select tb.c1 from tb, tb2 where tb1.id = \"1\";",
            "create index idx on tb (c1, c2);",
            "drop index idx on tb;",
            "alter table tb add index idx (c1, c2);",
            "alter table tb drop index idx;",
            "alter table tb add c1 date not null default 1;",
            "alter table tb drop c1;",
            "alter table tb2 change c1 c1 date default 1;",
            "alter table tb2 rename to tb4;",
            "alter table pk add primary key (c1, c2);",
            "alter table pk drop primary key;",
            "alter table tb add constraint pk primary key (c1, c2);",
            "alter table tb drop primary key pk;",
            "alter table tb add constraint fk foreign key (c1, c2) references ftb (c4, c5);",
            "alter table tb drop foreign key fk;",
        ];
        let incorrect = vec![
            "1;",
        ];

        for s in &correct {
            assert!(parse_sql(s).is_ok());
        }

        for s in &incorrect {
            assert!(parse_sql(s).is_err());
        }
    }
}

// pub fn parse(sql: &String) {
//     let root = "E:\\database\\";
//     let res = parse_sql(sql.as_str()).unwrap();

//     for i in &res {
//         match i {
//             Some(cmd) => { // cmd ast::Cmd
//                 match cmd {
//                     ast::Cmd::Stmt(x) => { // ast::Stmt
//                         match x {
//                             ast::Stmt::CreateDatabase{ db_name } => {
//                                 utils::file::create_table(root.to_owned() + db_name);
//                                 // println!("{}", db_name);
//                             },
//                             ast::Stmt::DropDatabase{ db_name } => {
//                                 utils::file::drop_table(root.to_owned() + db_name);
//                             }
//                             ast::Stmt::ShowDatabase{ db_name } => {
//                                 let dbs = utils::file::get_database_list(root);
//                                 for db in dbs {
//                                     println!("{}", db);
//                                     let tables = utils::file::get_table_list(&(root.to_owned() + &db + "\\"));
//                                     for table in tables {
//                                         println!(" - {}", table);
//                                     }
//                                 }
//                             }
//                             _ => {},
//                         }
//                     },
//                     ast::Cmd::Explain(_x) => println!("this is a explain"),
//                     ast::Cmd::ExplainQueryPlan(_x) => println!("this is a explain query plan"),
//                 }
//             },
//             None => println!("None"),
//         }
//     }
// }
