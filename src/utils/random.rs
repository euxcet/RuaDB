use std::iter;
use rand::prelude::*;
use rand::{Rng, thread_rng};
use rand::distributions::{Standard, Alphanumeric};
use rand::distributions::uniform::{SampleUniform, SampleBorrow};
use rand::SeedableRng;

pub struct Generator {
    thread_rng: ThreadRng,
    std_rng: StdRng,
    seedable: bool,
}


impl Generator {
    pub fn new(seedable: bool) -> Self {
        Generator {
            thread_rng: thread_rng(),
            std_rng: SeedableRng::from_seed([3; 32]),
            seedable: seedable,
        }
    }

    pub fn gen<T>(&mut self) -> T
        where Standard: Distribution<T> {
        if self.seedable {
            self.std_rng.gen::<T>()
        }
        else {
            self.thread_rng.gen::<T>()
        }
    }

    pub fn gen_range<T: SampleUniform, B1, B2>(&mut self, low: B1, high: B2) -> T
        where
            B1: SampleBorrow<T> + Sized,
            B2: SampleBorrow<T> + Sized, {
        if self.seedable {
            self.std_rng.gen_range::<T, B1, B2>(low, high)
        }
        else {
            self.thread_rng.gen_range::<T, B1, B2>(low, high)
        }
    }

    pub fn gen_string(&mut self, len: usize) -> String {
        if self.seedable {
            iter::repeat(())
            .map(|()| self.std_rng.sample(Alphanumeric))
            .take(len)
            .collect()
        }
        else {
            iter::repeat(())
            .map(|()| self.thread_rng.sample(Alphanumeric))
            .take(len)
            .collect()
        }

    }

    pub fn gen_string_s(&mut self, len: usize) -> String {
        if self.seedable {
            let l: usize = self.std_rng.gen_range(0, len);
            iter::repeat(())
            .map(|()| self.std_rng.sample(Alphanumeric))
            .take(l)
            .collect()
        }
        else {
            let l: usize = self.std_rng.gen_range(0, len);
            iter::repeat(())
            .map(|()| self.thread_rng.sample(Alphanumeric))
            .take(l)
            .collect()
        }
    }
}
