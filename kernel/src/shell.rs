use crate::{BUF_SIZE, kprint, kprintln, serial};

enum State {
    Normal,
    Esc,
    Csi,
}

struct Shell {
    buf: [u8; BUF_SIZE],
    len: usize,
    state: State,
}

impl Shell {
    fn new() -> Self {
        Self {
            buf: [0u8; BUF_SIZE],
            len: 0,
            state: State::Normal,
        }
    }
    fn handle_byte(&mut self, b: u8) {
        match self.state {
            State::Normal => self.handle_normal(b),
            State::Esc => match b {
                0x1b => {}
                b'[' => self.state = State::Csi,
                _ => {
                    self.state = State::Normal;
                    self.handle_normal(b)
                }
            },
            State::Csi if (0x40..=0x7e).contains(&b) => {
                self.state = State::Normal;
            }
            State::Csi => {}
        }
    }
    fn handle_normal(&mut self, b: u8) {
        match b {
            0x1b => self.state = State::Esc,
            0x0d => {
                serial::write_byte(b'\r');
                serial::write_byte(b'\n');
                match core::str::from_utf8(&self.buf[..self.len]) {
                    Ok(line) => {
                        execute(line);
                    }
                    Err(_) => kprintln!("<invalid utf-8>"),
                }
                self.len = 0;
                self.buf.fill(0);
                kprint!("> ");
            }
            0x7f | 0x08 => {
                if self.len > 0 {
                    self.len -= 1;
                    serial::write_byte(0x08);
                    serial::write_byte(b' ');
                    serial::write_byte(0x08);
                }
            }
            0x20..=0x7e if self.len < BUF_SIZE => {
                self.buf[self.len] = b;
                self.len += 1;
                serial::write_byte(b);
            }
            _ => {}
        }
    }
}

pub fn shell() -> ! {
    let mut shell = Shell::new();
    kprint!("> ");
    loop {
        if let Some(b) = serial::try_read_byte() {
            shell.handle_byte(b);
        }
    }
}

fn execute(line: &str) {
    let (command, args) = line.split_once(' ').unwrap_or((line, ""));

    match command {
        "help" => {
            kprintln!("help");
            kprintln!("show this message");
            kprintln!();
            kprintln!("echo");
            kprintln!("return output");
            kprintln!("\tSYNTAX: echo <message>");
        }
        "echo" => {
            kprintln!("{}", args);
        }
        "" => {
            kprintln!();
        }
        _ => {
            kprintln!("Unknown command: {}", command);
        }
    }
}
