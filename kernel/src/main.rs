#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod serial;
pub mod idt;

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);
fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    kprintln!("start");
    idt::init();
    kprintln!("IDT loaded");
    unsafe { core::arch::asm!("int3"); }
    kprintln!("returned from handler");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("PANIC: {}", info);
    loop {}
}
