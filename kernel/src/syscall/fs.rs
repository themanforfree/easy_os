//! File and filesystem-related syscalls

use log::trace;

use crate::{
    memory::{PageTable, VirtAddr},
    proc::current_token,
    sbi::read_chars_blocking,
};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("sys_write: fd = {fd}, buf = {buf:p}, len = {len}");
    match fd {
        FD_STDOUT => {
            let token = current_token();
            let pt = PageTable::from_token(token);
            let slice = pt.copy_in(VirtAddr::new(buf as usize), len);
            let str = core::str::from_utf8(&slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("sys_read: fd = {fd}, buf = {buf:p}, len = {len}");
    match fd {
        FD_STDIN => {
            let kernel_buffer = read_chars_blocking(len);

            let token = current_token();
            let pt = PageTable::from_token(token);
            pt.copy_out(VirtAddr::new(buf as usize), &kernel_buffer);

            0
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
