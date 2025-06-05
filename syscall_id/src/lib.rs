#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

pub const SYSCALL_OPEN: usize = 56;
pub const SYSCALL_CLOSE: usize = 57;
pub const SYSCALL_READ: usize = 63;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_FORK: usize = 220;
pub const SYSCALL_EXEC: usize = 221;
pub const SYSCALL_WAITPID: usize = 260;

#[cfg(test)]
fn test_runner(_tests: &[&dyn Fn()]) {
    unreachable!("this function will never be called");
}
