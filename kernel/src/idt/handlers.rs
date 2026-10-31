#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}
pub(super) extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    crate::exp_println!("DIVIDE ERROR\n{:#x?}", frame);
    loop {}
}
pub(super) extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    crate::exp_println!("BREAKPOINT\n{:#x?}", frame);
}
pub(super) extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    crate::exp_println!("INVALID OPCODE (vector 6)\n{:#x?}", frame);
    loop {}
}

pub(super) extern "x86-interrupt" fn general_protection_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::exp_println!(
        "GENERAL PROTECTION FAULT (vector 13)\nerror code: {:#x}\n{:#x?}",
        error_code,
        frame
    );
    loop {}
}
