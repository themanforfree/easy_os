#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

pub mod syscall_id;

#[cfg(test)]
fn test_runner(_tests: &[&dyn Fn()]) {
    unreachable!("this function will never be called");
}
