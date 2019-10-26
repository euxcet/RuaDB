use std::io;
use std::io::prelude::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::SeekFrom;

use super::super::pagedef::*;
use super::super::super::pagedef::*;
use super::super::utils::bitmap::*;

pub struct FileManager {
    fd: Vec<Option<String>>,
    fm: Box<Bitmap>,
    tm: Box<Bitmap>,
}

impl FileManager {
    fn _create_file(&self, name: &str) -> io::Result<()> {
        let mut f = OpenOptions::new().create(true).append(true).open(name)?;

        Ok(())
    }

    fn _open_file(&mut self, name: &str, file_id: i32) -> io::Result<()> {
        let mut f = OpenOptions::new().create(true).read(true).write(true).open(name)?;
        self.fd[file_id as usize] = Some(name.to_owned());

        Ok(())
    }

    fn _delete_file(&self, name: &str) -> io::Result<()> {
        fs::remove_file(name)
    }


    pub fn new() -> Self {
        Self {
            fd: vec![None; MAX_FILE_NUM],
            fm: Box::new(Bitmap::new(MAX_FILE_NUM, 1)),
            tm: Box::new(Bitmap::new(MAX_TYPE_NUM, 1)),
        }
    }

    pub fn write_page(&self, file_id: i32, page_id: i32, buf: &[u8], off: i32) -> io::Result<()> {
        let fname = &self.fd[file_id as usize].as_ref().unwrap();
        let offset = page_id << PAGE_SIZE_IDX;

        let mut f = OpenOptions::new().read(true).write(true).open(fname)?;
        f.seek(SeekFrom::Start(offset as u64))?;
        let r = f.write(&buf[(off as usize) .. (off as usize) + PAGE_SIZE])?;

        Ok(())
    }

    pub fn read_page(&self, file_id: i32, page_id: i32, buf: &mut [u8], off: i32) -> io::Result<()>{
        let fname = &self.fd[file_id as usize].as_ref().unwrap();
        let offset = page_id << PAGE_SIZE_IDX;

        let mut f = OpenOptions::new().read(true).open(fname)?;
        f.seek(SeekFrom::Start(offset as u64))?;
        f.read(&mut buf[(off as usize) .. (off as usize) + PAGE_SIZE])?;

        Ok(())
    }

    pub fn close_file(&mut self, file_id: i32) -> io::Result<()> {
        self.fm.set_bit(file_id, 1);
        Ok(())
    }
    
    pub fn create_file(&self, name: &str) -> io::Result<()> {
        self._create_file(name)
    }

    pub fn delete_file(&self, name: &str) -> io::Result<()> {
        self._delete_file(name)
    }

    pub fn open_file(&mut self, name: &str) -> i32 {
        let file_id = self.fm.find_left_one();
        self.fm.set_bit(file_id, 0);
        assert!(self._open_file(name, file_id).is_ok());
        

        file_id
    }
    
    pub fn new_type(&mut self) -> i32 {
        let t = self.tm.find_left_one();
        self.tm.set_bit(t, 0);

        t
    }

    pub fn close_type(&mut self, type_id: i32) {
        self.tm.set_bit(type_id, 1);
    }
}
