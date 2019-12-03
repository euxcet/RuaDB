use crate::parser::ast;
use crate::rm::record::*;

pub struct Column {
    pub tb_name: Option<Name>,
    pub col_name: Name,
}


trait QueryNode {
    fn query(&self) -> Vec<Record>;
}

struct SelectNode {
    pub table_list: Vec<ast::Name>,
    pub condition: Option<Vec<ast::WhereClause>>,
}

impl QueryNode for SelectNode {

}

struct ProjectNode {

}

impl QueryNode for ProjectNode {

}

struct ProductNode {

}


impl QueryNode for ProductNode {

}

pub struct QueryTree {
    pub root: QueryNode,
}

impl QueryTree {
    pub fn new(table_list: &Vec<ast::Name>, selector: &ast::Selector, where_clause: &Option<Vec<ast::WhereClause>>) -> Self {

        Self {
            root: 
        }
    }

    pub fn query() -> Vec<Record> {
        unimplemented!()
    }
}

