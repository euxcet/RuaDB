use crate::parser::ast;
use crate::rm::record::*;
use crate::rm::record_manager::*;
use crate::rm::pagedef::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

trait QueryNode {
    fn query(&self) -> RecordList;
}

struct SelectNode {
    pub root_dir: String,
    pub database: String,
    pub rm: Rc<RefCell<RecordManager>>,
    pub son: Option<Box<dyn QueryNode>>,
    pub table_list: Vec<ast::Name>,
    pub condition: Option<Vec<ast::WhereClause>>,
}

impl SelectNode {
    fn is_valid(&self, record: &Record, ty: &Vec<ColumnType>) -> bool {
        match self.condition {
            Some(ref condition) => {
                let mut valid = true;
                for cond in condition {
                    valid = valid & record.match_(cond, ty);
                }
                valid
            }
            None => true,
        }
    }
}

impl QueryNode for SelectNode {
    fn query(&self) -> RecordList {
        match self.son {
            Some(ref son) => {
                let son_record_list = son.query();
                let mut record_list = RecordList {
                    ty: son_record_list.ty,
                    record: Vec::new(),
                    ptrs: Vec::new(),
                };
                for i in 0..son_record_list.record.len() {
                    if self.is_valid(&son_record_list.record[i], &record_list.ty) {
                        record_list.record.push(son_record_list.record[i].clone());
                        record_list.ptrs.push(son_record_list.ptrs[i]);
                    }
                }
                record_list
            }
            None => {
                assert_eq!(self.table_list.len(), 1);
                let mut path: PathBuf = [self.root_dir.clone(), self.database.clone(), self.table_list[0].clone()].iter().collect();
                path.set_extension("rua");
                let mut th = self.rm.borrow_mut().open_table(path.to_str().unwrap(), false);
                let btree = th.get_born_btree();
                let mut bucket = btree.first_bucket();
                let mut record_list = RecordList {
                    ty: th.get_column_types().cols.clone(),
                    record: Vec::new(),
                    ptrs: Vec::new(),
                };
                while bucket.is_some() {
                    let bucket_ = bucket.unwrap();
                    for data in &bucket_.data {
                        let record = th.get_record_(*data).0;
                        if self.is_valid(&record, &record_list.ty) {
                            record_list.record.push(record);
                            record_list.ptrs.push(StrPointer::new(*data));
                        }
                    }
                    bucket = if bucket_.next == 0 {None} else {Some(th.get_bucket_(bucket_.next))};
                }
                th.close();
                record_list
            }
        }
    }
}

struct ProjectNode {
    pub son: Box<dyn QueryNode>,
    pub cols: Vec<ast::Column>,
}

impl QueryNode for ProjectNode {
    fn query(&self) -> RecordList {
        if self.cols.len() == 0 {
            self.son.query()
        }
        else {
            let record_list = self.son.query();
            let mut sub_cols = Vec::new();
            for i in 0..record_list.ty.len() {
                for col in &self.cols {
                    if record_list.ty[i].match_(col) {
                        sub_cols.push(i);
                    }
                }
            }
            record_list.sub_record_list(&sub_cols)
        }
    }
}

struct ProductNode {
    pub son: Vec<Box<dyn QueryNode>>,
}

impl ProductNode {
    fn concat(records: &Vec<RecordList>, pos: usize, current_record: &mut Record, result: &mut RecordList) {
        if pos == records.len() {
            result.record.push(current_record.clone());
            result.ptrs.push(StrPointer::new(0));
            return;
        }
        let record_list = &records[pos];
        for r in &record_list.record {
            for col in &r.cols {
                current_record.cols.push(col.clone());
            }
            ProductNode::concat(records, pos + 1, current_record, result);
            for _ in &r.cols {
                current_record.cols.pop();
            }
        }
    }
}

impl QueryNode for ProductNode {
    fn query(&self) -> RecordList {
        let mut record_lists: Vec<RecordList> = self.son.iter().map(|node| node.query()).collect();
        if record_lists.len() == 1 {
            record_lists.pop().unwrap()
        }
        else {
            let mut ty = Vec::new();
            for record_list in &record_lists {
                for t in &record_list.ty {
                    ty.push(t.clone());
                }
            }
            let mut result = RecordList {
                ty: ty,
                record: Vec::new(),
                ptrs: Vec::new(),
            };
            ProductNode::concat(&record_lists, 0, &mut Record{ cols: Vec::new(), }, &mut result);
            result
        }
    }
}

pub struct QueryTree {
    root_dir: String,
    database: String,
    rm: Rc<RefCell<RecordManager>>,
    root: Option<Box<dyn QueryNode>>,
}

impl QueryTree {
    pub fn new(root_dir: &String, database_dir: &String, rm: Rc<RefCell<RecordManager>>) -> Self {
        Self {
            root_dir: root_dir.clone(),
            database: database_dir.clone(),
            rm: rm.clone(),
            root: None,
        }
    }

    pub fn build(&mut self, table_list: &Vec<ast::Name>, selector: &ast::Selector, where_clause: &Option<Vec<ast::WhereClause>>) {
        self.root = Some(self.project_layer(table_list, selector, where_clause));
    }

    fn project_layer(&self, table_list: &Vec<ast::Name>, selector: &ast::Selector, where_clause: &Option<Vec<ast::WhereClause>>) -> Box<dyn QueryNode> {
        match selector {
            ast::Selector::All => {
                self.select_where_layer(table_list, where_clause)
            },
            ast::Selector::Columns(cols) => {
                Box::new(ProjectNode {
                    son: self.select_where_layer(table_list, where_clause),
                    cols: cols.clone(),
                })
            },
        }
    }

    fn select_where_layer(&self, table_list: &Vec<ast::Name>, where_clause: &Option<Vec<ast::WhereClause>>) -> Box<dyn QueryNode> {
        Box::new(
            SelectNode {
                root_dir: self.root_dir.clone(),
                database: self.database.clone(),
                rm: self.rm.clone(),
                son: Some(self.product_layer(table_list)),
                table_list: table_list.clone(),
                condition: where_clause.clone(),
            }
        )
    }

    fn product_layer(&self, table_list: &Vec<ast::Name>) -> Box<dyn QueryNode> {
        Box::new(
            ProductNode {
                son: table_list.iter().map(|tb_name| self.select_layer(tb_name.clone())).collect(),
            }
        )
    }

    fn select_layer(&self, tb_name: ast::Name) -> Box<dyn QueryNode> {
        Box::new(
            SelectNode {
                root_dir: self.root_dir.clone(),
                database: self.database.clone(),
                rm: self.rm.clone(),
                son: None,
                table_list: vec![tb_name],
                condition: None,
            }
        )
    }

    pub fn query(&self) -> RecordList {
        match self.root {
            Some(ref root) => root.query(),
            None => panic!(),
        }
    }
}