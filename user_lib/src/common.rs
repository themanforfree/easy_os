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
    unsafe extern "C" {
        static sbss: u8;
        static ebss: u8;
    }
    let bss_start = unsafe { &sbss as *const u8 as usize };
    let bss_end = unsafe { &ebss as *const u8 as usize };
    (bss_start..bss_end).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
