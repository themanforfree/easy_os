use crate::memory::UserBuffer;

mod inode;
mod stdio;

pub use self::inode::{OpenFlags, list_apps, open_file};
pub use self::stdio::{Stdin, Stdout};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
