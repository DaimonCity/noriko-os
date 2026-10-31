use crate::idt::{InterruptStackFrame};

pub type Handler = extern "x86-interrupt" fn(InterruptStackFrame);
pub type HandlerWithCode = extern "x86-interrupt" fn(InterruptStackFrame, u64);

pub trait HandlerFn {
    fn addr(self) -> u64;
}

impl HandlerFn for Handler {
    fn addr(self) -> u64 {
        self as *const () as u64
    }
}

impl HandlerFn for HandlerWithCode {
    fn addr(self) -> u64 {
        self as *const () as u64
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub(super) enum Vector {
    DivideError = 0,
    Breakpoint = 3,
    InvalidOpcode = 6,
    DoubleFault = 8,
    GeneralProtection = 13,
    PageFault = 14,
}

#[derive(Clone, Copy)]
pub(super) struct Exception {
    vector: usize,
    addr: u64,
}

impl Exception {
    pub fn addr(&self) -> u64 {
        self.addr
    }
    pub fn vector(&self) -> usize {
        self.vector
    }

    pub fn divide_error(h: Handler) -> Self {
        Exception {
            vector: Vector::DivideError as usize,
            addr: h.addr(),
        }
    }
    pub fn breakpoint(h: Handler) -> Self {
        Exception {
            vector: Vector::Breakpoint as usize,
            addr: h.addr(),
        }
    }
    pub fn invalid_opcode(h: Handler) -> Self {
        Exception {
            vector: Vector::InvalidOpcode as usize,
            addr: h.addr(),
        }
    }
    pub fn general_protection(h: HandlerWithCode) -> Self {
        Exception {
            vector: Vector::GeneralProtection as usize,
            addr: h.addr(),
        }
    }
}
