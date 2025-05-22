#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

#[macro_use]
extern crate user_lib;

fn fib(n: usize) -> usize {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

#[unsafe(no_mangle)]
fn main() -> i32 {
    let n = 10;
    let result = fib(n);
    println!("Fibonacci of {} is {}", n, result);
    0
}
