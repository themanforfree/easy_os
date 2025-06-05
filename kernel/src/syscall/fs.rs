//! File and filesystem-related syscalls

use log::trace;

use crate::{
    fs::{OpenFlags, open_file},
    memory::VirtAddr,
    proc::current_proc,
};

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("sys_write: fd = {fd}, buf = {buf:p}, len = {len}");
    let proc = current_proc();
    let pt = proc.page_table();
    let inner = proc.borrow_inner_mut();

    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.writable() {
            return -1;
        }
        drop(inner);
        file.write(pt.translate_bytes_buffer(VirtAddr::new(buf as usize), len)) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("sys_read: fd = {fd}, buf = {buf:p}, len = {len}");
    let proc = current_proc();
    let pt = proc.page_table();
    let inner = proc.borrow_inner_mut();

    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        drop(inner);
        file.read(pt.translate_bytes_buffer(VirtAddr::new(buf as usize), len)) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let proc = current_proc();
    let pt = proc.page_table();
    let path = pt.read_c_str(path).unwrap();
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = proc.borrow_inner_mut();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let proc = current_proc();
    let mut inner = proc.borrow_inner_mut();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}
