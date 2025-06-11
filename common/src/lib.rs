#![cfg_attr(not(unix), no_std)]
#![cfg_attr(not(unix), feature(custom_test_frameworks))]
#![cfg_attr(not(unix), test_runner(test_runner))]

pub mod sig;
pub mod syscall_id;

#[cfg(all(not(unix), test))]
fn test_runner(_tests: &[&dyn Fn()]) {
    unreachable!("this function will never be called");
}
