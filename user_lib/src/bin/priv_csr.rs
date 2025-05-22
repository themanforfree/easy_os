#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

#[macro_use]
extern crate user_lib;

use riscv::register::sstatus::{self, SPP};

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        sstatus::set_spp(SPP::User);
    }
    0
}
