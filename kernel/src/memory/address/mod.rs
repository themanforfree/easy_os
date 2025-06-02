//! Use SV39 architecture address space

mod phys;
mod virt;

pub use phys::{PhysAddr, PhysPageNum};
pub use virt::{VirtAddr, VirtPageNum};

macro_rules! add_impl {
    ($($t:ty)*) => ($(
        impl core::ops::Add<usize> for $t {
            type Output = $t;
            fn add(self, rhs: usize) -> $t { Self(self.0 + rhs) }
        }
    )*)
}

macro_rules! sub_impl {
    ($($t:ty)*) => ($(
        impl core::ops::Sub<usize> for $t {
            type Output = $t;
            fn sub(self, rhs: usize) -> $t { Self(self.0 - rhs) }
        }

        impl core::ops::Sub<$t> for $t {
            type Output = usize;
            fn sub(self, rhs: $t) -> usize { self.0 - rhs.0 }
        }
    )*)
}

macro_rules! add_assign_impl {
    ($($t:ty)*) => ($(
        impl core::ops::AddAssign<usize> for $t {
            fn add_assign(&mut self, rhs: usize) {
                self.0 += rhs;
            }
        }
    )*)
}

macro_rules! sub_assign_impl {
    ($($t:ty)*) => ($(
        impl core::ops::SubAssign<usize> for $t {
            fn sub_assign(&mut self, rhs: usize) {
                self.0 -= rhs;
            }
        }
    )*)
}

macro_rules! from_impl {
    ($($t:ty)*) => ($(
        impl From<$t> for usize {
            fn from(addr: $t) -> Self {
                addr.0
            }
        }

        impl From<usize> for $t {
            fn from(addr: usize) -> Self {
                Self::new(addr)
            }
        }
    )*)
}

macro_rules! all_impl {
    ($($t:ty)*) => {
        add_impl!($($t)*);
        sub_impl!($($t)*);
        add_assign_impl!($($t)*);
        sub_assign_impl!($($t)*);
        from_impl!($($t)*);
    };
}

all_impl!(PhysAddr PhysPageNum VirtAddr VirtPageNum);
