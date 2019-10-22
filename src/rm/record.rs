pub enum Data {
    Str(Option<String>),
    Int(Option<i64>),
    Float(Option<f64>),
    Date(Option<u64>),
} 

pub const MAX_STRING_LENGTH: usize = 1024;

pub enum Type {
    Str(bool, String, u32),
    Int(bool, i64),
    Float(bool, f64),
    Date(bool, u64),
}

pub struct ColumnType {
    name: String,
    t: Type,
}

pub struct ColumnData {
    name: String,
    data: Data,
}

pub struct Record {
    record: Vec<ColumnData>,
}
