use crate::parser::ast;

pub const TYPE_INT: i32 = 1;
pub const TYPE_STR: i32 = 2;
pub const TYPE_FLOAT: i32 = 3;
pub const TYPE_DATE: i32 = 4;
pub const TYPE_NUMERIC: i32 = 5;

pub fn datatype2int(ty: &Type) -> i32 {
    match ty {
        Type::Int(_) => TYPE_INT,
        Type::Str(_) => TYPE_STR,
        Type::Float(_) => TYPE_FLOAT,
        Type::Date(_) => TYPE_DATE,
        Type::Numeric(_) => TYPE_NUMERIC,
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Data {
    Str(String),
    Int(i64),
    Float(f64),
    Date(u64),
    Numeric(i64),
} 

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Type {
    Str(Option<String>),
    Int(Option<i64>),
    Float(Option<f64>),
    Date(Option<u64>),
    Numeric(Option<i64>),
}

impl Default for Type {
    fn default() -> Self {
        Self::Int(None)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnType {
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


#[derive(Debug)]
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

#[derive(PartialEq, Debug)]
pub struct Record {
    pub record: Vec<ColumnData>,
}
