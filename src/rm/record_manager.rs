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

// #[test]
fn file() {
    let mut r = RecordManager::new();
    let mut fh1 = r.open("d:/Rua/test/a.rua");
    let mut fh2 = r.open("d:/Rua/test/b.rua");

    let id = ColumnType {
        is_primary: true,
        name: String::from("id"),
        has_index: true,
        .. Default::default()
    };

    let name = ColumnType {
        name: String::from("name"),
        has_default: true,
        data_type: Type::Str(100, Some(String::from("lyt"))),
        .. Default::default()
    };

    let columns = vec![id, name];
    fh1.set_column(&columns);
    fh2.set_column(&columns);

    fh1.close();
    fh2.close();
}

