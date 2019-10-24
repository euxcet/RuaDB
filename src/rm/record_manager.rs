use std::cell::RefCell;
use std::rc::Rc;

use super::file_handler::FileHandler;
use super::filesystem::bufmanager::buf_page_manager::BufPageManager;

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
        self.bpm.borrow_mut().file_manager.create_file(path);
    }

    pub fn delete(&mut self, path: &str) {
        self.bpm.borrow_mut().file_manager.delete_file(path);
    }

    pub fn open(&mut self, path: &str) -> FileHandler {
        let fd = self.bpm.borrow_mut().file_manager.open_file(path);
        
        FileHandler::new(fd, self.bpm.clone())
    }

}

