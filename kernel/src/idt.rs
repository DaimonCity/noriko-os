use core::arch::asm;


const IDT_SIZE: usize = 256;

#[derive(Clone, Copy)]
#[repr(C)]
struct Entry {
    offset_low: u16,  // 0000 0000 0000 0000
    selector: u16,    // 0000 0000 0000 0000
    ist: u8,          // 0000 0000
    type_attr: u8,    // 0000 0000
    offset_mid: u16,  // 0000 0000 0000 0000
    offset_high: u32, // 0000 0000 0000 0000 0000 0000 0000 0000
    reserved: u32,    // 0000 0000 0000 0000 0000 0000 0000 0000
}

impl Entry {
    const MISSING: Entry = Entry {
        offset_low: 0,
        selector: 0,
        ist: 0,
        type_attr: 0,
        offset_mid: 0,
        offset_high: 0,
        reserved: 0,
    };

    fn new(handler: u64, selector: u16) -> Entry {
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
struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: [Entry; IDT_SIZE] = [Entry::MISSING; IDT_SIZE];

// То, что процессор кладёт на стек при исключении.
#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: BREAKPOINT\n{:#x?}", frame);
}

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

pub fn init() {
    let cs = read_cs();

    unsafe {
        IDT[3] = Entry::new(breakpoint_handler as *const () as u64, cs);

        let ptr = IdtPtr {
            limit: (IDT_SIZE * size_of::<Entry>() - 1) as u16,
            base: &raw const IDT as u64,
        };

        asm!("lidt [{}]",
        in(reg) &ptr,
        options(readonly, nostack, preserves_flags));
    }
}
