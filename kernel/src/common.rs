use core::arch::asm;

use crate::sbi::shutdown;
use log::error;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        error!("[kernel] Panicked: {}", info.message());
    }
    shutdown(true)
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
