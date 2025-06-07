#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

use alloc::{format, string::String, vec::Vec};
use user_lib::{console::getchar, exec, fork, waitpid};

#[macro_use]
extern crate user_lib;
extern crate alloc;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    let args = line
                        .split(' ')
                        .map(|arg| format!("{arg}\0"))
                        .collect::<Vec<String>>();
                    let mut args_addr = args
                        .iter()
                        .map(|arg| arg.as_ptr())
                        .collect::<Vec<*const u8>>();
                    args_addr.push(core::ptr::null());

                    let pid = fork();
                    if pid == 0 {
                        // child process
                        if exec(args[0].as_str(), &args_addr) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                    line.clear();
                }
                print!(">> ");
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}
