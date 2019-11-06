use std::mem::{transmute, size_of};
use super::table_handler::*;
use super::record::*;
use super::pagedef::*;

// structs that needed to be written to the file should be contained in the bytevec_decl! macro.
bytevec_decl! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct ColumnTypeInFile {
        pub name: u64,
        pub foreign_table_name: u64,
        pub index: u32,
        /*
            data_type [bit0, bit1, bit2, 0, 0, 0, 0, 0]
            bit meaning
            0   Data::Str
            1   Data::Int
            2   Data::Float
            3   Data::Date
            4   Data::Numeric
        */
        pub data_type: u8,
        pub data: u64,
        /*
            flags [0 .. 8]
            [can_be_null, has_index, has_default, is_primary, is_foreign, default_null, 0, 0]
        */
        pub flags: u8
    }
    #[derive(PartialEq, Eq, Debug)]
    pub struct RecordInFile {
        pub record: String
    }
}

/*
    ColumnDataInFile represents the specific arrangement of RecordInFile::record
    It's not stored in the file separately
*/
#[repr(C, packed)]
pub struct ColumnDataInFile {
    pub index: u32,
    /*
        flags [0 .. 8]
        [default, is_null, data_type_bit0, data_type_bit1, data_byte_bit2, 0, 0, 0]
        bit meaning
        0   Data::Str
        1   Data::Int
        2   Data::Float
        3   Data::Date
        4   Data::Numeric
    */
    pub flags: u8,
    pub data: u64
}

impl ColumnDataInFile {
    // &[u8] to ColumnDataInFile
    pub fn new(data: &[u8]) -> Self {
        ColumnDataInFile {
            index: unsafe {*(data.as_ptr() as *const u32)},
            flags: data[4],
            data: unsafe {*(data.as_ptr().add(5) as *const u64)},
        }
    }

    // ColumnData to ColumnDataInFile
    pub fn from(th: &TableHandler, cd: &ColumnData) -> Self {
        match &cd.data {
            Some(data) => {
                ColumnDataInFile {
                    index: cd.index,
                    flags: cd.default as u8 | match data {
                        Data::Str(_) => 0 << 2,
                        Data::Int(_) => 1 << 2,
                        Data::Float(_) => 2 << 2,
                        Data::Date(_) => 3 << 2,
                    },
                    data: match data {
                        Data::Str(d) => th.insert_string(&d).to_u64(),
                        Data::Int(d) => unsafe{transmute(*d)},
                        Data::Float(d) => unsafe{transmute(*d)},
                        Data::Date(d) => unsafe{transmute(*d)},
                    },
                }
            }
            None => {
                ColumnDataInFile {
                    index: cd.index,
                    flags: cd.default as u8 | 2,
                    data: 0,
                }
            }
        }
    }

    // ColumnDataInFile to ColumnData
    pub fn to_column_data(&self, th: &TableHandler) -> ColumnData {
        ColumnData {
            index: self.index,
            default: self.flags & 1 != 0,
            data: if self.flags & 2 == 0 {
                Some(match (self.flags >> 2) & 7 {
                        0 => Data::Str(th.get_string(&StrPointer::new(self.data))),
                        1 => Data::Int(unsafe{transmute(self.data)}),
                        2 => Data::Float(unsafe{transmute(self.data)}),
                        3 => Data::Date(unsafe{transmute(self.data)}),
                        _ => unreachable!(),

                    }
                )
            } else {None},
        }
    }

    // ColumnDataInFile to String (for RecordInFile)
    pub fn to_string(&self) -> String {
        unsafe {
            let index: [u8; 4] = transmute(self.index);
            let flags: [u8; 1] = transmute(self.flags);
            let data: [u8; 8] = transmute(self.data);
            format!("{}{}{}", String::from_utf8_unchecked(index.to_vec()), String::from_utf8_unchecked(flags.to_vec()), String::from_utf8_unchecked(data.to_vec()))
        }
    }
}


impl RecordInFile {
    // Record to RecordInFile
    pub fn from(th: &TableHandler, record: &Record) -> Self {
        RecordInFile {
            record: record.record.iter()
                    .map(|c| ColumnDataInFile::from(th, c).to_string())
                    .fold(String::new(), |s, v| s + &v)
        }
    }

    // RecordInFile to Record
    pub fn to_record(&self, th: &TableHandler) -> Record {
        let r: &[u8] = self.record.as_bytes();
        let size_of_data = size_of::<ColumnDataInFile>();
        assert_eq!(r.len() % size_of_data, 0);
        let mut result = Record{ record: vec![] };
        for offset in (0..r.len()).step_by(size_of_data) {
            result.record.push(ColumnDataInFile::new(&r[offset .. offset + size_of_data]).to_column_data(th));
        }
        result
    }
}

impl ColumnTypeInFile {
    // ColumnType to ColumnTypeInFile
    pub fn from(th: &TableHandler, ct: &ColumnType) -> Self {
        ColumnTypeInFile {
            name: th.insert_string(&ct.name).to_u64(),
            foreign_table_name: th.insert_string(&ct.foreign_table_name).to_u64(),
            index: ct.index,
            data_type: match ct.data_type {
                Type::Str(_, _) => 0,
                Type::Int(_) => 1,
                Type::Float(_) => 2,
                Type::Date(_) => 3,
            },
            data: match &ct.data_type {
                Type::Str(_, data) => match data {
                    Some(data) => th.insert_string(&data).to_u64(),
                    None => 0,
                },
                Type::Int(data) => unsafe{transmute(data.unwrap_or(0))},
                Type::Float(data) => unsafe{transmute(data.unwrap_or(0.0))},
                Type::Date(data) => unsafe{transmute(data.unwrap_or(0))},
            },
            flags: (ct.can_be_null as u8) |
                   (ct.has_index as u8) << 1 |
                   (ct.has_default as u8) << 2 |
                   (ct.is_primary as u8) << 3 |
                   (ct.is_foreign as u8) << 4 |
                   (ct.default_null as u8) << 5,
        }
    }

    // ColumnTypeInFile to ColumnType
    pub fn to_column_type(&self, th: &TableHandler) -> ColumnType {
        let has_default = self.flags & 4 > 0;
        ColumnType {
            name: th.get_string(&StrPointer::new(self.name)),
            foreign_table_name: th.get_string(&StrPointer::new(self.foreign_table_name)),
            index: self.index,
            data_type: match self.data_type {
                0 => Type::Str(0, if has_default {Some(th.get_string(&StrPointer::new(self.data)))} else {None}),
                1 => Type::Int(if has_default {Some(unsafe{transmute(self.data)})} else {None}),
                2 => Type::Float(if has_default {Some(unsafe{transmute(self.data)})} else {None}),
                3 => Type::Date(if has_default {Some(unsafe{transmute(self.data)})} else {None}),
                _ => unreachable!(),
            },
            can_be_null: self.flags & 1 > 0,
            has_index: self.flags & 2 > 0,
            has_default: self.flags & 4 > 0,
            is_primary: self.flags & 8 > 0,
            is_foreign: self.flags & 16 > 0,
            default_null: self.flags & 32 > 0,
        }
    }
}