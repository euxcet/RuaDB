use crate::parser::ast;
use crate::rm::record::*;
use crate::rm::record_manager::*;
use crate::rm::pagedef::*;
use crate::index::btree::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

trait QueryNode {
    fn get_name(&self) -> &str;
    fn query(&self) -> RecordList;
}

#[derive(Debug, Clone)]
struct Range {
    pub min: Option<Data>,
    pub min_equal: bool,
    pub max: Option<Data>,
    pub max_equal: bool,
    pub not_equal: Vec<Data>,
}

impl Range {
    pub fn new() -> Self {
        Self {
            min: None,
            min_equal: false,
            max: None,
            max_equal: false,
            not_equal: Vec::new(),
        }
    }

    pub fn is_single(&self) -> bool {
        self.min.is_some() && self.max.is_some() && self.min == self.max && self.min_equal && self.max_equal
    }

    pub fn contains(&self, data: &Data) -> bool {
        (self.min.is_none() || ( // min
            if self.min_equal {
                data >= self.min.as_ref().unwrap()
            }
            else {
                data > self.min.as_ref().unwrap()
            }
        ))
        &&
        (self.max.is_none() || ( // max
            if self.max_equal {
                data <= self.max.as_ref().unwrap()
            }
            else {
                data < self.max.as_ref().unwrap()
            }
        ))
        &&
        self.not_equal.iter().map(|d| data != d).fold(true, |s, v| s & v)
    }

    pub fn intersection(&self, other: &Range) -> Range {
        let mut min = None;
        let mut max = None;
        let mut min_equal = false;
        let mut max_equal = false;
        if self.min.is_none() {
            min = other.min.clone();
            min_equal = other.min_equal;
        }
        else if other.min.is_none() {
            min = self.min.clone();
            min_equal = self.min_equal;
        }
        else {
            if self.min.as_ref().unwrap() > other.min.as_ref().unwrap() {
                min = self.min.clone();
                min_equal = self.min_equal;
            }
            else if self.min.as_ref().unwrap() < other.min.as_ref().unwrap() {
                min = other.min.clone();
                min_equal = other.min_equal;
            }
            else {
                min = self.min.clone();
                min_equal = self.min_equal & other.min_equal;
            }
        }
        if self.max.is_none() {
            max = other.max.clone();
            max_equal = other.max_equal;
        }
        else if other.max.is_none() {
            max = self.max.clone();
            max_equal = self.max_equal;
        }
        else {
            if self.max.as_ref().unwrap() < other.max.as_ref().unwrap() {
                max = self.max.clone();
                max_equal = self.max_equal;
            }
            else if self.max.as_ref().unwrap() > other.max.as_ref().unwrap() {
                max = other.max.clone();
                max_equal = other.max_equal;
            }
            else {
                max = self.max.clone();
                max_equal = self.max_equal & other.max_equal;
            }
        }
        let mut range = Range {
            min: min,
            min_equal: min_equal,
            max: max,
            max_equal: max_equal,
            not_equal: Vec::new(),
        };
        for data in &self.not_equal {
            if range.contains(data) {
                range.not_equal.push(data.clone());
            }
        }
        for data in &other.not_equal {
            if range.contains(data) {
                range.not_equal.push(data.clone());
            }
        }
        range
    }
}

#[derive(Debug, Clone)]
struct RangeCondition {
    pub col: ast::Column,
    pub range: Range,
}

impl RangeCondition {
    pub fn contains(&self, data: &Data) -> bool {
        self.range.contains(data)
    }

    pub fn is_single(&self) -> bool {
        self.range.is_single()
    }

