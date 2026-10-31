#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod idt;
mod serial;

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);
fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    idt::init();
    serial_test_typing();
    loop {
        {}
    }
}

fn serial_test_typing() {
    kprintln!("type something:");
    loop {
        if let Some(b) = serial::try_read_byte() {
            kprintln!("got byte: {:#x}", b);
        }
    }
}

fn _idt_test_divide_exception() {
    kprintln!("start divide test");
    unsafe {
        core::arch::asm!(
        "div ecx",
        inout("eax") 1u32 => _,
        inout("edx") 0u32 => _,
        in("ecx") 0u32,
        );
    }
    kprintln!("returned from divide handler");
}

fn _idt_test_breakpoint_exception() {
    kprintln!("start breakpoint test");
    kprintln!("IDT loaded");
    unsafe {
        core::arch::asm!("int3");
    }
    kprintln!("returned from breakpoint handler");
}

fn _idt_test_invalid_opcode_exception() {
    kprintln!("start ud2 test");
    unsafe {
        core::arch::asm!("ud2");
    }
    kprintln!("unreachable");
}

fn _idt_test_gpf_exception() {
    kprintln!("start gpf test");
    unsafe {
        core::arch::asm!(
        "mov ax, 0x28",
        "mov ds, ax",
        out("ax") _,
        );
    }
    kprintln!("unreachable");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("PANIC: {}", info);
    loop {}
}
