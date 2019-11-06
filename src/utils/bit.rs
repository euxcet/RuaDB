pub fn get_free_index(slot: u32) -> u32 {
    assert_ne!(slot, u32::max_value());
    (slot + 1).trailing_zeros()
}

pub fn set_used(slot: &mut u32, index: u32) {
    *slot |= 1 << index;
}

pub fn set_free(slot: &mut u32, index: u32) {
    *slot |= 1 << index;
    *slot ^= 1 << index;
}

pub fn all_used(slot: u32, len: usize) -> bool {
    free_num(slot, len) == 0
}

pub fn all_free(slot: u32, len: usize) -> bool {
    free_num(slot, len) == len as u32
}

pub fn free_num(slot: u32, len: usize) -> u32 {
    (slot << (32 - len)).count_zeros() + len as u32 - 32
}

pub fn used_num(slot: u32, len: usize) -> u32 {
    (slot << (32 - len)).count_ones()
}

pub fn is_used(slot: u32, index: usize) -> bool {
    (slot & (1 << index)) != 0
}

pub fn is_free(slot: u32, index: usize) -> bool {
    (slot & (1 << index)) == 0
}

pub fn is_one(slot: u32, index: usize) -> bool {
    (slot & (1 << index)) != 0
}

pub fn is_zero(slot: u32, index: usize) -> bool {
    (slot & (1 << index)) != 0
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bit() {
        assert_eq!(free_num(0x0fff0ff1, 30), 9);
        assert_eq!(free_num(0x0fff0ff1, 32), 11);

        assert_eq!(free_num(0x0fff0ff1, 1), 0);
        assert_eq!(free_num(0x0fff0ff1, 10), 3);

        assert!(all_free(0x0fff0ff2, 1));
        assert!(!all_free(0x0fff0ff2, 2));

        assert!(is_used(0x0fff0ff2, 5));
        assert!(is_free(0x0fff0ff2, 31));

        assert!(all_used(0x3fffffff, 30));

        let mut a = 0x00ff0f11;
        set_free(&mut a, 0);
        assert_eq!(a, 0x00ff0f10);
        set_free(&mut a, 0);
        assert_eq!(a, 0x00ff0f10);
        set_used(&mut a, 3);
        assert_eq!(a, 0x00ff0f18);
    }


    #[test]
    #[warn(exceeding_bitshifts)]
    fn bit_op_test() {
        assert_eq!(get_free_index(0xFFu32), 8);
        assert!(all_used(0xFFFFFFFFu32, 30));
        assert!(!all_used(0xFFFFFF8Fu32, 30));
        let mut a = 0x1u32;
        set_used(&mut a, 1);
        assert_eq!(a, 0x3u32);
    }
}