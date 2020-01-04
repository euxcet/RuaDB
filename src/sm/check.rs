use crate::parser::ast::*;
use crate::rm::record::*;
use crate::index::btree::*;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::defer;

use super::system_manager::SystemManager;
use super::query_tree::QueryTree;


pub fn check_create_table(field_list: &Vec<Field>, sm: &SystemManager) -> bool {
    let mut name_field = HashMap::new();
    let mut primary_key: Vec<&Vec<String>> = Vec::new();
    let mut foreign_list: Vec<(&Vec<String>, &String, &Vec<String>)> = Vec::new();

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
            Field::ForeignKeyField { column_list, foreign_tb_name, foreign_column_list } => {
                foreign_list.push((column_list, foreign_tb_name, foreign_column_list))
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

    for (cols, ft_name, fcols) in foreign_list {
        if cols.len() != fcols.len() {
            return false;
        }
        let no_repeat = check_no_repeat(cols) && check_no_repeat(fcols);
        if !no_repeat {
            return false;
        }

        let all_found = cols.iter().fold(true, |all_found, col_name| all_found && name_field.contains_key(col_name));
        if !all_found {
            return false;
        }

        let th = sm.open_table(ft_name, false);
        if th.is_none() {
            return false;
        }

        let th = th.unwrap();
        defer!(th.close());
        let fmap = th.get_column_types_as_hashmap();
        let all_found = fcols.iter().fold(true, |all_found, fcol_name| all_found && fmap.contains_key(fcol_name));
        if !all_found {
            return false;
        }

        use crate::parser::ast;
        let tys: Vec<&ast::Type> = cols.iter().map(|col_name| name_field.get(col_name).unwrap().1).collect();
        let fcts: Vec<&ColumnType> = fcols.iter().map(|fcol_name| fmap.get(fcol_name).unwrap()).collect();

        let type_valid = tys.iter().zip(fcts.iter()).fold(true, |all_valid, (ty, fct)| all_valid && (fct.data_type.of_same_type(ty)));
        if !type_valid {
            return false;
        }
        let pri_cols = th.get_primary_cols();
        if pri_cols.is_none() {
            return false;
        }
        let pri_cols = pri_cols.unwrap().cols;
        if fcts.len() < pri_cols.len() {
            return false;
        }

        let same = fcts.iter().zip(pri_cols.iter()).fold(true, |prefix, (fct, pri_col)| prefix && (fct == &pri_col));
        if !same {
            return false;
        }
    }

    // for (name, fk) in &name_foreign_key {
    //     if !name_field.contains_key(name) {
    //         return false;
    //     }
    //     let th = sm.open_table(fk.1, false);
    //     if th.is_none() {
    //         return false;
    //     }
    //     let th = th.unwrap();
    //     let primary_cols = th.get_primary_cols();
    //     defer!(th.close());

    //     if primary_cols.is_none() {
    //         return false;
    //     }
    //     let primary_cols = primary_cols.unwrap().cols;
    //     if primary_cols.len() != 1 {
    //         return false;
    //     }
    //     if &primary_cols[0].name != fk.2 {
    //         return false;
    //     }

    //     let foreign_type = &primary_cols[0].data_type;
    //     let this_type = name_field.get(name).unwrap().1;
    //     if !foreign_type.of_same_type(this_type) {
    //         return false;
    //     }
    // }

    true
}

pub fn check_drop_table(tb_name: &String, sm: &SystemManager) -> bool {
    let tables = sm.get_tables();
    for table in &tables {
        if table != tb_name {
            let th = sm.open_table(table, false).unwrap();
            defer!(th.close());
            let cts = th.get_column_types().cols;
            let foreign_this = cts.iter().fold(false, |foreign, ct| foreign || (ct.is_foreign && &ct.foreign_table_name == tb_name));
            if foreign_this {
                return false;
            }
        }
    }
    true
}

pub fn table_foreign_this_table(tb_name: &String, sm: &SystemManager) -> Vec<String> {
    sm.get_tables().into_iter().filter(
        |name| {
            if name == tb_name {
                false
            } else {
                let th = sm.open_table(&name, false).unwrap();
                defer!(th.close());
                let cts = th.get_column_types().cols;
                let foreign_this = cts.iter().fold(false, |foreign, ct| foreign || (ct.is_foreign && &ct.foreign_table_name == tb_name));
                foreign_this
            }
        }
    ).collect()
}



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
    if duplicate {
        return false;
    }

    let mut ft_name_fk_this_col: HashMap<&String, HashMap<&String, &ColumnType>> = HashMap::new();

    for ct in cols {
        if ct.is_foreign {
            let cs = ft_name_fk_this_col.entry(&ct.foreign_table_name).or_insert(HashMap::new());
            cs.insert(&ct.foreign_table_column, ct);
        }
    }

    for (ft_name, fk_this_col) in ft_name_fk_this_col {
        // in a single foreign table
        let fth = sm.open_table(ft_name, false).unwrap();
        defer!(fth.close());

        let primary_cols = fth.get_primary_cols().unwrap().cols;
        let pri_btree = fth.get_primary_btree().unwrap();
        assert!(fk_this_col.len() == primary_cols.len());

        // TODO: support part of primary key
        // TODO: support null foreign key
        let ordered_index: Vec<u32> = primary_cols.iter().map(|p| fk_this_col.get(&p.name).unwrap().index).collect();
        for record in &records {
            let ri = RawIndex::from_record(record, &ordered_index);
            let res = pri_btree.search_record(&ri);
            if res.is_none() {
                return false;
            }
        }
    }
    true
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

pub fn check_delete(tb_name: &String, map: &HashMap<String, ColumnType>, where_clause: &Option<Vec<WhereClause>>, sm: &SystemManager) -> bool {
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

    // check if foreign
    let foreign_tables = table_foreign_this_table(tb_name, sm);
    if foreign_tables.len() > 0 {
        let database = sm.current_database.as_ref().unwrap();
        let mut tree = QueryTree::new(&sm.root_dir, database, sm.rm.clone());
        tree.build(&vec![tb_name.clone()], &Selector::All, where_clause);
        let record_list = tree.query();

        let th = sm.open_table(tb_name, false).unwrap();
        defer!(th.close());
        let primary_cols = th.get_primary_column_index().unwrap();
        let ris: Vec<RawIndex> = record_list.record.iter().map(
            |record| {
                RawIndex::from_record(record, &primary_cols)
            }
        ).collect();

        for name in foreign_tables {
            let fth = sm.open_table(&name, false).unwrap();
            defer!(fth.close());
            let foreign_btrees = fth.get_btrees().into_iter().filter_map(
                |btree| {
                    if btree.is_foreign() && btree.get_foreign_table_name() == tb_name {
                        Some(btree)
                    } else {
                        None
                    }
                }
            );
            for fb in foreign_btrees {
                for ri in &ris {
                    if fb.search_record(ri).is_some() {
                        return false;
                    }
                }
            }
        }
    }

    true
}

pub fn check_update(tb_name: &String, map: &HashMap<String, ColumnType>, set_clause: &Vec<SetClause>, where_clause: &Option<Vec<WhereClause>>, sm: &SystemManager) -> bool {
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

    let set_column: Vec<String> = set_clause.iter().map(|s| s.col_name.clone()).collect();
    let no_repeat = check_no_repeat(&set_column);
    if !no_repeat {
        return false;
    }


    let database = sm.current_database.as_ref().unwrap();
    let mut tree = QueryTree::new(&sm.root_dir, database, sm.rm.clone());
    tree.build(&vec![tb_name.clone()], &Selector::All, where_clause);
    let record_list = tree.query();

    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());

    let affected_cols_index: HashSet<_> = set_column.iter().map(|name| map.get(name).unwrap().index).collect();
    let pri_cols = th.get_primary_column_index().unwrap();
    let pri_affected = pri_cols.iter().fold(false, |affected, pri_index| affected || affected_cols_index.contains(pri_index));
    if pri_affected {
        // primary key affected
        let foreign_table = table_foreign_this_table(tb_name, sm);
        if foreign_table.len() > 0 {
            // referenced by other table
            for table in foreign_table {
                let fth = sm.open_table(&table, false).unwrap();
                defer!(fth.close());
                // find other table reference these primary columns
                let foreign_btrees = fth.get_btrees().into_iter().filter_map(
                    |btree| {
                        if btree.is_foreign() && btree.get_foreign_table_name() == tb_name {
                            Some(btree)
                        } else {
                            None
                        }
                    }
                );
                // tranverse all foreign keys btrees to find any row reference these record
                for ftree in foreign_btrees {
                    for record in &record_list.record {
                        if ftree.search_record(&RawIndex::from_record(record, &pri_cols)).is_some() {
                            return false;
                        }
                    }
                }
            }
        }
        // affect foreign btree
        let affected_foreign_btree: Vec<BTree> = th.get_btrees().into_iter().filter(
            |t| 
                t.is_foreign() &&
                t.index_col.iter().fold(false, |affected, i| affected || affected_cols_index.contains(i))
        ).collect();

        let (ptr, mut pri_btree) = th.get_primary_btree_with_ptr().unwrap();
        let mut deleted = Vec::new();
        let mut inserted = Vec::new();

        let mut duplicate = false;
        let mut foreign_incorrect = false;

        for btree in affected_foreign_btree {
            let foreign_table = btree.get_foreign_table_name();
            let fth = sm.open_table(foreign_table, false).unwrap();
            defer!(fth.close());
            let pri_btree = fth.get_primary_btree().unwrap();

            for (ptr, record) in record_list.ptrs.iter().zip(record_list.record.iter()) {
                let mut new_record = record.clone();
                new_record.set_(set_clause, &record_list.ty);

                let ri = RawIndex::from_record(record, &btree.index_col);
                if pri_btree.search_record(&ri).is_none() {
                    return false;
                }
            }
        }

        for (ptr, record) in record_list.ptrs.iter().zip(record_list.record.iter()) {
            let mut new_record = record.clone();
            new_record.set_(set_clause, &record_list.ty);

            let ri = RawIndex::from_record(record, &pri_cols);
            pri_btree.delete_record(&ri, ptr.to_u64());
            deleted.push((ptr, ri));

            let new_ri = RawIndex::from_record(&new_record, &pri_cols);
            let dup = pri_btree.insert_record(&new_ri, ptr.to_u64(), false);
            if dup {
                duplicate = true;
                break;
            } else {
                inserted.push((ptr, new_ri));
            }
        }

        while let Some((ptr, ri)) = inserted.pop() {
            pri_btree.delete_record(&ri, ptr.to_u64());
        }

        while let Some((ptr, ri)) = deleted.pop() {
            pri_btree.insert_record(&ri, ptr.to_u64(), false);
        }
        th.update_btree(&ptr, &pri_btree);

        if duplicate {
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
    btrees.iter().fold(false, |found, btree| found || (&btree.index_name == idx_name && btree.is_index()))
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

pub fn check_drop_column(tb_name: &String, col_name: &String, sm: &SystemManager) -> bool {
    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());
    let map = th.get_column_types_as_hashmap();

    if !map.contains_key(col_name) {
        return false;
    }

    let del_index = map.get(col_name).unwrap().index;
    let pri_cols = th.get_primary_column_index();

    if let Some(pri_cols) = pri_cols {
        if pri_cols.len() >= 2 {
            for i in pri_cols {
                if i == del_index {
                    return false;
                }
            }
        } else if pri_cols == vec![del_index] {
            if table_foreign_this_table(tb_name, sm).len() > 0 {
                return false;
            }
        }
    }

    let btrees = th.get_btrees();

    for t in btrees {
        let cols = t.index_col;
        if cols.len() > 1 {
            for i in cols {
                if i == del_index {
                    return false;
                }
            }
        }
    }


    true
}

pub fn check_change_column(tb_name: &String, col_name: &String, field: &Field, sm: &SystemManager) -> bool {
    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());
    let map = th.get_column_types_as_hashmap();
    if !map.contains_key(col_name) {
        return false;
    }

    let cf = match field {
        Field::ColumnField{ col_name: _, ty: _, not_null: _, default_value: _ } => true,
        _ => false,
    };

    if !cf {
        return false;
    }

    let origin_col = map.get(col_name).unwrap();
    let index = origin_col.index;

    let affected = th.get_btrees().iter().fold(false, 
        |affected, t| 
            affected || t.index_col.iter().fold(false, |affected, &i| affected || index == i)
    );
    if affected {
        return false;
    }

    let new_col = ColumnType::from_field(&origin_col.tb_name, index, field);
    if map.contains_key(&new_col.name) {
        return false;
    }

    if !origin_col.data_type.comparable(&new_col.data_type) {
        return false;
    }

    true
}

// TODO: merge: btree
pub fn check_add_primary_key(tb_name: &String, column_list: &Vec<String>, sm: &SystemManager) -> bool {
    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());
    let map = th.get_column_types_as_hashmap();
    if !(check_no_repeat(column_list)
        && th.get_primary_btree().is_none()
        && column_list.iter().fold(true, |all_found, name| all_found && map.contains_key(name))) {
            return false;
    }
    let pri_cols: Vec<u32> = column_list.iter().map(|name| map.get(name).unwrap().index).collect();

    let database = sm.current_database.as_ref().unwrap();
    let mut tree = QueryTree::new(&sm.root_dir, database, sm.rm.clone());
    tree.build(&vec![tb_name.clone()], &Selector::All, &None);
    let record_list = tree.query();

    let mut btree = BTree::new(&th, pri_cols.clone(), "", BTree::primary_ty());

    let mut duplicate = false;
    for (ptr, record) in record_list.ptrs.iter().zip(record_list.record.iter()) {
        let ri = RawIndex::from_record(record, &pri_cols);
        let dup = btree.insert_record(&ri, ptr.to_u64(), false);
        if dup {
            duplicate = true;
            break;
        }
    }
    btree.clear();
    if duplicate {
        return false;
    }

    true
}

