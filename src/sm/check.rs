use crate::parser::ast::*;
use crate::sm::system_manager::SystemManager;
use crate::rm::record::*;
use crate::index::btree::*;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::defer;

pub fn check_create_table(field_list: &Vec<Field>, sm: &SystemManager) -> bool {
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
        let no_repeat = check_no_repeat(primary_key);
        if !no_repeat {
            return false;
        }
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
        let th = sm.open_table(fk.1, false);
        if th.is_none() {
            return false;
        }
        let th = th.unwrap();
        let primary_cols = th.get_primary_cols();
        defer!(th.close());

        if primary_cols.is_none() {
            return false;
        }
        let primary_cols = primary_cols.unwrap().cols;
        if primary_cols.len() != 1 {
            return false;
        }
        if &primary_cols[0].name != fk.2 {
            return false;
        }

        let foreign_type = &primary_cols[0].data_type;
        let this_type = name_field.get(name).unwrap().1;
        if !foreign_type.of_same_type(this_type) {
            return false;
        }
    }

    true
}

pub fn check_drop_table(tb_name: &String, sm: &SystemManager) -> bool {
    let tables = sm.get_tables();
    for table in &tables {
        if table != tb_name {
            let th = sm.open_table(table, false).unwrap();
            let cts = th.get_column_types().cols;
            let foreign_this = cts.iter().fold(false, |foreign, ct| foreign || (ct.is_foreign && &ct.foreign_table_name == tb_name));
            if foreign_this {
                return false;
            }
        }
    }

    true
}

// TODO: foreign key
pub fn check_insert_value(tb_name: &String, value_lists: &Vec<Vec<Value>>, sm: &SystemManager) -> bool {
    use crate::rm::pagedef::*;
    use crate::rm::in_file::*;

    let th = sm.open_table(tb_name, false).unwrap();
    let cts = th.get_column_types();

    let cols = &cts.cols;
    let col_num = cols.len();
    defer!(th.close());

    for values in value_lists {
        if values.len() != col_num {
            return false;
        }
        for i in 0..col_num {
            if !cols[i].data_type.valid_value(&values[i]) {
                return false;
            }
            if values[i].is_null() && !cols[i].can_be_null {
                return false;
            }
        }
    }
    let records: Vec<Record> = value_lists.iter().map(|v| Record::from_value_lists(v, &cts.cols)).collect();

    let mut inserted: Vec<(StrPointer, RawIndex)> = Vec::new();
    let t = th.get_primary_btree_with_ptr();

    let mut duplicate = false;
    if let Some((ptr_ptr, mut pri_btree)) = t {
        for record in &records {
            let (ptr, rif) = th.insert_record_get_record_in_file(record);
            let ri = RawIndex::from(&rif.get_index(&th, &pri_btree.index_col));
            let dup = pri_btree.insert_record(&ri, ptr.to_u64(), false);
            if dup {
                duplicate = true;
                break;
            } else {
                inserted.push((ptr, ri));
            }
        }
        while let Some((ptr, ri)) = inserted.pop() {
            pri_btree.delete_record(&ri, ptr.to_u64());
        }
        th.update_btree(&ptr_ptr, &pri_btree);
    }
    duplicate
}

pub fn check_no_repeat(names: &Vec<String>) -> bool {
    let mut set = HashSet::new();
    for name in names {
        if set.contains(name) {
            return false;
        }
        set.insert(name);
    }
    true
}

pub fn check_select(tb_cols: &HashMap<&String, HashMap<String, ColumnType>>, selector: &Selector, where_clause: &Option<Vec<WhereClause>>) -> bool {
    let mut col_tbs: HashMap<&String, HashSet<&String>> =  HashMap::new();
    for (tb_name, cols) in tb_cols {
        for (col_name, _) in cols {
            let tbs_contains_this_col_name = col_tbs.entry(col_name).or_insert(HashSet::new());
            tbs_contains_this_col_name.insert(*tb_name);
        }
    }

    let valid_qualified_col = |qualified_col: &Column| -> bool {
        let ts = col_tbs.get(&qualified_col.col_name);
        if let Some(ref tb) = qualified_col.tb_name {
            !ts.is_none() && ts.unwrap().contains(tb)
        } else {
            !ts.is_none() && ts.unwrap().len() == 1
        }
    };

    let get_col = |qualified_col: &Column| -> &ColumnType {
        let tb_name = {
            if let Some(ref tb) = qualified_col.tb_name {
                tb
            } else {
                let ts = col_tbs.get(&qualified_col.col_name).unwrap();
                assert!(ts.len() == 1);
                let mut v = Vec::new();
                for t in ts {
                    v.push(*t);
                }
                v[0]
            }
        };
        tb_cols.get(tb_name).unwrap().get(&qualified_col.col_name).unwrap()
    };

    match selector {
        Selector::Columns(qualified_cols) => {
            for qualified_col in qualified_cols {
                if !valid_qualified_col(qualified_col) {
                    return false;
                }
            }
        },
        _ => {}, 
    }

    if let Some(where_clause) = where_clause {
        for sub_where in where_clause {
            match sub_where {
                WhereClause::IsAssert {col, null: _} => {
                    if !valid_qualified_col(col) {
                        return false;
                    }
                },
                WhereClause::Comparison {col, op: _, expr} => {
                    if !valid_qualified_col(col) {
                        return false;
                    }
                    let ct = get_col(col);
                    match expr {
                        Expr::Value(v) => {
                            if !ct.data_type.valid_value(v) {
                                return false;
                            }
                        },
                        Expr::Column(c) => {
                            let another_ct = get_col(c);
                            if !ct.data_type.comparable(&another_ct.data_type) {
                                return false;
                            }
                        }
                    }
                },
            }
        }
    }
    true
}

