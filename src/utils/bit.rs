pub fn get_free_index(slot: u32) -> u32 {
    (slot + 1).trailing_zeros()
}

pub fn set_used(slot: &mut u32, index: u32) {
    *slot |= 1 << index;
}

pub fn all_used(slot: u32, len: usize) -> bool {
    slot & ((1 << len) - 1) == ((1 << len) - 1)
}