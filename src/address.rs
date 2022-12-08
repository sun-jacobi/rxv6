#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct Addr(pub(crate) usize);

use core::convert::From;
use core::convert::Into;
use core::ops::{Add, Sub};

impl Add<usize> for Addr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Addr(self.0 + rhs)
    }
}

impl From<Addr> for usize {
    fn from(addr: Addr) -> Self {
        addr.0
    }
}

impl Sub<usize> for Addr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        Addr(self.0 - rhs)
    }
}

impl From<usize> for Addr {
    fn from(num: usize) -> Self {
        Self(num)
    }
}

impl Addr {
    pub(crate) fn round_up_pg(self) -> Self {
        Self((self.0 + 4095) & (!4096))
    }

    pub(crate) fn round_down_pg(self) -> Self {
        Self((self.0) & (!4096))
    }
}
