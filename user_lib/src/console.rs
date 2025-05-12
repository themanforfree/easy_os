use crate::write;
use core::fmt::Write;
struct Console;
const STDOUT: usize = 1;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(STDOUT, s.as_bytes());
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
