use crate::{BUF_SIZE, kprint, kprintln, process, serial};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static SHELL: ShellStatic = ShellStatic::new();
// static TICKER: AtomicU32 = AtomicU32::new(0);
//
// fn get_current_tick() -> u32 {
//     TICKER.load(Ordering::Relaxed)
// }

pub extern "C" fn shell_entry() -> ! {
    kprintln!("[Shell Task] Started execution in its own stack context!");
    kprint!("> "); // Печатаем приглашение один раз при запуске

    loop {
        if let Some(b) = serial::try_read_byte() {
            SHELL.with_lock(|shell| {
                shell.handle_byte(b);
            });
            // TICKER.store(0, Ordering::Relaxed);
        } else {
            // Если ввода нет, даем процессору отдохнуть или переключаем контекст
            core::hint::spin_loop();
            // TICKER.fetch_add(1, Ordering::SeqCst);
            // if (get_current_tick() + 1) >= 10_000_000 {
            process::schedule();
            //     TICKER.store(0, Ordering::Relaxed);
            // }
        }
    }
}

enum State {
    Normal,
    Esc,
    Csi,
}

pub(crate) struct Shell {
    buf: [u8; BUF_SIZE],
    len: usize,
    state: State,
}

pub(crate) struct ShellStatic {
    locked: AtomicBool,
    shell: UnsafeCell<Shell>,
}
unsafe impl Sync for ShellStatic {}
impl ShellStatic {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            shell: UnsafeCell::new(Shell::new()),
        }
    }

    fn with_lock<R>(&self, f: impl FnOnce(&mut Shell) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        let result = unsafe { f(&mut *self.shell.get()) };

        self.locked.store(false, Ordering::Release);
        result
    }
}

impl Shell {
    pub(crate) const fn new() -> Self {
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

fn execute(line: &str) {
    let (command, args) = line.trim().split_once(' ').unwrap_or((line.trim(), ""));

    match command {
        "help" => {
            kprintln!("Available commands:");
            kprintln!("  help              - show this message");
            kprintln!("  echo <msg>        - print message");
            kprintln!("  sysinfo           - OS classification & resource status (Lectures 1-3)");
            kprintln!("  ps                - list process descriptors / PCB table (Lecture 4)");
            kprintln!("  spawn [name]      - create a new process (worker|calc|io)");
            kprintln!("  kill <pid>        - finish process -> ZOMBIE state (exit)");
            kprintln!("  wait              - reap zombie process and free PCB slot");
        }
        "echo" => kprintln!("{}", args),
        "sysinfo" => crate::services::sys_info(),
        "ps" => crate::services::sys_ps(),
        "spawn" => crate::services::sys_spawn(args),
        "kill" => {
            if let Ok(pid) = args.trim().parse::<usize>() {
                crate::services::sys_exit(pid, 0);
            } else {
                kprintln!("Usage: kill <pid>");
            }
        }
        "wait" => crate::services::sys_wait(),
        "sched" => crate::process::schedule(),
        "" => {}
        _ => kprintln!("Unknown command: {}", command),
    }
}
