pub mod event;
use std::{
    cmp::min,
    ops::{Add, Sub},
};

pub use event::{Event, Events};

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct Scroll {
    v: u64,
    max: u64,
}

impl Scroll {
    pub fn new(max: u64) -> Self {
        Self { v: 0, max }
    }

    pub fn val(&self) -> u64 {
        self.v
    }

    pub fn inc(&mut self) {
        self.v = min(self.v + 1, self.max);
    }

    pub fn dec(&mut self) {
        self.v = match self.v {
            0 => 0,
            x => x - 1,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModInt {
    p: u64,
    value: u64,
}
impl ModInt {
    pub fn new(p: u64, value: u64) -> Self {
        Self { p, value }
    }

    pub fn val(&self) -> u64 {
        self.value
    }

    pub fn inc(&mut self) {
        *self = self.add(ModInt::new(self.p, 1));
    }

    pub fn dec(&mut self) {
        *self = self.sub(ModInt::new(self.p, 1));
    }
}

impl Add for ModInt {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.value += rhs.value;
        self.value %= self.p;
        self
    }
}

impl Sub for ModInt {
    type Output = Self;
    fn sub(self, mut rhs: Self) -> Self::Output {
        let n = self.p - rhs.value;
        rhs.value = n;
        self.add(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll() {
        let mut scroll = Scroll::new(7);
        scroll.dec();
        assert_eq!(scroll.val(), 7);
        scroll.inc();
        assert_eq!(scroll.val(), 0);
    }

    #[test]
    fn test_add() {
        let tests: [(u64, (u64, u64), u64); 2] = [(7, (3, 4), 0), (7, (4, 5), 2)];

        for (p, (lhs, rhs), expected) in tests.iter() {
            let p = *p;
            let lhs = *lhs;
            let rhs = *rhs;

            let lhs = ModInt::new(p, lhs);
            let rhs = ModInt::new(p, rhs);

            assert_eq!((lhs + rhs).val(), *expected);
        }
    }

    #[test]
    fn test_sub() {
        let tests: [(u64, (u64, u64), u64); 2] = [(7, (3, 4), 6), (7, (5, 4), 1)];

        for (p, (lhs, rhs), expected) in tests.iter() {
            let p = *p;
            let lhs = *lhs;
            let rhs = *rhs;

            let lhs = ModInt::new(p, lhs);
            let rhs = ModInt::new(p, rhs);

            assert_eq!((lhs - rhs).val(), *expected);
        }
    }
}
