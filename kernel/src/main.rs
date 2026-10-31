#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod serial;
mod idt;

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);
fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    idt::init();
    idt_test_invalid_opcode_exception();
    loop {}
}

fn idt_test_divide_exception() {
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

fn idt_test_breakpoint_exception() {
    kprintln!("start breakpoint test");
    kprintln!("IDT loaded");
    unsafe { core::arch::asm!("int3"); }
    kprintln!("returned from breakpoint handler");
}

fn idt_test_invalid_opcode_exception() {
    kprintln!("start ud2 test");
    unsafe { core::arch::asm!("ud2"); }
    kprintln!("unreachable");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("PANIC: {}", info);
    loop {}
}
