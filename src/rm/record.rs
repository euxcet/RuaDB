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