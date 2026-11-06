use crate::{kprintln, process};
use core::sync::atomic::{AtomicUsize, Ordering};


static COUNTER: AtomicUsize = AtomicUsize::new(0);
/// Пример фоновой задачи (Worker)
extern "C" fn worker_entry() -> ! {
    kprintln!("[Worker Task] Started execution in its own stack context!");
    loop {
        // Простейшая имитация полезной работы
        for _ in 0..500_000 {
            core::hint::spin_loop();
        }
        kprintln!("[Worker Task] {} Tick...", COUNTER.load(Ordering::Relaxed));
        COUNTER.fetch_add(1, Ordering::SeqCst);
        process::schedule();
    }
}

/// Сервисная процедура: классификация ОС и отслеживание ресурсов (Лекции 1-3)
pub fn sys_info() {
    let (active, zombies, free) = process::resource_stats();
    kprintln!("=== Noriko OS: System Classification & Resources ===");
    kprintln!("Architecture     : Monolithic (3-layer structured: Main -> Services -> Utils)");
    kprintln!("Interaction Mode : Interactive Time-Sharing prototype");
    kprintln!("Users            : Single-user");
    kprintln!(
        "Process Table    : {} total slots ({} active, {} zombie, {} free)",
        process::MAX_PROCESSES,
        active,
        zombies,
        free
    );
}

/// Сервисная процедура: вывод таблицы процессов (дескрипторов PCB)
pub fn sys_ps() {
    kprintln!("PID  PPID  STATE     PRIO  NAME");
    kprintln!("---------------------------------------");
    process::for_each_process(|p| {
        kprintln!(
            "{:<4} {:<5} {:<9} {:<5} {}",
            p.pid,
            p.ppid,
            p.state.as_str(),
            p.priority,
            p.name
        );
    });
}

/// Сервисная процедура: создание процесса
pub fn sys_spawn(name: &str) {
    let (static_name, entry) = match name {
        "worker" => ("worker_task", worker_entry),
        _ => ("user_task", worker_entry),
    };
    match process::spawn(static_name, 5, entry) {
        Ok(pid) => kprintln!(
            "Spawned process '{}' with PID {} (Entry: {:#x}, State: READY)",
            static_name,
            pid,
            entry as *const () as usize
        ),
        Err(e) => kprintln!("Spawn error: {}", e),
    }
}

/// Сервисная процедура: завершение процесса (перевод в Zombie)
pub fn sys_exit(pid: usize, code: i32) {
    match process::exit_process(pid, code) {
        Ok(()) => kprintln!("Process {} exited with code {} -> State: ZOMBIE", pid, code),
        Err(e) => kprintln!("Error: {}", e),
    }
}

/// Сервисная процедура: вызов wait() родительским процессом для очистки зомби
pub fn sys_wait() {
    match process::wait_zombie() {
        Some((pid, code)) => kprintln!("Reaped ZOMBIE process PID {} (exit status: {})", pid, code),
        None => kprintln!("No zombie child processes waiting."),
    }
}
