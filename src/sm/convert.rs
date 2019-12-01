use crate::parser::ast::*;
use crate::rm::record;

use std::collections::{HashMap, HashSet};
use std::string::ToString;

pub const TYPE_INT: i32 = 1;
pub const TYPE_VARCHAR: i32 = 2;
pub const TYPE_FLOAT: i32 = 3;
pub const TYPE_DATE: i32 = 4;

pub const VALUE_INT: i32 = 1;
pub const VALUE_STR: i32 = 2;
pub const VALUE_FLOAT: i32 = 3;
pub const VALUE_DATE: i32 = 4;
pub const VALUE_NULL: i32 = 5;

pub fn type2int(ty: &Type) -> i32 {
    match ty {
        Type::Int(_) => TYPE_INT,
        Type::Varchar(_) => TYPE_VARCHAR,
        Type::Float => TYPE_FLOAT,
        Type::Date => TYPE_DATE,
    }
}

pub fn value2int(value: &Value) -> i32 {
    match value {
        Value::Int(_) => VALUE_INT,
        Value::Str(_) => VALUE_STR,
        Value::Float(_) => VALUE_FLOAT,
        Value::Date(_) => VALUE_DATE,
        Value::Null => VALUE_NULL,
    }
}

pub fn str2date(s: &str) -> u64 {
    1575185880u64
}

pub fn date2str(date: u64) -> String {
    "2019.12.01 15:38:00".to_string()
}

pub fn value2data(v: &Value) -> Option<record::Data> {
    use std::str::FromStr;
    use record::Data;
    match v {
        Value::Int(s) => {
            Some(Data::Int(i64::from_str(s).unwrap()))
        },
        Value::Str(s) => {
            Some(Data::Str(s.clone()))
        },
        Value::Float(s) => {
            Some(Data::Float(f64::from_str(s).unwrap()))
        },
        Value::Date(s) => {
            Some(Data::Date(str2date(s)))
        },
        Value::Null => {
            None
        },
    }
}

pub fn datatype_from_type(ty: &Type, dv: &Option<Value>) -> record::Type {
    use std::str::FromStr;

    match ty {
        Type::Int(_) => {
            if dv.is_some() {
                if let Value::Int(s) = dv.as_ref().unwrap() {
                    record::Type::Int(Some(i64::from_str(s).unwrap()))
                } else {
                    unreachable!();
                }
            } else {
                record::Type::Int(None)
            }
        },
        Type::Varchar(_) => {
            if dv.is_some() {
                if let Value::Str(s) = dv.as_ref().unwrap() {
                    record::Type::Str(Some(s.to_string()))
                } else {
                    unreachable!();
                }
            } else {
                record::Type::Str(None)
            }
        },
        Type::Date => {
            if dv.is_some() {
                if let Value::Date(s) = dv.as_ref().unwrap() {
                    record::Type::Date(Some(str2date(s)))
                } else {
                    unreachable!();
                }
            } else {
                record::Type::Date(None)
            }
        },
        Type::Float => {
            if dv.is_some() {
                if let Value::Float(s) = dv.as_ref().unwrap() {
                    record::Type::Float(Some(f64::from_str(s).unwrap()))
                } else {
                    unreachable!();
                }
            } else {
                record::Type::Float(None)
            }
        },
    }
}

pub fn primary_column_types_from_column_types(cts: &Vec<record::ColumnType>) -> Vec<&record::ColumnType> {
    cts.iter().filter_map(
        |x| if x.is_primary {
            Some(x)
        } else {
            None
        }
    ).collect()
}

pub fn primary_set_from_column_types(cts: &Vec<record::ColumnType>) -> HashSet<String> {
    cts.iter().filter_map(
        |x| if x.is_primary {
            Some(x.name.clone())
        } else {
            None
        }
    ).collect()
}

pub fn primary_set_from_column_types_hashmap(ct_map: &HashMap<String, record::ColumnType>) -> HashSet<String> {
    ct_map.iter().filter_map(
        |(name, ct)| if ct.is_primary {
            Some(name.clone())
        } else {
            None
        }
    ).collect()
}