pub fn check_delete(tb_name: &String, map: &HashMap<String, ColumnType>, where_clause: &Option<Vec<WhereClause>>) -> bool {
    let valid_qualified_col = |qualified_col: &Column| -> bool {
        if let Some(ref tb) = qualified_col.tb_name {
            tb == tb_name && map.contains_key(&qualified_col.col_name)
        } else {
            map.contains_key(&qualified_col.col_name)
        }
    };

    let get_col = |qualified_col: &Column| -> &ColumnType {
        map.get(&qualified_col.col_name).unwrap()
    };

    if let Some(where_clause) = where_clause {
        for sub_where in where_clause {
            match sub_where {
                WhereClause::IsAssert {col, null: _} => {
                    if !valid_qualified_col(col) {
                        return false;
                    }
                },
                WhereClause::Comparison {col, op: _, expr} => {
                    if !valid_qualified_col(col) {
                        return false;
                    }
                    let ct = get_col(col);
                    match expr {
                        Expr::Value(v) => {
                            if !ct.data_type.valid_value(v) {
                                return false;
                            }
                        },
                        Expr::Column(c) => {
                            let another_ct = get_col(c);
                            if !ct.data_type.comparable(&another_ct.data_type) {
                                return false;
                            }
                        }
                    }
                },
            }
        }
    }
    true
}

pub fn check_update(tb_name: &String, map: &HashMap<String, ColumnType>, set_clause: &Vec<SetClause>, where_clause: &Option<Vec<WhereClause>>) -> bool {
    let valid_qualified_col = |qualified_col: &Column| -> bool {
        if let Some(ref tb) = qualified_col.tb_name {
            tb == tb_name && map.contains_key(&qualified_col.col_name)
        } else {
            map.contains_key(&qualified_col.col_name)
        }
    };

    let get_col = |col_name: &String| -> &ColumnType {
        map.get(col_name).unwrap()
    };

    if let Some(where_clause) = where_clause {
        for sub_where in where_clause {
            match sub_where {
                WhereClause::IsAssert {col, null: _} => {
                    if !valid_qualified_col(col) {
                        return false;
                    }
                },
                WhereClause::Comparison {col, op: _, expr} => {
                    if !valid_qualified_col(col) {
                        return false;
                    }
                    let ct = get_col(&col.col_name);
                    match expr {
                        Expr::Value(v) => {
                            if !ct.data_type.valid_value(v) {
                                return false;
                            }
                        },
                        Expr::Column(c) => {
                            let another_ct = get_col(&c.col_name);
                            if !ct.data_type.comparable(&another_ct.data_type) {
                                return false;
                            }
                        }
                    }
                },
            }
        }
    }

    for sub_set_clause in set_clause {
        let ct = get_col(&sub_set_clause.col_name);
        if !ct.data_type.valid_value(&sub_set_clause.value) {
            return false;
        }
    }

    true
}

pub fn check_create_index(idx_name: &String, map: &HashMap<String, ColumnType>, column_list: &Vec<String>, btrees: &Vec<BTree>) -> bool {
    column_list.len() > 0 
        && check_no_repeat(column_list)
        && !btrees.iter().fold(false, |found, btree| found || (&btree.index_name == idx_name))
        && column_list.iter().fold(true, |found, column_name| found && map.contains_key(column_name))
}

pub fn check_drop_index(idx_name: &String, btrees: &Vec<BTree>) -> bool {
    idx_name != "primary" 
        && btrees.iter().fold(false, |found, btree| found || (&btree.index_name == idx_name))
}

pub fn check_add_column(map: &HashMap<String, ColumnType>, field: &Field) -> bool {
    match field {
        Field::ColumnField { col_name, ty: _, not_null: _, default_value: _ } => {
            if map.contains_key(col_name) {
                return false;
            }
        },
        _ => return false,
    }
    true
}

pub fn check_drop_column(map: &HashMap<String, ColumnType>, col_name: &String) -> bool {
    map.contains_key(col_name)
}

pub fn check_change_column(map: &HashMap<String, ColumnType>, col_name: &String, field: &Field) -> bool {
    if !map.contains_key(col_name) {
        return false;
    }

    let origin_col = map.get(col_name).unwrap();
    let index = origin_col.index;
    let new_col = ColumnType::from_field(&origin_col.tb_name, index, field);
    // TODO: ensure column type can be converted
    // TODO: out of range value

    true
}

pub fn check_add_primary_key(map: &HashMap<String, ColumnType>, column_list: &Vec<String>) -> bool {
    check_no_repeat(column_list)
        && map.iter().fold(true, |no_primary, (_, ct)| no_primary && ct.is_primary)
        && column_list.iter().fold(true, |all_found, name| all_found && map.contains_key(name))
}