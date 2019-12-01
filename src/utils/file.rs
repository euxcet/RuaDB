use std::fs;
use std::path::PathBuf;

pub fn get_database_list(path: &str) -> Vec<String> {
    let files = fs::read_dir(path).unwrap();

    let mut result = Vec::new();
    
    for file in files {
        let fname = file.unwrap().path().into_os_string().into_string();
        let fname = &fname.unwrap()[path.len()..];
        result.push(fname.to_string());
    }
    result
}

pub fn get_table_list(path: &str) -> Vec<String> {
    get_database_list(path)
}

pub fn create_table(path: String) -> std::io::Result<()> {
    fs::create_dir(path)?;
    Ok(())
}

pub fn drop_table(path: String) -> std::io::Result<()> {
    fs::remove_dir(path)?;
    Ok(())
}

