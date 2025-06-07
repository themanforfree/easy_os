#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_utils::test_runner)]
#![feature(linkage)]

extern crate alloc;

use core::ptr::addr_of_mut;

use alloc::vec::Vec;
use bitflags::bitflags;
use buddy_system_allocator::LockedHeap;

#[macro_use]
pub mod console;
mod common;
mod syscall;
pub mod test_utils;

const USER_HEAP_SIZE: usize = 4096 * 4;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    common::clear_bss();
    unsafe {
        HEAP.lock()
            .init(addr_of_mut!(HEAP_SPACE) as usize, USER_HEAP_SIZE);
    }
    let mut args = Vec::new();
    let argv_ptr = argv as *const *const u8;
    for i in 0..argc {
        let arg_ptr = unsafe { *argv_ptr.add(i) };
        if !arg_ptr.is_null() {
            let arg = unsafe { core::ffi::CStr::from_ptr(arg_ptr) };
            args.push(arg.to_str().unwrap_or(""));
        }
    }
    exit(main(argc, args.as_slice()));
    unreachable!()
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    unreachable!("main() should be defined in user program");
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall::sys_read(fd, buffer)
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

pub fn exec(path: &str, args: &[*const u8]) -> isize {
    syscall::sys_exec(path, args)
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

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    syscall::sys_open(path, flags.bits())
}

pub fn close(fd: usize) -> isize {
    syscall::sys_close(fd)
}

pub fn pipe(pipe: &mut [usize]) -> isize {
    syscall::sys_pipe(pipe)
}
