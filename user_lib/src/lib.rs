#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_utils::test_runner)]
#![feature(linkage)]

#[macro_use]
pub mod console;
mod common;
mod syscall;
pub mod test_utils;

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

pub fn fork() -> isize {
    syscall::sys_fork()
}

pub fn exec(_path: &str) -> isize {
    todo!()
}

pub fn yield_() -> isize {
    syscall::sys_yield()
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match syscall::sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match syscall::sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
