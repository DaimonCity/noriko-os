use crate::shell;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const MAX_PROCESSES: usize = 16;
pub const STACK_SIZE: usize = 2_i32.pow(14) as usize; // (2^14) КБ стека на процесс

const _: () = assert!(STACK_SIZE % 16 == 0);
#[unsafe(naked)]
#[allow(unused)]
unsafe extern "C" fn switch_context(old_rsp: *mut u64, new_rsp: u64) {
    core::arch::naked_asm!(
        // 1. Сохраняем регистры текущего потока на его собственный стек
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // 2. Записываем текущий RSP по указателю old_rsp (в дескриптор старого процесса)
        "mov [rdi], rsp",
        // 3. Переключаем процессор на стек нового процесса (new_rsp)
        "mov rsp, rsi",
        // 4. Восстанавливаем регистры нового потока с его стека
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        // 5. Прыгаем по адресу возврата, лежащему на вершине нового стека
        "ret",
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Unused,        // Свободный слот в таблице процессов
    Created,       // Создан (6) - переходное состояние при fork()
    Ready,         // Готов к запуску (3) - в очереди к процессору
    RunningKernel, // Выполняется в режиме ядра (2)
    RunningUser,   // Выполняется в режиме задачи (1)
    Sleeping,      // Ожидает недоступный ресурс / сон (4)
    Zombie,        // Зомби (7) - завершился (exit), ждет wait() от родителя
}

impl ProcessState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessState::Unused => "UNUSED",
            ProcessState::Created => "CREATED",
            ProcessState::Ready => "READY",
            ProcessState::RunningKernel => "RUN_KRN",
            ProcessState::RunningUser => "RUN_USR",
            ProcessState::Sleeping => "SLEEP",
            ProcessState::Zombie => "ZOMBIE",
        }
    }
}

/// Аппаратный контекст процесса для диспетчеризации (Лекция 4)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Context {
    pub rsp: u64,
}
/// Дескриптор процесса (PCB / TCB — Лекция 4)
pub struct ProcessDescriptor {
    pub pid: usize,
    pub ppid: usize,
    pub name: &'static str,
    pub state: ProcessState,
    pub priority: u8,
    pub exit_code: i32,
    pub context: Context,
    pub stack: Stack,
    pub next_ready: Option<usize>, // Ссылка для очереди готовых потоков (Лекция 4, рис. 2)
}

#[repr(align(16))]
pub struct Stack([u8; STACK_SIZE]);
impl Stack {
    pub const fn empty() -> Self {
        Self([0; STACK_SIZE])
    }
}

impl Deref for Stack {
    type Target = [u8; STACK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ProcessDescriptor {
    const EMPTY: Self = Self {
        pid: 0,
        ppid: 0,
        name: "",
        state: ProcessState::Unused,
        priority: 0,
        exit_code: 0,
        context: Context { rsp: 0 },
        stack: Stack::empty(),
        next_ready: None,
    };
}

static NEXT_PID: AtomicUsize = AtomicUsize::new(2);
static CURRENT_PID: AtomicUsize = AtomicUsize::new(0);
static CURRENT_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Защищенная таблица процессов с внутренней изменяемостью (Interior Mutability)
/// Реализует блокирующую переменную F(D) для критической секции (Лекция 7, рис. 2)
static PROCESS_TABLE: ProcessTable = ProcessTable::new();
struct ProcessTable {
    locked: AtomicBool,
    entries: UnsafeCell<[ProcessDescriptor; MAX_PROCESSES]>,
}
impl ProcessTable {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            entries: UnsafeCell::new([ProcessDescriptor::EMPTY; MAX_PROCESSES]),
        }
    }

    /// Вход в критическую секцию, выполнение замыкания f и освобождение ресурса
    fn with_lock<R>(&self, f: impl FnOnce(&mut [ProcessDescriptor; MAX_PROCESSES]) -> R) -> R {
        // Неделимая операция «проверка-установка» (Test-and-Set)
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        // Критическая секция: монопольный доступ к таблице процессов
        let result = unsafe { f(&mut *self.entries.get()) };

        // Освобождение ресурса
        self.locked.store(false, Ordering::Release);
        result
    }
}

unsafe impl Sync for ProcessTable {}

/// Инициализация подсистемы процессов: создание Процесса 0 и Процесса 1 (init)
pub fn init() {
    PROCESS_TABLE.with_lock(|t| {
        // Нулевой процесс (swapper/idle) создается вручную
        let _ = _spawn_fill("swapper/idle", 0, 0, 0, crate::main_loop, &mut t[0]);
        // Создаем процесс init/shell it
        let _ = _spawn_fill("init/shell", 1, 0, 10, shell::shell_entry, &mut t[1]);
    });
}

