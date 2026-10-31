use crate::{BUF_SIZE, kprint, kprintln, serial};

pub fn shell() -> ! {
    let mut buf = [0u8; BUF_SIZE]; // 64 байта, все нули
    let mut len = 0; // сколько ячеек занято

    kprint!("> ");
    loop {
        if let Some(b) = serial::try_read_byte() {
            match b {
                0x0d => {
                    serial::write_byte(b'\r');
                    serial::write_byte(b'\n');
                    match core::str::from_utf8(&buf[..len]) {
                        Ok(line) => {
                            execute(line);
                        }
                        Err(_) => kprintln!("<invalid utf-8>"),
                    }
                    len = 0;
                    buf.fill(0);
                    kprint!("> ");
                }
                0x7f | 0x08 => {
                    if len > 0 {
                        len -= 1;
                        serial::write_byte(0x08);
                        serial::write_byte(b' ');
                        serial::write_byte(0x08);
                    }
                }
                _ => {
                    if len < BUF_SIZE {
                        buf[len] = b;
                        len += 1;
                        serial::write_byte(b);
                    }
                }
            }
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
            kprintln!("");
        }
        _ => {
            kprintln!("Unknown command: {}", command);
        }
    }
}
