use std::iter;
use rand::prelude::*;
use rand::{Rng, thread_rng};
use rand::distributions::{Standard, Alphanumeric};
use rand::SeedableRng;

pub struct Generator {
    threadRng: ThreadRng,
    stdRng: StdRng,
    seedable: bool,
}


impl Generator {
    pub fn new(seedable: bool) -> Self {
        Generator {
            threadRng: thread_rng(),
            stdRng: SeedableRng::from_seed([1; 32]),
            seedable: seedable,
        }
    }

    pub fn gen<T>(&mut self) -> T
        where Standard: Distribution<T> {
        if self.seedable {
            self.stdRng.gen::<T>()
        }
        else {
            self.threadRng.gen::<T>()
        }
    }

    pub fn gen_string(&mut self, len: usize) -> String {
        if self.seedable {
            iter::repeat(())
            .map(|()| self.stdRng.sample(Alphanumeric))
            .take(len)
            .collect()
        }
        else {
            iter::repeat(())
            .map(|()| self.threadRng.sample(Alphanumeric))
            .take(len)
            .collect()
        }

    }

    pub fn gen_string_s(&mut self, len: usize) -> String {
        if self.seedable {
            let l: usize = self.stdRng.gen_range(0, len);
            iter::repeat(())
            .map(|()| self.stdRng.sample(Alphanumeric))
            .take(l)
            .collect()
        }
        else {
            let l: usize = self.stdRng.gen_range(0, len);
            iter::repeat(())
            .map(|()| self.threadRng.sample(Alphanumeric))
            .take(l)
            .collect()
        }
    }
}