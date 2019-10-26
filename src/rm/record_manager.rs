use std::cell::RefCell;
use std::rc::Rc;

use super::file_handler::FileHandler;
use super::filesystem::bufmanager::buf_page_manager::BufPageManager;
use super::record::*;

struct RecordManager {
    bpm: Rc<RefCell<BufPageManager>>,
}

impl RecordManager {
    pub fn new() -> Self {
        Self {
            bpm: Rc::new(RefCell::new(BufPageManager::new())),
        }
    }

    pub fn create(&mut self, path: &str) {
        assert!(self.bpm.borrow_mut().file_manager.create_file(path).is_ok());
    }

    pub fn delete(&mut self, path: &str) {
        assert!(self.bpm.borrow_mut().file_manager.delete_file(path).is_ok());
    }

    pub fn open(&mut self, path: &str) -> FileHandler {
        let fd = self.bpm.borrow_mut().file_manager.open_file(path);
        FileHandler::new(fd, self.bpm.clone())
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
    use crate::utils::random;

    #[test]
    fn basic_test() {
        let mut r = RecordManager::new();
        r.create("d:/Rua/test/basic_test.rua");
        let mut fh = r.open("d:/Rua/test/basic_test.rua");

        let columns = vec![
            ColumnType {
                name: String::from("id"),
                data_type: Type::Int(None),
                has_index: true,
                is_primary: true,
                .. Default::default()
            },
            ColumnType {
                name: String::from("name"),
                data_type: Type::Str(100, Some(String::from("lyt"))),
                .. Default::default()
            },
            ColumnType {
                name: String::from("value"),
                data_type: Type::Float(Some(123.456f64)),
                has_default: true,
                .. Default::default()
            }
        ];

        let record = Record {
            record: vec![
                ColumnData {
                    index: 0,
                    default: false,
                    data: Some(Data::Int(65535)),
                },
                ColumnData {
                    index: 1,
                    default: false,
                    data: Some(Data::Str(String::from("str"))),
                },
                ColumnData {
                    index: 2,
                    default: true,
                    data: None,
                }
            ],
        };

        fh.set_columns(&columns);

        let rid = fh.create_record(&record);


        let r = fh.get_record(rid);
        assert_eq!(r.record.len(), record.record.len());
        assert_eq!(r.record[0].data, Some(Data::Int(65535)));
        assert_eq!(r.record[1].data, Some(Data::Str(String::from("str"))));
        assert_eq!(r.record[2].data, Some(Data::Float(123.456f64)));

        fh.update_record(rid, &ColumnData {
                    index: 0,
                    default: false,
                    data: Some(Data::Int(i64::max_value())),
                });
        fh.update_record(rid, &ColumnData {
                    index: 1,
                    default: false,
                    data: Some(Data::Str(String::from("fuck"))),
                });
        fh.update_record(rid, &ColumnData {
                    index: 2,
                    default: false,
                    data: Some(Data::Float(55555.55555f64)),
                });

        let r = fh.get_record(rid);
        assert_eq!(r.record[0].data, Some(Data::Int(i64::max_value())));
        assert_eq!(r.record[1].data, Some(Data::Str(String::from("fuck"))));
        assert_eq!(r.record[2].data, Some(Data::Float(55555.55555f64)));

        fh.close();
    }


    fn gen_random_columns(gen: &mut random::Generator, number: usize, MAX_STRING_LENGTH: u32) -> Vec<ColumnType> {
        let mut columns = Vec::new();
        for i in 0..number {
            let ty_rand = gen.gen::<u8>() % 4;
            let has_default = gen.gen::<bool>();
            let ty: Type = match ty_rand {
                0 => Type::Int(if has_default {Some(gen.gen::<i64>())} else {None}),
                1 => Type::Float(if has_default {Some(gen.gen::<f64>())} else {None}),
                2 => Type::Date(if has_default {Some(gen.gen::<u64>())} else {None}),
                3 => Type::Str(MAX_STRING_LENGTH, if has_default {Some(gen.gen_string_s(MAX_STRING_LENGTH as usize))} else {None}),
                _ => unreachable!()
            };

            columns.push(
                ColumnType {
                    index: i as u32,
                    name: gen.gen_string(MAX_COLUMN_NAME_LENGTH),
                    data_type: ty,
                    has_default: has_default,
                    default_null: !has_default,
                    .. Default::default()
                }
            );
        }
        columns
    }

    #[test]
    #[should_panic]
    fn set_columns_test() {
        let mut r = RecordManager::new();
        r.create("d:/Rua/test/records_test.rua");
        let mut fh = r.open("d:/Rua/test/records_test.rua");

        let mut gen = random::Generator::new(true);

        const MAX_STRING_LENGTH: u32 = 1000;
        let columns = gen_random_columns(&mut gen, MAX_COLUMN_NUMBER + 1, MAX_STRING_LENGTH);

        fh.set_columns(&columns);
        fh.close()
    }

    #[test]
    fn full_test() {

        let mut r = RecordManager::new();
        r.create("d:/Rua/test/records_test.rua");
        let mut fh = r.open("d:/Rua/test/records_test.rua");

        let mut gen = random::Generator::new(true);

        const MAX_STRING_LENGTH: u32 = 1000;
        // let columns = gen_random_columns(&mut gen, MAX_COLUMN_NUMBER, MAX_STRING_LENGTH);
        let columns = gen_random_columns(&mut gen, 10, MAX_STRING_LENGTH);

        fh.set_columns(&columns);
        fh.close();

        let mut fh = r.open("d:/Rua/test/records_test.rua");
        // let columns_ = fh.get_columns();

        // assert_eq!(columns, columns_);

        fh.close();

        /*

        let record = Record {
            record: vec![
                ColumnData {
                    index: 0,
                    default: false,
                    data: Some(Data::Int(65535)),
                },
                ColumnData {
                    index: 1,
                    default: false,
                    data: Some(Data::Str(String::from("str"))),
                },
                ColumnData {
                    index: 2,
                    default: true,
                    data: None,
                }
            ],
        };


        let rid = fh.create_record(&record);


        let r = fh.get_record(rid);
        assert_eq!(r.record.len(), record.record.len());
        assert_eq!(r.record[0].data, Some(Data::Int(65535)));
        assert_eq!(r.record[1].data, Some(Data::Str(String::from("str"))));
        assert_eq!(r.record[2].data, Some(Data::Float(123.456f64)));

        fh.update_record(rid, &ColumnData {
                    index: 0,
                    default: false,
                    data: Some(Data::Int(i64::max_value())),
                });
        fh.update_record(rid, &ColumnData {
                    index: 1,
                    default: false,
                    data: Some(Data::Str(String::from("fuck"))),
                });
        fh.update_record(rid, &ColumnData {
                    index: 2,
                    default: false,
                    data: Some(Data::Float(55555.55555f64)),
                });

        let r = fh.get_record(rid);
        assert_eq!(r.record[0].data, Some(Data::Int(i64::max_value())));
        assert_eq!(r.record[1].data, Some(Data::Str(String::from("fuck"))));
        assert_eq!(r.record[2].data, Some(Data::Float(55555.55555f64)));
        */

    }
}