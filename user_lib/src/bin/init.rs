#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Hello, world!");
    0
}
