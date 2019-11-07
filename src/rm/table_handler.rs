use super::file_handler::*;
use super::record::*;
use super::in_file::*;
use super::pagedef::*;
use std::fmt;

pub struct TableHandler {
    // TODO: support multiple filehandlers
    fh: FileHandler,
}

impl fmt::Debug for TableHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TableHandler")
    }
}

impl TableHandler {
    pub fn new(fh: FileHandler) -> Self {
        TableHandler {
            fh: fh,
        }
    }

    pub fn close(&self) {
        self.fh.close();
    }

    pub fn delete(&self, ptr: &mut StrPointer) {
        self.fh.delete(ptr);
    }

    // for String
    pub fn insert_string(&self, s: &String) -> StrPointer {
        self.fh.insert::<String, u32>(&s)
    }

    pub fn get_string(&self, ptr: &StrPointer) -> String {
        self.fh.get::<String, u32>(ptr)
    }

    pub fn update_string(&self, ptr: &mut StrPointer, s: &String) {
        self.fh.update::<String, u32>(ptr, &s);
    }

    // for Record
    pub fn insert_record(&self, record: &Record) -> StrPointer {
        self.fh.insert::<RecordInFile, u32>(&RecordInFile::from(self, record))
    }

    pub fn get_record(&self, ptr: &StrPointer) -> Record {
        self.fh.get::<RecordInFile, u32>(ptr).to_record(self)
    }

    pub fn update_record(&self, ptr: &mut StrPointer, record: &Record) {
        self.fh.update::<RecordInFile, u32>(ptr, &RecordInFile::from(self, record));
    }

    // for ColumnType
    pub fn insert_column_type(&self, ct: &ColumnType) -> StrPointer {
        self.fh.insert::<ColumnTypeInFile, u32>(&ColumnTypeInFile::from(self, ct))
    }

    pub fn get_column_type(&self, ptr: &StrPointer) -> ColumnType {
        self.fh.get::<ColumnTypeInFile, u32>(ptr).to_column_type(self)
    }

    pub fn update_column_type(&self, ptr: &mut StrPointer, ct: &ColumnType) {
        self.fh.update::<ColumnTypeInFile, u32>(ptr, &ColumnTypeInFile::from(self, ct))
    }
}