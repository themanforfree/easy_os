use core::fmt::Write;
use sbi_rt::console_write_byte;

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            console_write_byte(c);
        }
        Ok(())
    }
}

#[doc(hidden)]
#[inline]
pub fn _print(args: core::fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(core::format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!($($arg)*));
        $crate::println!();
    }}
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_print() {
        print!("   ");
    }
}
