//! 系统调用解析执行模块

use arch::interrupt::TrapFrame;
use process::*;
use util;
use kmodule::{do_init_module, do_cleanup_module, print_modules};

/// 系统调用入口点
///
/// 当发生系统调用中断时，中断服务例程将控制权转移到这里。
/// 它从中断帧中提取参数，根据系统调用号分发执行具体操作。
/// 它同时支持 xv6的64位程序 和 uCore的32位程序。
pub fn syscall(tf: &TrapFrame, is32: bool) -> i32 {
    let id = match is32 {
        false => Syscall::Xv6(tf.rax),
        true => Syscall::Ucore(tf.rax),
    };
    let args = match is32 {
        // For ucore x86
        true => [tf.rdx, tf.rcx, tf.rbx, tf.rdi, tf.rsi, 0],
        // For xv6 x86_64
        false => [tf.rdi, tf.rsi, tf.rdx, tf.rcx, tf.r8, tf.r9],
    };

    match id {
        Syscall::Xv6(SYS_READ) | Syscall::Ucore(UCORE_SYS_READ) =>
            sys_read(args[0], args[1] as *mut u8, args[2]),
        Syscall::Xv6(SYS_WRITE) | Syscall::Ucore(UCORE_SYS_WRITE) =>
            sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Xv6(SYS_OPEN) | Syscall::Ucore(UCORE_SYS_OPEN) =>
            sys_open(args[0] as *const u8, args[1]),
        Syscall::Xv6(SYS_CLOSE) | Syscall::Ucore(UCORE_SYS_CLOSE) =>
            sys_close(args[0]),
        Syscall::Xv6(SYS_WAIT) | Syscall::Ucore(UCORE_SYS_WAIT) =>
            sys_wait(args[0], args[1] as *mut i32),
        Syscall::Xv6(SYS_FORK) | Syscall::Ucore(UCORE_SYS_FORK) =>
            sys_fork(tf),
        Syscall::Xv6(SYS_KILL) | Syscall::Ucore(UCORE_SYS_KILL) =>
            sys_kill(args[0]),
        Syscall::Xv6(SYS_EXIT) | Syscall::Ucore(UCORE_SYS_EXIT) =>
            sys_exit(args[0]),
        Syscall::Ucore(UCORE_SYS_YIELD) =>
            sys_yield(),
        Syscall::Ucore(UCORE_SYS_GETPID) =>
            sys_getpid(),
        Syscall::Ucore(UCORE_SYS_SLEEP) =>
            sys_sleep(args[0]),
        Syscall::Ucore(UCORE_SYS_GETTIME) =>
            sys_get_time(),
        Syscall::Ucore(UCORE_SYS_LAB6_SET_PRIORITY) =>
            sys_lab6_set_priority(args[0]),
        Syscall::Ucore(SYS_INIT_MODULE) =>
            sys_init_module(args[0] as *const u8),
        Syscall::Ucore(SYS_CLEANUP_MODULE) =>
            sys_cleanup_module(args[0] as *const u8),
        Syscall::Ucore(SYS_LIST_MODULE) =>
            sys_list_module(),
        Syscall::Ucore(UCORE_SYS_PUTC) =>
            {
                print!("{}", args[0] as u8 as char);
                0
            }
        _ => {
            error!("unknown syscall {:#x?}", id);
            -1
        }
    }
}

fn sys_read(fd: usize, base: *mut u8, len: usize) -> i32 {
    info!("read: fd: {}, base: {:?}, len: {:#x}", fd, base, len);
    use core::slice;
    use core::str;
    use alloc::string::String;

    let slice = unsafe { slice::from_raw_parts_mut(base, len) };
    
    // let mut data = String::new();
    let mut data = String::from("matrix");

    // use core::io;
    // io::stdin().read_line(&mut data)
    //     .ok()
    //     .expect("Failed to read line");
    slice.copy_from_slice(unsafe { &(data.as_bytes()[..len]) });
    print!("{}", str::from_utf8(slice).unwrap());
    0
}

fn sys_write(fd: usize, base: *const u8, len: usize) -> i32 {
    // info!("write: fd: {}, base: {:?}, len: {:#x}", fd, base, len);
    use core::slice;
    use core::str;
    let slice = unsafe { slice::from_raw_parts(base, len) };
    print!("{}", str::from_utf8(slice).unwrap());
    0
}

fn sys_open(path: *const u8, flags: usize) -> i32 {
    let path = unsafe { util::from_cstr(path) };
    info!("open: path: {:?}, flags: {:?}", path, flags);
    match path {
        "stdin:" => 0,
        "stdout:" => 1,
        _ => -1,
    }
}

fn sys_close(fd: usize) -> i32 {
    info!("close: fd: {:?}", fd);
    0
}

