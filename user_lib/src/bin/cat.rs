#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

use user_lib::{OpenFlags, open, read};

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main(_argc: usize, argv: &[&str]) -> i32 {
    assert_eq!(argv.len(), 2);
    let fd = open(argv[1], OpenFlags::RDONLY);
    if fd == -1 {
        println!("cat: {}: No such file or directory", argv[1]);
        return -1;
    }
    let mut buffer = [0u8; 64];
    loop {
        let n: usize = read(fd as usize, &mut buffer) as usize;
        if n == 0 {
            break;
        }
        print!("{}", core::str::from_utf8(&buffer[..n]).unwrap());
    }
    println!();
    0
}
