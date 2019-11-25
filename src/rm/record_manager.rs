use std::cell::RefCell;
use std::rc::Rc;

use crate::settings;
use super::file_handler::FileHandler;
use super::filesystem::bufmanager::buf_page_manager::BufPageManager;
use super::table_handler::*;

pub struct RecordManager {
    bpm: Rc<RefCell<BufPageManager>>,
    root_dir: String,
}

impl RecordManager {
    pub fn new() -> Self {
        let settings = settings::Settings::new().unwrap();

        #[cfg(target_os = "macos")]
        let rd = settings.database.rd_macos;
        #[cfg(target_os = "windows")]
        let rd = settings.database.rd_windows;
        #[cfg(target_os = "linux")]
        let rd = settings.database.rd_linux;

        Self {
            bpm: Rc::new(RefCell::new(BufPageManager::new())),
            root_dir: rd,
        }
    }

    pub fn create_table(&mut self, path: &str) {
        assert!(self.bpm.borrow_mut().file_manager.create_file((self.root_dir.clone() + path).as_str()).is_ok());
    }

    pub fn delete_table(&mut self, path: &str) {
        assert!(self.bpm.borrow_mut().file_manager.delete_file((self.root_dir.clone() + path).as_str()).is_ok());
    }

    pub fn open_table(&mut self, path: &str) -> TableHandler {
        let fd = self.bpm.borrow_mut().file_manager.open_file((self.root_dir.clone() + path).as_str());
        TableHandler::new(FileHandler::new(fd, self.bpm.clone()))
    }
}

impl Drop for RecordManager {
    fn drop(&mut self) {
        self.bpm.borrow_mut().close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::record::*;
    use crate::utils::random;

    fn gen_random_columns(gen: &mut random::Generator, number: usize, max_string_length: usize) -> Vec<ColumnType> {
        let mut columns = Vec::new();
        for i in 0..number {
            let ty_rand = gen.gen::<u8>() % 4;
            let has_default = gen.gen::<bool>();
            let ty: Type = match ty_rand {
                0 => Type::Int(if has_default {Some(gen.gen::<i64>())} else {None}),
                1 => Type::Float(if has_default {Some(gen.gen::<f64>())} else {None}),
                2 => Type::Date(if has_default {Some(gen.gen::<u64>())} else {None}),
                3 => Type::Str(if has_default {Some(gen.gen_string_s(max_string_length))} else {None}),
                4 => Type::Numeric(if has_default {Some(gen.gen::<i64>())} else {None}),
                _ => unreachable!()
            };

            columns.push(
                ColumnType {
                    index: i as u32,
                    name: gen.gen_string(max_string_length),
                    data_type: ty,
                    has_default: has_default,
                    default_null: !has_default,
                    .. Default::default()
                }
            );
        }
        columns
    }

    fn gen_record(gen: &mut random::Generator, columns: &Vec<ColumnType>, max_string_length: usize) -> Record {
        let mut record = Vec::new();
        for c in columns.iter() {
            let default = if c.has_default {gen.gen()} else {false};
            record.push(ColumnData {
                index: c.index,
                data: if default {
                    match &c.data_type {
                        &Type::Int(Some(x)) => Some(Data::Int(x)),
                        &Type::Float(Some(x)) => Some(Data::Float(x)),
                        &Type::Date(Some(x)) => Some(Data::Date(x)),
                        &Type::Str(Some(ref x)) => Some(Data::Str(x.clone())),
                        &Type::Numeric(Some(x)) => Some(Data::Numeric(x)),
                        _ => unreachable!(),
                    }
                } else {
                    match &c.data_type {
                        &Type::Int(_) => Some(Data::Int(gen.gen::<i64>())),
                        &Type::Float(_) => Some(Data::Float(gen.gen::<f64>())),
                        &Type::Date(_) => Some(Data::Date(gen.gen::<u64>())),
                        &Type::Str(_) => Some(Data::Str(gen.gen_string_s(max_string_length as usize))),
                        &Type::Numeric(_) => Some(Data::Numeric(gen.gen::<i64>()))
                    }
                },
                default: default,
            });
        }
        Record {
            record: record
        }
    }

    #[test]
    fn alloc_record() {
        let mut gen = random::Generator::new(false);
        const MAX_STRING_LENGTH: usize = 10;
        const MAX_RECORD_NUMBER: usize = 1000;

        let mut r = RecordManager::new();
        r.create_table("alloc_record.rua");

        let columns = gen_random_columns(&mut gen, 10, MAX_STRING_LENGTH);
        let mut c_ptrs = Vec::new();
        let th = r.open_table("alloc_record.rua");
        for c in &columns {
            c_ptrs.push(th.insert_column_type(c));
        }
        th.close();

        let th = r.open_table("alloc_record.rua");
        for i in 0..columns.len() {
            assert_eq!(th.get_column_type(&c_ptrs[i]), columns[i]);
        }
        th.close();

        let mut ptrs = Vec::new();
        let mut records = Vec::new();

        let th = r.open_table("alloc_record.rua");
        for _ in 0..MAX_RECORD_NUMBER {
            let record = gen_record(&mut gen, &columns, MAX_STRING_LENGTH);
            ptrs.push(th.insert_record(&record));
            records.push(record);
        }
        th.close();

        let th = r.open_table("alloc_record.rua");
        for i in 0..ptrs.len() {
            assert_eq!(th.get_record(&ptrs[i]).0, records[i]);
        }
    }
}