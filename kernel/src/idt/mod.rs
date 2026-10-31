use core::arch::asm;
use core::cell::UnsafeCell;

mod entry;
mod handlers;
mod types;

use crate::idt::handlers::*;
use crate::idt::types::{Exception};
use entry::{Entry, IdtPtr};

const IDT_SIZE: usize = 256;

struct IdtTable(UnsafeCell<[Entry; IDT_SIZE]>);
unsafe impl Sync for IdtTable {}

// static mut IDT: [Entry; IDT_SIZE] = [Entry::MISSING; IDT_SIZE];
static IDT: IdtTable = IdtTable(UnsafeCell::new([Entry::MISSING; IDT_SIZE]));

fn read_cs() -> u16 {
    let value: u16;
    unsafe {
        asm!(
        "mov {0:x}, cs",
        out(reg) value,
        options(nomem, nostack, preserves_flags));
    }
    value
}

fn register(e: Exception) {
    let cs = read_cs();
    let entry = Entry::new(e.addr(), cs);
    unsafe {
        (*IDT.0.get())[e.vector()] = entry;
    }
}

pub fn init() {
    register(Exception::divide_error(divide_error_handler));
    register(Exception::breakpoint(breakpoint_handler));
    register(Exception::invalid_opcode(invalid_opcode_handler));
    register(Exception::general_protection(general_protection_handler));

    let ptr = IdtPtr::new();

    unsafe {
        asm!("lidt [{}]",
        in(reg) &ptr,
        options(readonly, nostack, preserves_flags));
    }
}
