use core::arch::asm;
use core::fmt::{self, Write};

const COM1: u16 = 0x3F8;
const COM1_STATUS: u16 = 0x3FD;

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags),
        );
    }
    value
}

pub fn try_read_byte() -> Option<u8> {
    let status: u8 = unsafe { inb(COM1_STATUS) };

    if (status & 0x01) != 0 {
        return Some(unsafe { inb(COM1) });
    }
    None
}

// Записать один байт в порт ввода-вывода.
unsafe fn out_b(port: u16, value: u8) {
    unsafe {
        asm!(
        "out dx, al", // операция вывода al на dx
        in("dx") port, // port to dx reg
        in("al") value, // value to al reg
        options(nomem, nostack, preserves_flags),
        );
    }
}

pub fn write_byte(b: u8) {
    unsafe {
        out_b(COM1, b);
    }
}

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().for_each(|c| write_byte(c as u8));
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = Serial.write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => { $crate::kprint!("{}\n", format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! exp_print {
    () => {
    $crate::kprint!("EXCEPTION:")
    };
    ($($arg:tt)*) => {
        $crate::kprint!("EXCEPTION: {}", format_args!($($arg)*))
    };
}
#[macro_export]
macro_rules! exp_println {
    () => {
    $crate::kprintln!("EXCEPTION:")
    };
    ($($arg:tt)*) => {
        $crate::kprintln!("EXCEPTION: {}", format_args!($($arg)*))
    };
}
