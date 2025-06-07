#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(user_lib::test_utils::test_runner)]

use alloc::{format, string::String, vec::Vec};
use user_lib::{OpenFlags, close, console::getchar, exec, fork, open, waitpid};

#[macro_use]
extern crate user_lib;
extern crate alloc;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

fn parse_cmd(line: &str) -> (Vec<String>, Option<String>, Option<String>) {
    let mut args = Vec::new();
    let mut input_file = None;
    let mut output_file = None;

    let mut arg_iter = line.split_whitespace();
    while let Some(arg) = arg_iter.next() {
        // TODO: check valid
        match arg {
            ">" => output_file = Some(format!("{}\0", arg_iter.next().unwrap())),
            "<" => input_file = Some(format!("{}\0", arg_iter.next().unwrap())),
            _ => args.push(format!("{arg}\0")),
        }
    }

    (args, input_file, output_file)
}

fn get_args_addr(args: &[String]) -> Vec<*const u8> {
    let mut args_addr = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const u8>>();
    args_addr.push(core::ptr::null());
    args_addr
}

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
                    let (args, input_file, output_file) = parse_cmd(&line);
                    let args_addr = get_args_addr(&args);
                    let pid = fork();
                    if pid == 0 {
                        // child process
                        if let Some(input_file) = input_file {
                            close(0);
                            let fd = open(input_file.as_str(), OpenFlags::RDONLY);
                            debug_assert_eq!(fd, 0);
                        }
                        if let Some(output_file) = output_file {
                            close(1);
                            let fd =
                                open(output_file.as_str(), OpenFlags::CREATE | OpenFlags::WRONLY);
                            debug_assert_eq!(fd, 1);
                        }

                        if exec(args[0].as_str(), &args_addr) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }

                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
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
