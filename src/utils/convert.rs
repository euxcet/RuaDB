use std::mem::transmute;
use std::convert::TryInto;

// TODO: optimize
pub unsafe fn vec_u8_to_string(data: &Vec<u8>) -> String {
    String::from_utf8_unchecked(data.to_vec())
}

pub unsafe fn vec_u16_to_string(data: &Vec<u16>) -> String {
    data.iter()
        .map(|x| transmute::<u16, [u8; 2]>(*x))
        .fold(String::new(), |s, v| s + &String::from_utf8_unchecked(v.to_vec()))
}

pub unsafe fn vec_u32_to_string(data: &Vec<u32>) -> String {
    data.iter()
        .map(|x| transmute::<u32, [u8; 4]>(*x))
        .fold(String::new(), |s, v| s + &String::from_utf8_unchecked(v.to_vec()))
}

pub unsafe fn vec_u64_to_string(data: &Vec<u64>) -> String {
    data.iter()
        .map(|x| transmute::<u64, [u8; 8]>(*x))
        .fold(String::new(), |s, v| s + &String::from_utf8_unchecked(v.to_vec()))
}

pub fn string_to_vec_u8(data: &String) -> Vec<u8> {
    data.as_bytes().to_vec()
}

pub unsafe fn string_to_vec_u16(data: &String) -> Vec<u16> {
    let bytes = data.as_bytes();
    let mut res = Vec::new();
    for i in (0..bytes.len()).step_by(2) {
        res.push(transmute::<[u8; 2], u16>(bytes[i .. i + 2].try_into().expect("slice with incorrect length")));
    }
    res
}

pub unsafe fn string_to_vec_u32(data: &String) -> Vec<u32> {
    let bytes = data.as_bytes();
    let mut res = Vec::new();
    for i in (0..bytes.len()).step_by(4) {
        res.push(transmute::<[u8; 4], u32>(bytes[i .. i + 4].try_into().expect("slice with incorrect length")));
    }
    res
}

pub unsafe fn string_to_vec_u64(data: &String) -> Vec<u64> {
    let bytes = data.as_bytes();
    let mut res = Vec::new();
    for i in (0..bytes.len()).step_by(8) {
        res.push(transmute::<[u8; 8], u64>(bytes[i .. i + 8].try_into().expect("slice with incorrect length")));
    }
    res
}

#[test]
fn convert() {
    unsafe {
        assert_eq!(string_to_vec_u8(&vec_u8_to_string(&vec![1, 2, 3])), vec![1, 2, 3]);
        assert_eq!(string_to_vec_u16(&vec_u16_to_string(&vec![1, 2, 3])), vec![1, 2, 3]);
        assert_eq!(string_to_vec_u32(&vec_u32_to_string(&vec![1, 2, 3])), vec![1, 2, 3]);
        assert_eq!(string_to_vec_u64(&vec_u64_to_string(&vec![1, 2, 3])), vec![1, 2, 3]);
    }
}