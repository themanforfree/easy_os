#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main(_argc: usize, argv: &[&str]) -> i32 {
    for arg in argv.iter().skip(1) {
        print!("{} ", arg);
    }
    println!();
    0
}