pub fn primary_index_from_column_types(cts: &Vec<record::ColumnType>) -> Vec<u32> {
    cts.iter().filter_map(
        |c| if c.is_primary {
            Some(c.index)
        } else {
            None
        }
    ).collect()
}

pub fn column_types_from_fields(field_list: &Vec<Field>) -> Vec<record::ColumnType> {
    let mut primary_key: &Vec<String> = &vec![];
    let mut foreign_key = Vec::new();

    let mut res: Vec<record::ColumnType> = Vec::new();
    let mut map: HashMap<String, usize> = HashMap::new();

    let mut col_index = 0;
    for field in field_list {
        match field {
            Field::ColumnField {col_name, ty, not_null, default_value} => {
                let c = record::ColumnType {
                    name: col_name.clone(),
                    index: col_index,
                    data_type: datatype_from_type(ty, default_value),
                    numeric_precision: 0,
                    can_be_null: !not_null,
                    has_index: false,
                    has_default: default_value.is_some(),
                    is_primary: false,
                    is_foreign: false,
                    default_null: default_value.is_some() && value2int(default_value.as_ref().unwrap()) == VALUE_NULL,
                    foreign_table_name: "".to_string(),
                    foreign_table_column: "".to_string(),
                };
                map.insert(col_name.clone(), col_index as usize);
                res.push(c);

                col_index += 1;
            },
            Field::PrimaryKeyField {column_list} => {
                primary_key = column_list;
            },
            Field::ForeignKeyField {col_name, foreign_tb_name, foreign_col_name } => {
                foreign_key.push((col_name, foreign_tb_name, foreign_col_name));
            }
        }
    }

    for primary in primary_key {
        let i = map.get(primary).unwrap();
        res[*i].is_primary = true;
    }

    for fk in &foreign_key {
        let i = map.get(fk.0).unwrap() ;
        res[*i].is_foreign = true;
        res[*i].foreign_table_name = fk.1.clone();
        res[*i].foreign_table_column = fk.2.clone();
    }

    res
}


// Field | Type    | Null | Key | Default | Extra
pub fn print_from_column_types(cts: &Vec<record::ColumnType>) -> Vec<String> {
    let mut res = vec![];
    for i in 0..5 {
        res.push(vec![])
    }
    let mut non_primary_col_number = 0;

    fn print_from_column_type(ct: &record::ColumnType, is_mul: bool) -> Vec<String> {
        use record::Type;
        let mut content = Vec::new();
        content.push(ct.name.clone());
        content.push(
            match ct.data_type {
                Type::Str(_) => "VARCHAR",
                Type::Int(_) => "INT",
                Type::Float(_) => "FLOAT",
                Type::Date(_) => "DATE",
                Type::Numeric(_) => "NUMERIC",
            }.to_string()
        );

        content.push(
            if ct.can_be_null {
                "YES"
            } else {
                "NO"
            }.to_string()
        );

        content.push(
            if ct.is_primary {
                "PRI"
            } else if is_mul {
                "MUL"
            } else {
                ""
            }.to_string()
        );

        content.push(
            match &ct.data_type {
                Type::Str(s) => {
                    if let Some(s) = s{
                        s.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                Type::Int(s) => {
                    if let Some(s) = s {
                        s.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                Type::Float(s) => {
                    if let Some(s) = s {
                        s.to_string()
                    } else {
                        "NULL".to_string()
                    }
                },
                Type::Date(s) => {
                    if let Some(s) = s {
                        s.to_string()
                    } else {
                        "NULL".to_string()
                    }
                },
                Type::Numeric(s) => {
                    if let Some(s) = s {
                        s.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
            }
        );
        content
    }

    for ct in cts {
        let content = print_from_column_type(ct, non_primary_col_number == 0);
        for i in 0..5 {
            res[i].push(content[i].clone());
        }
        if !ct.is_primary {
            non_primary_col_number += 1;
        }
    }

    res.iter().map(|x| x.join("\n")).collect()
}