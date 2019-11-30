use crate::parser::ast::*;
use crate::sm::system_manager::SystemManager;

use std::collections::HashMap;

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
        Type::Float(_) => TYPE_FLOAT,
        Type::Date(_) => TYPE_DATE,
    }
}

pub fn value2int(value: &Value) -> i32 {
    match value {
        Value::Int(_) => VALUE_INT,
        Value::Str(_) => VALUE_STR,
        Value::Float(_) => VALUE_FLOAT,
        Value::Date(_) => VALUE_DATE,
        Value::Null(_) => VALUE_NULL,
    }
}

pub fn valid_type_value(ty: &Type, value: &Value) -> bool {
    let v = value2int(value);
    let t = type2int(ty);
    v == t || v == VALUE_NULL
}

pub valid_field_list(field_list: &Vec<Field>, sm: &SystemManager) -> bool {
    let mut name_field = HashMap::new();
    let mut primary_key: Vec<Vec<Name>> = Vec::new();
    let mut name_foreign_key = HashMap::new();

    for field in field_list {
        match field {
            Field::ColumnField {col_name, ty, not_null, dv} => {
                if name_field.contains_key(col_name) {
                    return false;
                }
                if let Some(ref v) = dv {
                    if !valid_type_value(ty, v) return false;
                }
                name_field.insert(col_name, (col_name, ty, not_null, dv));
            },
            Field::PrimaryKeyField {column_list} => {
                primary_key.push(column_list);
            },
            Field::ForeignKeyField {col_name, foreign_tb_name, foreign_col_name } => {
                name_foreign_key(col_name, (col_name, foreign_tb_name, foreign_col_name));
            }
        }
    }

    if primary_key.len() >= 2 {
        return false;
    }

    if primary_key.len() == 1 {
        let primary_key = primary_key[0];
        for key_name in &primary_key {
            if !name_field.contains_key(col_name) {
                return false;
            }
        }
    }

    for (name, fk) in &name_foreign_key {
        if !name_field.contains_key(name) {
            return false;
        }
        if let Some(th) = sm.open_table(fk.1) {
            let cts = th.get_column_types();
            // valid type 
            // column must be primary
        } else {
            return false;
        }
    }
}

