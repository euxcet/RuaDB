use crate::parser::ast::*;
use crate::sm::system_manager::SystemManager;
use crate::rm::record;

use std::collections::HashMap;
use std::collections::HashSet;


pub fn valid_field_list(field_list: &Vec<Field>, sm: &SystemManager) -> bool {
    let mut name_field = HashMap::new();
    let mut primary_key: Vec<&Vec<String>> = Vec::new();
    let mut name_foreign_key = HashMap::new();

    for field in field_list {
        match field {
            Field::ColumnField {col_name, ty, not_null, default_value} => {
                if name_field.contains_key(col_name) {
                    return false;
                }
                if let Some(ref v) = default_value {
                    if !v.of_type(ty) {
                        return false;
                    }
                }
                name_field.insert(col_name, (col_name, ty, not_null, default_value));
            },
            Field::PrimaryKeyField {column_list} => {
                primary_key.push(column_list);
            },
            Field::ForeignKeyField {col_name, foreign_tb_name, foreign_col_name } => {
                if name_foreign_key.contains_key(col_name) {
                    return false;
                }
                name_foreign_key.insert(col_name, (col_name, foreign_tb_name, foreign_col_name));
            }
        }
    }

    if primary_key.len() >= 2 {
        return false;
    }

    if primary_key.len() == 1 {
        let primary_key = primary_key[0];
        for col_name in primary_key {
            if !name_field.contains_key(col_name) {
                return false;
            }
        }
    }

    for (name, fk) in &name_foreign_key {
        if !name_field.contains_key(name) {
            return false;
        }
        // has foreign table
        if let Some(th) = sm.open_table(fk.1, false) {
            let ct_map = th.get_column_types_as_hashmap();
            // has foreign column
            if let Some(foreign_col) = ct_map.get(fk.2) {
                let this_field = name_field.get(name).unwrap();
                let f_ty = &foreign_col.data_type;
                let t_ty = this_field.1;
                // has same type
                if !f_ty.of_same_type(t_ty) {
                    return false;
                }
                // foreign column must be primary
                // if primary_set_from_column_types_hashmap(&ct_map) != vec![foreign_col.name.clone()].iter().collect() {
                //     return false;
                // }
            }
            else {
                return false
            }
            th.close();
        } else {
            return false;
        }
    }

    true
}



