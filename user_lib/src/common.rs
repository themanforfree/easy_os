use core::arch::asm;

use crate::exit;

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message();
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}, {}",
            location.file(),
            location.line(),
            err
        );
    } else {
        println!("Panicked: {}", err);
    }
    exit(1);
    unreachable!("sys_exit should not return");
}

pub fn clear_bss() {
    unsafe {
        asm!(
            "
            la a0, sbss
            la a1, ebss
        1:
            beq a0, a1, 2f
            sw zero, 0(a0)
            addi a0, a0, 4
            j 1b
        2:
        "
        )
    }
}
