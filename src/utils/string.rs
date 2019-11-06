use std::cmp;

pub fn copy_bytes_check(to: &mut [u8], from: &str, len: usize) {
    assert!(to.len() >= len);
    assert!(from.len() >= len);
    to[..len].copy_from_slice(from[..len].as_bytes());
}

pub fn copy_bytes(to: &mut [u8], from: &str) {
    let len = cmp::min(to.len(), from.len());
    copy_bytes_check(to, from, len);
}

pub fn copy_bytes_u8(to: &mut [u8], from: &[u8]) {
    let len = cmp::min(to.len(), from.len());
    assert!(to.len() >= len);
    assert!(from.len() >= len);
    to[..len].copy_from_slice(&from[..len]);
}

pub fn from_bytes(from: &[u8], len: usize) -> String {
    std::str::from_utf8(&from[..len]).unwrap().to_owned()
}