/// Порождение нового процесса (аналог fork + exec на уровне дескрипторов)
pub fn spawn(
    name: &'static str,
    priority: u8,
    entry_point: extern "C" fn() -> !,
) -> Result<usize, &'static str> {
    PROCESS_TABLE.with_lock(|table| {
        for slot in table.iter_mut() {
            if slot.state == ProcessState::Unused {
                let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);
                return Ok(_spawn_fill(name, pid, 1, priority, entry_point, slot));
            }
        }
        Err("Process table is full (resource leak / too many zombies?)")
    })
}

fn _spawn_fill(
    name: &'static str,
    pid: usize,
    ppid: usize,
    priority: u8,
    entry_point: extern "C" fn() -> !,
    slot: &mut ProcessDescriptor,
) -> usize {
    slot.pid = pid;
    slot.name = name;
    slot.ppid = ppid;
    slot.priority = priority;
    slot.exit_code = 0;
    slot.next_ready = None;

    let stack_top = slot.stack.as_mut_ptr() as usize + STACK_SIZE;
    let frame_ptr = (stack_top - 8 * 8) as *mut u64;
    unsafe {
        *frame_ptr.add(0) = 0; // r15
        *frame_ptr.add(1) = 0; // r14
        *frame_ptr.add(2) = 0; // r13
        *frame_ptr.add(3) = 0; // r12
        *frame_ptr.add(4) = 0; // rbx
        *frame_ptr.add(5) = 0; // rbp
        *frame_ptr.add(6) = entry_point as usize as u64; // адрес возврата для ret
    }
    slot.context.rsp = frame_ptr as u64;
    slot.state = ProcessState::Ready;
    pid
}

/// Завершение процесса и переход в состояние Зомби (7) по вызову exit()
pub fn exit_process(pid: usize, code: i32) -> Result<(), &'static str> {
    if pid <= 1 {
        return Err("Cannot terminate system process (PID 0 or 1)");
    }
    PROCESS_TABLE.with_lock(|table| {
        for slot in table.iter_mut() {
            if slot.state != ProcessState::Unused && slot.pid == pid {
                slot.state = ProcessState::Zombie;
                slot.exit_code = code;
                return Ok(());
            }
        }
        Err("Process not found")
    })
}

/// Сбор зомби-процессов родителем по вызову wait() (Лекция 4 и 6)
pub fn wait_zombie() -> Option<(usize, i32)> {
    PROCESS_TABLE.with_lock(|table| {
        for slot in table.iter_mut() {
            if slot.state == ProcessState::Zombie {
                let pid = slot.pid;
                let code = slot.exit_code;
                slot.state = ProcessState::Unused;
                return Some((pid, code));
            }
        }
        None
    })
}

/// Подсчет занятых и свободных слотов таблицы процессов (Лекция 1: отслеживание ресурса)
pub fn resource_stats() -> (usize, usize, usize) {
    let mut active = 0;
    let mut zombies = 0;
    PROCESS_TABLE.with_lock(|table| {
        for slot in table.iter() {
            match slot.state {
                ProcessState::Unused => {}
                ProcessState::Zombie => zombies += 1,
                _ => active += 1,
            }
        }
    });
    (active, zombies, MAX_PROCESSES - active - zombies)
}

pub fn for_each_process<F: FnMut(&ProcessDescriptor)>(mut f: F) {
    PROCESS_TABLE.with_lock(|table| {
        for slot in table.iter() {
            if slot.state != ProcessState::Unused {
                f(slot);
            }
        }
    })
}

pub fn get_current_pid() -> usize {
    CURRENT_PID.load(Ordering::SeqCst)
}

/// Планировщик (Лекция 5): обход таблицы по индексам в поисках следующего READY процесса
pub fn schedule() {
    let switch_ptrs = PROCESS_TABLE.with_lock(|table| {
        let current_idx = CURRENT_INDEX.load(Ordering::SeqCst);
        let mut next_idx = None;

        for i in 1..=MAX_PROCESSES {
            let idx = (current_idx + i) % MAX_PROCESSES;
            if table[idx].state == ProcessState::Ready {
                next_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = next_idx
            && idx != current_idx
        {
            // 1. Если текущий процесс выполнялся, возвращаем его в очередь готовых (Лекция 4, рис. 1)
            if table[current_idx].state == ProcessState::RunningKernel {
                table[current_idx].state = ProcessState::Ready;
            }
            // 2. Новый выбранный процесс переводим в состояние выполнения
            table[idx].state = ProcessState::RunningKernel;

            CURRENT_INDEX.store(idx, Ordering::SeqCst);
            CURRENT_PID.store(table[idx].pid, Ordering::SeqCst);

            // 3. Берем адрес RSP старого процесса (куда сохраним стек) и значение RSP нового
            let old_rsp = &raw mut table[current_idx].context.rsp;
            let new_rsp = table[idx].context.rsp;
            Some((old_rsp, new_rsp))
        } else {
            None
        }
    });

    if let Some((old_rsp, new_rsp)) = switch_ptrs {
        unsafe {
            switch_context(old_rsp, new_rsp);
        }
    }
}
