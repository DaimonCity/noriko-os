#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::arch::asm;
use core::panic::PanicInfo;

entry_point!(kernel_main);

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

const COM1: u16 = 0x3F8; // 1016: 256 * 3 + 15 * 16 + 8

fn serial_write_byte(b: u8) {
    unsafe {
        out_b(COM1, b);
    }
}

fn kprint(s: &str) {
    for c in s.chars() {
        serial_write_byte(c as u8);
    }
}

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    kprint("Hello World!");
    loop {
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint("PANIC\n");
    loop {}
}