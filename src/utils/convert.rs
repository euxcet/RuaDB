use std::mem::transmute;
use std::convert::TryInto;

// TODO: optimize

/*
pub unsafe fn vec_u64_to_vec_u8(data: &Vec<u64>) -> Vec<u8> {
    data.iter()
        .map(|x| transmute::<u64, [u8; 8]>(*x))
        .fold(Vec::new(), |s, v| s.extend(&mut v))
}
*/

pub unsafe fn u64_to_vec_u8(data: u64) -> Vec<u8> {
    transmute::<u64, [u8; 8]>(data).to_vec()
}

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

pub unsafe fn vec_u64_to_string_len(data: &Vec<u64>, len: usize) -> String {
    assert!(data.len() <= len);
    data.iter()
        .map(|x| transmute::<u64, [u8; 8]>(*x))
        .fold(String::new(), |s, v| s + &String::from_utf8_unchecked(v.to_vec()))
        + &String::from_utf8_unchecked(vec![0u8; (len - data.len()) * 8])
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
        let val = transmute::<[u8; 8], u64>(bytes[i .. i + 8].try_into().expect("slice with incorrect length"));
        res.push(val);
    }
    res
}

pub unsafe fn string_to_vec_u64_with_break(data: &String) -> Vec<u64> {
    let bytes = data.as_bytes();
    let mut res = Vec::new();
    for i in (0..bytes.len()).step_by(8) {
        let val = transmute::<[u8; 8], u64>(bytes[i .. i + 8].try_into().expect("slice with incorrect length"));
        if val == 0 {
            break;
        }
        else {
            res.push(val);
        }
    }
    res
}

pub fn str_to_date(s: &str) -> u64 {
    let date_vec: Vec<&str> = s.split("-").collect();
    if date_vec.len() != 3 {
        0
    }
    else {
        let year = u64::from_str_radix(date_vec[0], 10);
        let month = u64::from_str_radix(date_vec[1], 10);
        let day = u64::from_str_radix(date_vec[2], 10);
        if year.is_err() || month.is_err() || day.is_err() {
            0
        }
        else {
            let year = year.unwrap();
            let month = month.unwrap();
            let day = day.unwrap();
            if year < 1000 || year > 9999 || month > 12 || day > 31 {
                0
            }
            else {
                (year << 9) | (month << 5) | day
            }
        }
    }
}

pub fn date_to_str(date: u64) -> String {
    format!("{}-{:02}-{:02}", date >> 9, date >> 5 & 0b1111, date & 0b11111)
}

pub fn str_to_numeric(s: &str, precision: u8) -> i64 {
    assert!(precision > 0);
    let precision = precision - 1;
    let num_vec: Vec<&str> = s.split(".").collect();
    if num_vec.len() != 2 {
        0
    }
    else {
        let integer = num_vec[0];
        let decimal = num_vec[1];
        if decimal.len() > precision as usize {
            0
        }
        else {
            i64::from_str_radix(
                &format!(
                    "{}{}{}",
                    integer,
                    decimal,
                    String::from_utf8(vec![48; precision as usize - decimal.len()]).unwrap()
                ),
                10
            ).unwrap()
        }
    }
}

pub fn numeric_to_str(num: i64, precision: u8) -> String {
    assert!(precision > 0);
    let precision = precision - 1;
    let mut integer = num;
    let mut exp10 = 1;
    for _ in 0..precision {
        integer /= 10;
        exp10 *= 10;
    }
    let decimal = num - integer * exp10;
    if precision == 0 {
        format!("{}", integer)
    }
    else {
        format!("{}.{}", integer, decimal)
    }
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