    pub fn update(&mut self, op: &ast::Op, data: Option<Data>) {
        assert!(data.is_some());
        let data = data.unwrap();
        match op {
            ast::Op::Equal => {
                self.range = self.range.intersection(&Range {
                    min: Some(data.clone()),
                    min_equal: true,
                    max: Some(data.clone()),
                    max_equal: true,
                    not_equal: Vec::new(),
                });
            }
            ast::Op::NotEqual => {
                if self.contains(&data) {
                    self.range.not_equal.push(data.clone());
                }
            }
            ast::Op::LessEqual => {
                self.range = self.range.intersection(&Range {
                    min: None,
                    min_equal: false,
                    max: Some(data.clone()),
                    max_equal: true,
                    not_equal: Vec::new(),
                });
            }
            ast::Op::GreaterEqual => {
                self.range = self.range.intersection(&Range {
                    min: Some(data.clone()),
                    min_equal: true,
                    max: None,
                    max_equal: false,
                    not_equal: Vec::new(),
                });
            }
            ast::Op::Less => {
                self.range = self.range.intersection(&Range {
                    min: None,
                    min_equal: false,
                    max: Some(data.clone()),
                    max_equal: false,
                    not_equal: Vec::new(),
                });
            }
            ast::Op::Greater => {
                self.range = self.range.intersection(&Range {
                    min: Some(data.clone()),
                    min_equal: false,
                    max: None,
                    max_equal: false,
                    not_equal: Vec::new(),
                });
            }
        }
    }

