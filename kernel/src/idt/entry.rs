use crate::idt::{IDT, IDT_SIZE};
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy)]
#[repr(C)]
pub(super) struct Entry {
    offset_low: u16,  // 0000 0000 0000 0000
    selector: u16,    // 0000 0000 0000 0000
    ist: u8,          // 0000 0000
    type_attr: u8,    // 0000 0000
    offset_mid: u16,  // 0000 0000 0000 0000
    offset_high: u32, // 0000 0000 0000 0000 0000 0000 0000 0000
    reserved: u32,    // 0000 0000 0000 0000 0000 0000 0000 0000
}

impl Entry {
    pub(crate) const MISSING: Entry = Entry {
        offset_low: 0,
        selector: 0,
        ist: 0,
        type_attr: 0,
        offset_mid: 0,
        offset_high: 0,
        reserved: 0,
    };

    pub(super) fn new(handler: u64, selector: u16) -> Entry {
        let offset_low = (handler & 0xFFFF) as u16; // 0000 0000 0000 0000
        // F    F    F    F
        let offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        let offset_high = (handler >> 32) as u32;

        Entry {
            offset_low,
            selector,
            ist: 0,
            type_attr: 0x8E,
            offset_mid,
            offset_high,
            reserved: 0,
        }
    }
}

// Значение для регистра IDTR: размер таблицы и её адрес.
#[repr(C, packed)]
pub(super) struct IdtPtr {
    limit: u16,
    base: u64,
}

static IDT_PTR_CREATED: AtomicBool = AtomicBool::new(false);

impl IdtPtr {
    pub(super) fn new() -> Self {
        if IDT_PTR_CREATED.swap(true, Ordering::SeqCst) {
            panic!("IdtPtr already created");
        }
        Self {
            limit: (IDT_SIZE * size_of::<Entry>() - 1) as u16,
            base: &raw const IDT as u64,
        }
    }
}