/// Fork the current process. Return the child's PID.
fn sys_fork(tf: &TrapFrame) -> i32 {
    let mut processor = PROCESSOR.try().unwrap().lock();
    let mut mc = MC.try().unwrap().lock();
    let new = processor.current().fork(tf, &mut mc);
    let pid = processor.add(new);
    info!("fork: {} -> {}", processor.current_pid(), pid);
    pid as i32
}

/// Wait the process exit.
/// Return the PID. Store exit code to `code` if it's not null.
fn sys_wait(pid: usize, code: *mut i32) -> i32 {
    let mut processor = PROCESSOR.try().unwrap().lock();
    match processor.current_wait_for(pid) {
        WaitResult::Ok(pid, error_code) => {
            if !code.is_null() {
                unsafe { *code = error_code as i32 };
            }
            0
        },
        WaitResult::NotExist => -1,
    }
}

fn sys_yield() -> i32 {
    info!("yield:");
    let mut processor = PROCESSOR.try().unwrap().lock();
    processor.set_reschedule();
    0
}

/// Kill the process
fn sys_kill(pid: usize) -> i32 {
    PROCESSOR.try().unwrap().lock().kill(pid);
    0
}

/// Get the current process id
fn sys_getpid() -> i32 {
    PROCESSOR.try().unwrap().lock().current_pid() as i32
}

/// Exit the current process
fn sys_exit(error_code: usize) -> i32 {
    let mut processor = PROCESSOR.try().unwrap().lock();
    let pid = processor.current_pid();
    processor.exit(pid, error_code);
    0
}

fn sys_sleep(time: usize) -> i32 {
    info!("sleep: {} ticks", time);
    let mut processor = PROCESSOR.try().unwrap().lock();
    let pid = processor.current_pid();
    processor.sleep(pid, time);
    0
}

fn sys_get_time() -> i32 {
    let processor = PROCESSOR.try().unwrap().lock();
    processor.get_time() as i32
}

fn sys_lab6_set_priority(priority: usize) -> i32 {
    let mut processor = PROCESSOR.try().unwrap().lock();
    processor.lab6_set_priority(priority as u8);
    0
}

fn sys_init_module(name: *const u8) -> i32{
    do_init_module(name)
}

fn sys_cleanup_module(name: *const u8) -> i32{
    do_cleanup_module(name)
}

fn sys_list_module() -> i32{
    print_modules();
    0
}

#[derive(Debug)]
enum Syscall {
    Xv6(usize),
    Ucore(usize),
}

const SYS_FORK: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_WAIT: usize = 3;
const SYS_PIPE: usize = 4;
const SYS_READ: usize = 5;
const SYS_KILL: usize = 6;
const SYS_EXEC: usize = 7;
const SYS_FSTAT: usize = 8;
const SYS_CHDIR: usize = 9;
const SYS_DUP: usize = 10;
const SYS_GETPID: usize = 11;
const SYS_SBRK: usize = 12;
const SYS_SLEEP: usize = 13;
const SYS_UPTIME: usize = 14;
const SYS_OPEN: usize = 15;
const SYS_WRITE: usize = 16;
const SYS_MKNOD: usize = 17;
const SYS_UNLINK: usize = 18;
const SYS_LINK: usize = 19;
const SYS_MKDIR: usize = 20;
const SYS_CLOSE: usize = 21;
const SYS_CHMOD: usize = 22;

const UCORE_SYS_EXIT: usize = 1;
const UCORE_SYS_FORK: usize = 2;
const UCORE_SYS_WAIT: usize = 3;
const UCORE_SYS_EXEC: usize = 4;
const UCORE_SYS_CLONE: usize = 5;
const UCORE_SYS_YIELD: usize = 10;
const UCORE_SYS_SLEEP: usize = 11;
const UCORE_SYS_KILL: usize = 12;
const UCORE_SYS_GETTIME: usize = 17;
const UCORE_SYS_GETPID: usize = 18;
const UCORE_SYS_MMAP: usize = 20;
const UCORE_SYS_MUNMAP: usize = 21;
const UCORE_SYS_SHMEM: usize = 22;
const UCORE_SYS_PUTC: usize = 30;
const UCORE_SYS_PGDIR: usize = 31;
const UCORE_SYS_OPEN: usize = 100;
const UCORE_SYS_CLOSE: usize = 101;
const UCORE_SYS_READ: usize = 102;
const UCORE_SYS_WRITE: usize = 103;
const UCORE_SYS_SEEK: usize = 104;
const UCORE_SYS_FSTAT: usize = 110;
const UCORE_SYS_FSYNC: usize = 111;
const UCORE_SYS_GETCWD: usize = 121;
const UCORE_SYS_GETDIRENTRY: usize = 128;
const UCORE_SYS_DUP: usize = 130;
const UCORE_SYS_LAB6_SET_PRIORITY: usize = 255;

const SYS_INIT_MODULE: usize = 200;
const SYS_CLEANUP_MODULE: usize = 201;
const SYS_LIST_MODULE: usize = 202;