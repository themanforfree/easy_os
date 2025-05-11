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
    unsafe extern "C" {
        static sbss: u8;
        static ebss: u8;

    }
    let bss_start = unsafe { &sbss as *const u8 as usize };
    let bss_end = unsafe { &ebss as *const u8 as usize };
    (bss_start..bss_end).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
