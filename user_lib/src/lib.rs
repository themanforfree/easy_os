#![no_std]
#![feature(linkage)]

#[macro_use]
pub mod console;
mod common;
mod syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    common::clear_bss();
    exit(main());
    unreachable!()
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    unreachable!("main() should be defined in user program");
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    syscall::sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> isize {
    syscall::sys_exit(exit_code)
}
