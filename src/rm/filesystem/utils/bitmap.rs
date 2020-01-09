const LEAF_BIT: i32 = 32;
const MAX_INNER_NUM: usize = 67;

const BIAS: u32 = 5;

// pub const DE_BRUIJN: [i32; 32] = [0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9];

pub struct Bitmap {
    data: Vec<u32>,
    size: i32,
    root_bit: i32,
    root_level: i32,
    root_index: i32,
    inner: [u32; MAX_INNER_NUM],
    // inner_mask: u32,
    // root_mask: u32,
}


impl Bitmap {
    // fn get_mask(k: i32) -> u32 {
    //     (1 << k) - 1
    // }

    fn get_leaf_data(&self, index: i32) -> u32 {
        self.data[index as usize]
    }

    fn set_leaf_data(&mut self, index: i32, v: u32) {
        self.data[index as usize] = v;
    }
    fn set_leaf_bit(&mut self, index: i32, k: u32) -> i32 {
        let (pos, bit) = Self::get_pos(index);
        let mut umask = 1 << bit;
        let mask = !umask;
        if k == 0 {
            umask = 0;
        }
        let w = (self.get_leaf_data(pos) & mask) | umask;
        self.set_leaf_data(pos, w);

        pos
    }
    fn child_word(&self, start: i32, bit_num: i32, i: i32, j: i32) -> u32 {
        let index = (i << BIAS) + j;

        match start {
            0 => self.get_leaf_data(index),
            _ => self.inner[(start - bit_num + index) as usize],
        }
    }

    fn init(&mut self) {
        self.root_level = 0;
        let mut s = self.size;
        self.root_index = 0;
        while s > LEAF_BIT {
            let word_num = s >> BIAS;
            for i in 0..word_num {
                let mut w: u32 = 0;
                for j in 0..LEAF_BIT {
                    let c = self.child_word(self.root_index, s, i, j);
                    if c != 0 {
                        w += 1 << j;
                    }
                }
                self.inner[(self.root_index + i) as usize] = w;
            }
            self.root_level += 1;
            self.root_index += word_num;
            s = word_num;
        }
        self.root_bit = s;
        let i = 0;
        let mut w: u32 = 0;
        for j in 0..self.root_bit {
            let c = self.child_word(self.root_index, s, i, j);
            if c != 0 {
                w += 1 << j;
            }
        }
        self.inner[self.root_index as usize] = w;
    }

    fn _set_bit(start: &mut [u32], index: i32, k: u32) -> i32 {
        let (pos, bit) = Self::get_pos(index);
        let mut umask = 1 << bit;
        let mask = !umask;
        if k == 0 {
            umask = 0;
        }
        let p = pos as usize;
        start[p] = (start[p] & mask) | umask;

        pos
    }

    fn update_inner(&mut self, level: i32, offset: i32, index: i32, level_cap: i32, k: u32) {
        let off = offset as usize;
        let start = &mut(self.inner[off.. ]);
        let pos = Self::_set_bit(start, index, k);
        if level == self.root_level {
            return;
        }
        let c = match start[pos as usize] {
            0 => 0, 
            _ => 1,
        };
        self.update_inner(level + 1, offset + level_cap, pos, level_cap >> BIAS, c);
    }

    fn _find_left_one(&self, level: i32, offset: i32, pos: i32, prev_level_cap: i32) -> i32 {
        let lb = Self::lowbit(self.inner[(offset + pos) as usize] as i32);
        let index = Self::get_index(lb);
        let npos = (pos << BIAS) + index;

        match level {
            0 => npos,
            _ => self._find_left_one(level - 1, offset - prev_level_cap, npos, prev_level_cap << BIAS),
        }

    }


    fn get_index(k: i32) -> i32 {
        if k <= 0 {
            panic!("log error");
        }
        k.trailing_zeros() as i32
    }

    fn lowbit(k: i32) -> i32 {
        k & (-k)
    }

    fn get_pos(index: i32) -> (i32, i32) {
        let pos = index >> BIAS;
        let bit = index - (pos << BIAS);
        (pos, bit)
    }

    pub fn set_bit(&mut self, index: i32, k: u32) {
        let p = self.set_leaf_bit(index, k);
        let mut c: u32 = 1;
        if self.get_leaf_data(p) == 0 {
            c = 0;
        }
        self.update_inner(0, 0, p as i32, self.size >> BIAS, c);
    }

    pub fn find_left_one(&self) -> i32 {
        let i = self._find_left_one(self.root_level, self.root_index, 0, self.root_bit);
        let lb = Self::lowbit(self.get_leaf_data(i) as i32);
        (i << BIAS) + Self::get_index(lb)
    }

    pub fn new(cap: usize, k: u32) -> Self {
        let s = cap >> BIAS;
        let mut m = Self {
            size: s as i32,
            data: vec![ 
                match k {
                    1 => 0xffffffff,
                    _ => 0,
                }; 
                s], 
            inner: [0; MAX_INNER_NUM],
            root_bit: 0,
            root_level: 0,
            root_index: 0,
        };
        m.init();
        m
    }
}