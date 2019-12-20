use std::collections::HashMap;
use crate::parser::ast;
use crate::utils::convert;
use super::pagedef::StrPointer;

use crate::defer;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Data {
    Str(String),
    Int(i64),
    Float(f64),
    Date(u64),
    Numeric(i64),
} 

impl Data {
    pub fn from_value(value: &ast::Value, ct: &ColumnType) -> Option<Self> {
        use std::str::FromStr;
        if ct.numeric_precision > 0 {
            match value {
                ast::Value::Int(s) |
                ast::Value::Float(s) => Some(Self::Numeric(convert::str_to_numeric(s, ct.numeric_precision))),
                ast::Value::Null => None,
                _ => unreachable!(),
            }
        }
        else {
            match value {
                ast::Value::Int(s) => Some(Self::Int(i64::from_str(s).unwrap())),
                ast::Value::Str(s) => Some(Self::Str(s.to_string())),
                ast::Value::Date(s) => Some(Self::Date(convert::str_to_date(s))),
                ast::Value::Float(s) => Some(Self::Float(f64::from_str(s).unwrap())),
                ast::Value::Null => None,
            }
        }
    }

    pub fn to_string(&self, ct: &ColumnType) -> String {
        match self {
            Data::Str(d) => d.clone(),
            Data::Int(d) => d.to_string(),
            Data::Float(d) => d.to_string(),
            Data::Date(d) => convert::date_to_str(*d),
            Data::Numeric(d) => {
                convert::numeric_to_str(*d, ct.numeric_precision)
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Type {
    Str(Option<String>),
    Int(Option<i64>),
    Float(Option<f64>),
    Date(Option<u64>),
    Numeric(Option<i64>),
}

impl Type {
    pub fn of_same_type(&self, ty: &ast::Type) -> bool {
        match (self, ty) {
            (Type::Int(_), ast::Type::Int(_)) | 
            (Type::Str(_), ast::Type::Varchar(_)) | 
            (Type::Float(_), ast::Type::Float) | 
            (Type::Date(_), ast::Type::Date) |
            (Type::Numeric(_), ast::Type::Numeric(_, _))=> true,
            (_, _) => false, 
        }
    }
    pub fn comparable(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Int(_), Type::Int(_)) | 
            (Type::Str(_), Type::Str(_)) | 
            (Type::Float(_), Type::Float(_)) | 
            (Type::Date(_), Type::Date(_)) |
            (Type::Numeric(_), Type::Numeric(_)) => true,
            (_, _) => false, 
        }
    }

    pub fn valid_value(&self, value: &ast::Value) -> bool {
        match (self, value) {
            (Type::Int(_), ast::Value::Int(_)) |
            (Type::Str(_), ast::Value::Str(_)) |
            (Type::Float(_), ast::Value::Float(_)) |
            (Type::Date(_), ast::Value::Date(_)) |
            (Type::Numeric(_), ast::Value::Int(_)) |
            (Type::Numeric(_), ast::Value::Float(_)) => true,
            (_, _) => false,
        }
    }

    pub fn from_type(ty: &ast::Type, value: &Option<ast::Value>) -> Self {
        use std::str::FromStr; 
        match ty {
            ast::Type::Int(_) => {
                match value {
                    Some(ast::Value::Int(s)) => Self::Int(Some(i64::from_str(s).unwrap())),
                    None => Self::Int(None),
                    _ => unreachable!(),
                }
            },
            ast::Type::Varchar(_) => {
                match value {
                    Some(ast::Value::Str(s)) => Self::Str(Some(s.to_string())),
                    None => Self::Str(None),
                    _ => unreachable!(),
                }
            },
            ast::Type::Date => {
                match value {
                    Some(ast::Value::Date(s)) => Self::Date(Some(convert::str_to_date(s))),
                    None => Self::Date(None),
                    _ => unreachable!(),
                }
            },
            ast::Type::Float => {
                match value {
                    Some(ast::Value::Float(s)) => Self::Float(Some(f64::from_str(s).unwrap())),
                    None => Self::Float(None),
                    _ => unreachable!(),
                }
            },
            ast::Type::Numeric(_, p) => {
                match value {
                    Some(ast::Value::Int(s)) => Self::Numeric(Some(convert::str_to_numeric(s, *p as u8))),
                    Some(ast::Value::Float(s)) => Self::Numeric(Some(convert::str_to_numeric(s, *p as u8))),
                    None => Self::Numeric(None),
                    _ =>  unreachable!(),
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Str(_) => "VARCHAR",
            Type::Int(_) => "INT",
            Type::Float(_) => "FLOAT",
            Type::Date(_) => "DATE",
            Type::Numeric(_) => "NUMERIC",
        }.to_string()
    }

    pub fn get_default_string(&self) -> String {
        match &self { // Default
            Type::Str(Some(s)) => s.to_string(),
            Type::Str(None) => String::from("NULL"),
            Type::Int(Some(s)) => s.to_string(),
            Type::Int(None) => String::from("NULL"),
            Type::Float(Some(s)) => s.to_string(),
            Type::Float(None) => String::from("NULL"),
            Type::Date(Some(s)) => convert::date_to_str(*s),
            Type::Date(None) => String::from("NULL"),
            Type::Numeric(Some(s)) => s.to_string(),
            Type::Numeric(None) => String::from("NULL"),
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Int(None)
    }
}



#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnType {
    pub tb_name: String,
    pub name: String,
    pub index: u32,
    pub data_type: Type,
    pub numeric_precision: u8,
    pub can_be_null: bool,
    pub has_index: bool,
    pub has_default: bool,
    pub is_primary: bool,
    pub is_foreign: bool,
    pub default_null: bool,
    pub foreign_table_name: String,
    pub foreign_table_column: String,
}

impl ColumnType {
    pub fn match_(&self, col: &ast::Column) -> bool {
        match &col.tb_name {
            Some(tb_name) => tb_name == &self.tb_name && col.col_name == self.name,
            None => col.col_name == self.name,
        }
    }
    pub fn print(&self, is_mul: bool) -> Vec<String> {
        vec![
            self.name.clone(), // Field
            self.data_type.to_string(), // Type
            String::from(if self.can_be_null {"YES"} else {"NO"}), // Null
            String::from(if self.is_primary {"PRI"} else if is_mul {"MUL"} else {""}), // Key
            self.data_type.get_default_string(), // Default
            if self.is_foreign { format!("{}.{}", self.foreign_table_name, self.foreign_table_column)} else {String::from("")},
        ]
    }

    pub fn from_field(tb_name: &String, index: u32, field: &ast::Field) -> Self {
        unimplemented!();
    }
}

pub struct ColumnTypeVec {
    pub cols: Vec<ColumnType>,
}

impl ColumnTypeVec {
    pub fn from_fields<'a, 'b>(field_list: &'a Vec<ast::Field>, tb_name: &'b String) -> (Self, Vec<u32>, Vec<(&'a String, Vec<u32>)>) {
        let mut primary_key: &Vec<String> = &Vec::new();
        let mut foreign_list = Vec::new();

        let mut cols: Vec<ColumnType> = Vec::new();
        let mut map: HashMap<String, usize> = HashMap::new();

        for index in 0..field_list.len() {
            match &field_list[index] {
                ast::Field::ColumnField {col_name, ty, not_null, default_value} => {
                    map.insert(col_name.clone(), index);
                    cols.push(ColumnType {
                        tb_name: tb_name.clone(),
                        name: col_name.clone(),
                        index: index as u32,
                        data_type: Type::from_type(&ty, &default_value),
                        numeric_precision: match &ty {
                            ast::Type::Numeric(_, p) => *p as u8 + 1,
                            _ => 0,
                        },
                        can_be_null: !not_null,
                        has_index: false,
                        has_default: default_value.is_some(),
                        is_primary: false,
                        is_foreign: false,
                        default_null: default_value.is_some() && default_value.as_ref().unwrap().is_null(),
                        foreign_table_name: String::new(),
                        foreign_table_column: String::new(),
                    });
                },
                ast::Field::PrimaryKeyField {column_list} => {
                    primary_key = column_list;
                },
                ast::Field::ForeignKeyField { column_list, foreign_tb_name, foreign_column_list } => {
                    foreign_list.push((column_list, foreign_tb_name, foreign_column_list))
                }
            }
        }
        
        let mut primary_cols = Vec::new();

        for primary in primary_key {
            let index = map.get(primary).unwrap();
            cols[*index].is_primary = true;
            cols[*index].can_be_null = false;
        }

        let mut foreign_col_index  = Vec::new();

        for (cs, ft_name, fcs) in foreign_list {
            for (c_name, fc_name) in cs.iter().zip(fcs.iter()) {
                let index = map.get(c_name).unwrap();
                cols[*index].is_foreign = true;
                cols[*index].foreign_table_name = ft_name.clone();
                cols[*index].foreign_table_column = fc_name.clone();
            }

            foreign_col_index.push((ft_name, cs.iter().map(|c_name| *map.get(c_name).unwrap() as u32).collect()));
        }

        // for fk in &foreign_key {
        //     let index = map.get(fk.0).unwrap() ;
        //     cols[*index].is_foreign = true;
        //     cols[*index].foreign_table_name = fk.1.clone();
        //     cols[*index].foreign_table_column = fk.2.clone();
        // }

        (Self { cols: cols, }, primary_cols, foreign_col_index)
    }

    pub fn get_primary_index(&self) -> Vec<u32> {
        self.cols.iter().filter_map(
            |c| if c.is_primary {Some(c.index)} else {None}
        ).collect()
    }

    pub fn print(&self) -> Vec<String> {
        let mut res = vec![vec![]; 6];
        let mut non_primary_col_number = 0;
        for col in &self.cols {
            let content = col.print(non_primary_col_number == 0);
            for i in 0..6 {
                res[i].push(content[i].clone());
            }
            if !col.is_primary {
                non_primary_col_number += 1;
            }
        }
        res.iter().map(|x| x.join("\n")).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ColumnData {
    pub index: u32,
    pub default: bool,
    pub data: Option<Data>,
}

impl PartialEq for ColumnData {
    fn eq (&self, other: &ColumnData) -> bool {
        self.index == other.index && self.data == other.data
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Record {
    pub cols: Vec<ColumnData>,
}

impl Record {
    pub fn print(&self, ct: &Vec<ColumnType>) -> Vec<String> {
        self.cols.iter().zip(ct.iter()).map(|(x, ct)| {
            match &x.data {
                Some(d) => d.to_string(ct),
                None => String::new(),
            }
        }).collect()
    }

    pub fn from_value_lists(value_lists: &Vec<ast::Value>, cts: &Vec<ColumnType>) -> Self {
        let mut cols = Vec::new();
        for i in 0..value_lists.len() {
            cols.push(ColumnData {
                index: i as u32,
                default: false,
                data: Data::from_value(&value_lists[i], &cts[i])
            });
        }
        Self {
            cols: cols,
        }
    }

    pub fn sub_record(&self, sub_cols: &Vec<usize>) -> Record {
        let mut cols = Vec::new();
        for pos in sub_cols {
            cols.push(self.cols[*pos].clone());
        }
        Record {
            cols: cols,
        }
    }

    pub fn get_match_data(&self, col: &ast::Column, ty: &Vec<ColumnType>) -> (Option<Data>, Option<ColumnType>) {
        for i in 0..ty.len() {
            if ty[i].match_(col) {
                return (self.cols[i].data.clone(), Some(ty[i].clone()));
            }
        }
        (None, None)
    }

    pub fn set_(&mut self, value_list: &Vec<ast::SetClause>, ty: &Vec<ColumnType>) {
        for value in value_list {
            for i in 0..ty.len() {
                if ty[i].name == value.col_name {
                    self.cols[i].data = Data::from_value(&value.value, &ty[i]);
                }
            }
        }
    }

    pub fn match_(&self, condition: &ast::WhereClause, ty: &Vec<ColumnType>) -> bool {
        match condition {
            ast::WhereClause::IsAssert{col, null} => {
                for i in 0..ty.len() {
                    if ty[i].match_(col) {
                        return (self.cols[i].data.is_none() && *null) || (self.cols[i].data.is_some() && !*null);
                    }
                }
                true
            },
            ast::WhereClause::Comparison{col, op, expr} => {
                let l_data = self.get_match_data(col, ty);
                let r_data = match expr {
                    ast::Expr::Value(ref value) => {
                        Data::from_value(value, &l_data.1.unwrap())
                    }
                    ast::Expr::Column(ref r_col) => {
                        self.get_match_data(r_col, ty).0
                    }
                };
                let l_data = l_data.0;
                if l_data.is_none() || r_data.is_none() {
                    false
                }
                else {
                    let l_data = l_data.unwrap();
                    let r_data = r_data.unwrap();
                    match op {
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
    }
}

pub struct RecordList {
    pub ty: Vec<ColumnType>,
    // TODO: delete record
    pub record: Vec<Record>,
    pub ptrs: Vec<StrPointer>,
}

impl RecordList {
    pub fn sub_record_list(&self, sub_cols: &Vec<usize>) -> RecordList {
        let mut ty = Vec::new();
        for pos in sub_cols {
            ty.push(self.ty[*pos].clone());
        }
        RecordList {
            ty: ty,
            record: self.record.iter().map(|record| record.sub_record(sub_cols)).collect(),
            ptrs: self.ptrs.clone(),
        }
    }

    pub fn print(&self) -> Vec<Vec<String>> {
        let title: Vec<String> = self.ty.iter().map(|ty| ty.name.clone()).collect();
        let mut res = vec![vec![]; title.len()];
        for record in &self.record {
            let content = record.print(&self.ty);
            for i in 0..title.len() {
                res[i].push(content[i].clone());
            }
        }
        vec![title, res.iter().map(|x| x.join("\n")).collect()]
    }
}