pub fn check_drop_primary_key(tb_name: &String, sm: &SystemManager) -> bool {
    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());

    let btree = th.get_primary_btree();
    if btree.is_none() {
        return false;
    }

    let foreign_tables = table_foreign_this_table(tb_name, sm);
    if foreign_tables.len() > 0 {
        return false;
    }

    true
}

pub fn check_drop_constraint_primary_key(tb_name: &String, pk_name: &String, sm: &SystemManager) -> bool {
    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());

    let btree = th.get_primary_btree();
    if btree.is_none() {
        return false;
    } else if &btree.unwrap().index_name == pk_name {
        return false;
    }

    let foreign_tables = table_foreign_this_table(tb_name, sm);
    if foreign_tables.len() > 0 {
        return false;
    }

    true
}

pub fn check_add_constraint_foreign_key(tb_name: &String, fk_name: &String, column_list: &Vec<String>, foreign_tb_name: &String, foreign_column_list: &Vec<String>, sm: &SystemManager) -> bool {
    if column_list.len() != foreign_column_list.len() {
        return false;
    }
    let th = sm.open_table(tb_name, false).unwrap();
    let exists = th.get_btrees().iter().filter(|t| t.is_foreign() && t.get_foreign_constraint_name() == fk_name).next().is_some();
    if exists {
        th.close();
        return false;
    }

    let map = th.get_column_types_as_hashmap();

    let all_found = column_list.iter().fold(true, |all_found, c| all_found && map.contains_key(c));
    if !all_found {
        th.close();
        return false;
    }

    let fth = sm.open_table(foreign_tb_name, false);
    if fth.is_none() {
        th.close();
        return false;
    }

    let fth = fth.unwrap();
    defer!(fth.close());
    let fmap = fth.get_column_types_as_hashmap();

    let all_found = foreign_column_list.iter().fold(true, |all_found, c| all_found && fmap.contains_key(c));
    if !all_found {
        th.close();
        return false;
    }

    let this_table_column: Vec<ColumnType> = column_list.iter().map(|c| map.get(c).unwrap().clone()).collect();
    let foreign_table_column: Vec<ColumnType> = foreign_column_list.iter().map(|c| fmap.get(c).unwrap().clone()).collect();

    let this_cols_index: Vec<u32> = this_table_column.iter().map(|c| c.index).collect();
    let foreign_cols_index: Vec<u32> = foreign_table_column.iter().map(|c| c.index).collect();
    let pri_btree = fth.get_primary_btree();
    if pri_btree.is_none() {
        th.close();
        return false;
    }
    let pri_btree = pri_btree.unwrap();
    if &pri_btree.index_col != &foreign_cols_index {
        th.close();
        return false;
    }

    for (this_col, foreign_col) in this_table_column.iter().zip(foreign_table_column.iter()) {
        let this_type = &this_col.data_type;
        let foreign_type = &foreign_col.data_type;
        if !this_type.comparable(foreign_type) {
            th.close();
            return false;
        }
    }
    th.close();

    let database = sm.current_database.as_ref().unwrap();
    let mut tree = QueryTree::new(&sm.root_dir, database, sm.rm.clone());
    tree.build(&vec![tb_name.clone()], &Selector::All, &None);
    let record_list = tree.query();

    for record in &record_list.record {
        let bucket = pri_btree.search_record(&RawIndex::from_record(record, &this_cols_index));
        if bucket.is_none() {
            return false;
        }
    }

    true
}

pub fn check_drop_constraint_foreign_key(tb_name: &String, fk_name: &String, sm: &SystemManager) -> bool {
    let th = sm.open_table(tb_name, false).unwrap();
    defer!(th.close());
    let exists = th.get_btrees().iter().filter(|t| t.is_foreign() && t.get_foreign_constraint_name() == fk_name).next().is_some();
    if !exists {
        return false;
    }

    true
}