    fn match_(&self, record: &Record, ty: &Vec<ColumnType>) -> bool {
        let data = record.get_match_data(&self.col, ty).0;
        match data {
            Some(ref data) => self.contains(data),
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
struct PairCondition {
    pub l_col: ast::Column,
    pub r_col: ast::Column,
    pub op: ast::Op,
}

impl PairCondition {
    fn match_(&self, record: &Record, ty: &Vec<ColumnType>) -> bool {
        println!("{:?} {:?}", self.l_col, self.r_col);
        let l_data = record.get_match_data(&self.l_col, ty).0;
        let r_data = record.get_match_data(&self.r_col, ty).0;
        if l_data.is_none() || r_data.is_none() {
            false
        }
        else {
            let l_data = l_data.unwrap();
            let r_data = r_data.unwrap();
            println!("{:?} {:?}", l_data, r_data);
            match &self.op {
                ast::Op::Equal => l_data == r_data,
                ast::Op::NotEqual => l_data != r_data,
                ast::Op::LessEqual => l_data <= r_data,
                ast::Op::GreaterEqual => l_data >= r_data,
                ast::Op::Less => l_data < r_data,
                ast::Op::Greater => l_data > r_data,
            }
        }
    }
}

#[derive(Debug, Clone)]
struct NullCondition {
    pub col: ast::Column,
    pub is_null: bool,
}

impl NullCondition {
    fn match_(&self, record: &Record, ty: &Vec<ColumnType>) -> bool {
        false
    }
}

struct SelectNode {
    pub root_dir: String,
    pub database: String,
    pub rm: Rc<RefCell<RecordManager>>,
    pub son: Option<Box<dyn QueryNode>>,
    pub table_list: Option<ast::Name>,
    pub range_conds: Vec<RangeCondition>,
    pub pair_conds: Vec<PairCondition>,
    pub null_conds: Vec<NullCondition>,
    /*
    pub table_list: Vec<ast::Name>,
    pub condition: Option<Vec<ast::WhereClause>>,
    */
}

impl SelectNode {
    fn is_valid(&self, record: &Record, ty: &Vec<ColumnType>) -> bool {
        let mut valid = true;
        for cond in &self.range_conds {
            valid &= cond.match_(record, ty);
        }
        for cond in &self.pair_conds {
            valid &= cond.match_(record, ty);
        }
        for cond in &self.null_conds {
            valid &= cond.match_(record, ty);
        }
        valid
    }

    fn used_index_count(&self, index_col: &Vec<u32>, ty: &Vec<ColumnType>) -> usize {
        let mut count = 0;
        for index in index_col {
            let mut can_continue = false;
            for cond in &self.range_conds {
                if cond.col.col_name == ty[*index as usize].name {
                    count += 1;
                    can_continue = cond.is_single();
                    break;
                }
            }
            if !can_continue {
                break;
            }
        }
        count 
    }

    fn get_index(&self, index_col: &Vec<u32>, ty: &Vec<ColumnType>) -> (RawIndex, bool, bool) {
        let mut direction = false; // false -> left,  true -> right
        let mut can_be_equal = false;
        let mut raw_index = RawIndex {
            index: Vec::new(),
        };
        for index in index_col {
            let mut can_continue = false;
            for cond in &self.range_conds {
                if cond.col.col_name == ty[*index as usize].name {
                    if cond.range.min.is_some() {
                        raw_index.index.push(cond.range.min.clone().unwrap());
                        direction = true;
                        can_be_equal = cond.range.min_equal;
                    }
                    else {
                        raw_index.index.push(cond.range.max.clone().unwrap());
                        direction = false;
                        can_be_equal = cond.range.max_equal;
                    }
                    can_continue = cond.is_single();
                    break;
                }
            }
            if !can_continue {
                break;
            }
        }
        (raw_index, direction, can_be_equal)
   }
}

impl QueryNode for SelectNode {
    fn get_name(&self) -> &str {
        "select_node"
    }

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
                let mut path: PathBuf = [self.root_dir.clone(), self.database.clone(), self.table_list.clone().unwrap()].iter().collect();
                path.set_extension("rua");
                let th = self.rm.borrow_mut().open_table(path.to_str().unwrap(), false);

                let mut record_list = RecordList {
                    ty: th.get_column_types().cols,
                    record: Vec::new(),
                    ptrs: Vec::new(),
                };

                let btrees = th.get_btrees();
                let mut best_btree: Option<&BTree> = None;
                let mut max_used = 0;
                for btree in &btrees {
                    let used = self.used_index_count(&btree.index_col, &record_list.ty);
                    if used > max_used {
                        best_btree = Some(btree);
                        max_used = used;
                    }
                }
                if best_btree.is_none() {
                    let btree = th.get_born_btree();
                    let mut bucket = btree.first_bucket();
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
                }
                else {
                    let btree = best_btree.unwrap();
                    let (raw_index, direction, can_be_equal) = self.get_index(&btree.index_col, &record_list.ty);
                    let mut bucket = btree.search_record_with_op(&raw_index, direction, can_be_equal);
                    while bucket.is_some() {
                        let bucket_ = bucket.unwrap();
                        for data in &bucket_.data {
                            let record = th.get_record_(*data).0;
                            if self.is_valid(&record, &record_list.ty) {
                                record_list.record.push(record);
                                record_list.ptrs.push(StrPointer::new(*data));
                            }
                        }
                        if direction { // right
                            bucket = if bucket_.next == 0 {None} else {Some(th.get_bucket_(bucket_.next))};
                        }
                        else { // left
                            bucket = if bucket_.prev == 0 {None} else {Some(th.get_bucket_(bucket_.prev))};
                        }
                    }
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
    fn get_name(&self) -> &str {
        "project_node"
    }

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
    fn get_name(&self) -> &str {
        "product_name"
    }

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
    
    pub fn get_column_type(&self, name_cols: &Vec<(String, HashMap<String, ColumnType>)>, cond: &ast::Column) -> ColumnType {
        assert!(cond.tb_name.is_some());
        let cond_tb_name = cond.tb_name.as_ref().unwrap();
        for pair in name_cols {
            let tb_name = &pair.0;
            let hashmap = &pair.1;
            if tb_name == cond_tb_name {
                return hashmap.get(&cond.col_name).unwrap().clone()
            }
        }
        unreachable!()
    }

    pub fn get_table_name(&self, name_cols: &Vec<(String, HashMap<String, ColumnType>)>, cond: &ast::Column) -> String {
        if cond.tb_name.is_some() {
            return cond.tb_name.as_ref().unwrap().clone();
        }
        for pair in name_cols {
            let tb_name = &pair.0;
            let hashmap = &pair.1;
            if hashmap.get(&cond.col_name).is_some() {
                return tb_name.clone();
            }
        }
        unreachable!()
    }

    fn create_conds(&self, table_list: &Vec<ast::Name>, where_clause: &Option<Vec<ast::WhereClause>>, range_conds: &mut Vec<RangeCondition>, null_conds: &mut Vec<NullCondition>, pair_conds: &mut Vec<PairCondition>) {
        let name_cols: Vec<(String, HashMap<String, ColumnType>)> = table_list.iter()
                        .map(|tb_name| {
                            let mut path: PathBuf = [self.root_dir.clone(), self.database.clone(), tb_name.clone()].iter().collect();
                            path.set_extension("rua");
                            let th = self.rm.borrow_mut().open_table(path.to_str().unwrap(), false);
                            let map = th.get_column_types_as_hashmap();
                            th.close();
                            (tb_name.clone(), map)
                        }).collect();
        match where_clause {
            Some(ref conds) => {
                for cond in conds {
                    match cond {
                        ast::WhereClause::IsAssert{col, null} => { // null
                            null_conds.push(NullCondition {
                                col: ast::Column {
                                    tb_name: if col.tb_name.is_none() {Some(self.get_table_name(&name_cols, col))} else {col.tb_name.clone()},
                                    col_name: col.col_name.clone(),
                                },
                                is_null: *null,
                            });
                        },
                        ast::WhereClause::Comparison{col, op, expr} => {
                            match expr {
                                ast::Expr::Value(ref value) => { // range
                                    let mut matched = false;
                                    for cond in range_conds.iter_mut() {
                                        match col.tb_name {
                                            Some(ref tb_name) => {
                                                if col.col_name == cond.col.col_name && cond.col.tb_name.as_ref().unwrap() == tb_name {
                                                    cond.col = col.clone();
                                                    matched = true;
                                                }
                                            }
                                            None => {
                                                if col.col_name == cond.col.col_name {
                                                    matched = true;
                                                }
                                            }
                                        }
                                        if matched {
                                            let data = Data::from_value(value, &self.get_column_type(&name_cols, &cond.col));
                                            cond.update(op, data);
                                            break;
                                        }
                                    }
                                    if !matched {
                                        let col_with_tb_name = match col.tb_name {
                                            Some(ref tb_name) => {
                                                ast::Column {
                                                    tb_name: Some(tb_name.clone()),
                                                    col_name: col.col_name.clone(),
                                                }
                                            }
                                            None => {
                                                ast::Column {
                                                    tb_name: Some(self.get_table_name(&name_cols, &col)),
                                                    col_name: col.col_name.clone(),
                                                }
                                            }
                                        };
                                        let mut cond = RangeCondition {
                                            col: col_with_tb_name,
                                            range: Range::new(),
                                        };
                                        let data = Data::from_value(value, &self.get_column_type(&name_cols, &cond.col));
                                        cond.update(op, data);
                                        range_conds.push(cond);
                                    }
                                }
                                ast::Expr::Column(ref r_col) => { // pair
                                    pair_conds.push(PairCondition {
                                        l_col: ast::Column {
                                            tb_name: if col.tb_name.is_none() {Some(self.get_table_name(&name_cols, col))} else {col.tb_name.clone()},
                                            col_name: col.col_name.clone(),
                                        },
                                        r_col: ast::Column {
                                            tb_name: if r_col.tb_name.is_none() {Some(self.get_table_name(&name_cols, r_col))} else {r_col.tb_name.clone()},
                                            col_name: r_col.col_name.clone(),
                                        },
                                        op: op.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            None => {}
        }
    }

    pub fn build(&mut self, table_list: &Vec<ast::Name>, selector: &ast::Selector, where_clause: &Option<Vec<ast::WhereClause>>) {
        let mut range_conds: Vec<RangeCondition> = Vec::new();
        let mut null_conds: Vec<NullCondition> = Vec::new();
        let mut pair_conds: Vec<PairCondition> = Vec::new();
        self.create_conds(table_list, where_clause, &mut range_conds, &mut null_conds, &mut pair_conds);

        let mut table_range_conds: Vec<Vec<RangeCondition>> = Vec::new();
        let mut table_null_conds: Vec<Vec<NullCondition>> = Vec::new();

        for _ in 0..table_list.len() {
            table_range_conds.push(Vec::new());
            table_null_conds.push(Vec::new());
        }
        for i in 0..table_list.len() {
            for cond in &range_conds {
                if &table_list[i] == cond.col.tb_name.as_ref().unwrap() {
                    table_range_conds[i].push(cond.clone());
                }
            }
            for cond in &null_conds {
                if &table_list[i] == cond.col.tb_name.as_ref().unwrap() {
                    table_null_conds[i].push(cond.clone());
                }
            }
        }
        self.root = Some(self.project_layer(table_list, selector, table_range_conds, table_null_conds, pair_conds))
    }

    fn project_layer(&self, table_list: &Vec<ast::Name>, selector: &ast::Selector, table_range_conds: Vec<Vec<RangeCondition>>, table_null_conds: Vec<Vec<NullCondition>>, pair_conds: Vec<PairCondition>) -> Box<dyn QueryNode> {
        match selector {
            ast::Selector::All => {
                self.select_pair_layer(table_list, table_range_conds, table_null_conds, pair_conds)
            },
            ast::Selector::Columns(cols) => {
                Box::new(ProjectNode {
                    son: self.select_pair_layer(table_list, table_range_conds, table_null_conds, pair_conds),
                    cols: cols.clone(),
                })
            },
        }
    }

    fn select_pair_layer(&self, table_list: &Vec<ast::Name>, table_range_conds: Vec<Vec<RangeCondition>>, table_null_conds: Vec<Vec<NullCondition>>, pair_conds: Vec<PairCondition>) -> Box<dyn QueryNode> {
        Box::new(
            SelectNode {
                root_dir: self.root_dir.clone(),
                database: self.database.clone(),
                rm: self.rm.clone(),
                son: Some(self.product_layer(table_list, table_range_conds, table_null_conds)),
                table_list: None,
                range_conds: Vec::new(),
                null_conds: Vec::new(),
                pair_conds: pair_conds,
            }
        )
    }

    fn product_layer(&self, table_list: &Vec<ast::Name>, table_range_conds: Vec<Vec<RangeCondition>>, table_null_conds: Vec<Vec<NullCondition>>) -> Box<dyn QueryNode> {
        let mut son: Vec<Box<dyn QueryNode>> = Vec::new();
        let mut table_range_conds = table_range_conds;
        let mut table_null_conds = table_null_conds;
        for i in 0..table_list.len() {
            son.push(self.select_single_layer(table_list[i].clone(), table_range_conds[i].clone(), table_null_conds[i].clone()));
        }
        Box::new(
            ProductNode {
                son: son,
            }
        )
    }

    fn select_single_layer(&self, tb_name: ast::Name, range_conds: Vec<RangeCondition>, null_conds: Vec<NullCondition>) -> Box<dyn QueryNode> {
        Box::new(
            SelectNode {
                root_dir: self.root_dir.clone(),
                database: self.database.clone(),
                rm: self.rm.clone(),
                son: None,
                table_list: Some(tb_name),
                range_conds: range_conds,
                null_conds: null_conds,
                pair_conds: Vec::new(),
            }
        )
    }

    pub fn query(&self) -> RecordList {
        match self.root {
            Some(ref root) => root.query(),
            None => RecordList::new(),
        }
    }
}