use crate::parser::ast;
use crate::rm::record::*;

trait QueryNode {
    fn query(&self) -> Vec<Record>;
}

struct SelectNode {
    pub son: Option<Box<dyn QueryNode>>,
    pub table_list: Vec<ast::Name>,
    pub condition: Option<Vec<ast::WhereClause>>,
}

impl QueryNode for SelectNode {
    fn query(&self) -> Vec<Record> {
        unimplemented!()
    }
}

struct ProjectNode {

}

impl QueryNode for ProjectNode {
    fn query(&self) -> Vec<Record> {
        unimplemented!()
    }
}

struct ProductNode {

}

impl QueryNode for ProductNode {
    fn query(&self) -> Vec<Record> {
        unimplemented!()
    }
}

pub struct QueryTree {
    pub root: Box<dyn QueryNode>,
}

impl QueryTree {
    pub fn new(table_list: &Vec<ast::Name>, selector: &ast::Selector, where_clause: &Option<Vec<ast::WhereClause>>) -> Self {
        unimplemented!()
    }

    pub fn query(&self) -> Vec<Record> {
        unimplemented!()
    }
}

