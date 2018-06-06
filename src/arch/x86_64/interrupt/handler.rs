//! 中断服务例程
//!
//! # 中断处理总流程
//!
//! * 中断发生时，CPU首先完成以下工作：
//!     * 设置rsp的值（Ring3->0 或 DoubleFault，在TSS中设置）
//!     * 向栈中push一些值（Ring0时3项，Ring3时5项，某些中断还有错误码）
//!     * 跳转到`vector{i}`符号（位于`vector.asm`）
//! * 进入中断服务例程后，完成以下工作：
//!     * `vector{i}`向栈中push空错误码（如CPU没有做）和中断号
//!     * 跳转到`__alltraps`（位于`trap.asm`）
//!     * `__alltraps`向栈中push全部寄存器的值
//!     * 调用`rust_trap`，并把此时的rsp传过来，也就是`TrapFrame`的指针
//! * `rust_trap`完成以下工作：
//!     * 根据tf中的中断号，再次分发到具体的中断处理函数。
//!       一些函数可能会调用switch()，切换到其它线程执行，在某一时刻再切换回来。
//!     * 如果需要，执行进程调度
//! * `__alltraps`完成以下工作：
//!     * 检查tf中的cs段，如果是用户态，向TSS中设置下一次中断时重置rsp的值，使其指向该用户线程的内核栈
//!     * 从栈中pop全部寄存器的值，pop中断号和错误码
//!     * 执行iret，CPU从栈中pop一些值（若其中的CS是Ring0，3项，否则5项），重置rip和rsp（Ring3时）

use super::TrapFrame;

#[no_mangle]
pub extern fn rust_trap(tf: &mut TrapFrame) {
    trace!("Interrupt: {:#x}", tf.trap_num);
    // Dispatch
    match tf.trap_num as u8 {
        T_BRKPT => breakpoint(),
        T_DBLFLT => double_fault(tf),
        T_PGFLT => page_fault(tf),
        T_IRQ0...64 => {
            let irq = tf.trap_num as u8 - T_IRQ0;
            match irq {
                IRQ_TIMER => timer(),
                IRQ_KBD => keyboard(),
                IRQ_COM1 => com1(),
                IRQ_COM2 => com2(),
                _ => panic!("Invalid IRQ number."),
            }
            #[cfg(feature = "use_apic")]
            use arch::driver::apic::ack;
            #[cfg(not(feature = "use_apic"))]
            use arch::driver::pic::ack;
            ack(irq);
        }
        T_SWITCH_TOK => to_kernel(tf),
        T_SWITCH_TOU => to_user(tf),
        T_SYSCALL => syscall(tf),
        T_SYSCALL32 => syscall32(tf),
        T_DIVIDE | T_GPFLT | T_ILLOP => error(tf),
        _ => panic!("Unhandled interrupt {:x}", tf.trap_num),
    }

    use process::PROCESSOR;
    PROCESSOR.try().unwrap().lock().schedule();
}

fn breakpoint() {
    error!("\nEXCEPTION: Breakpoint");
}

fn double_fault(tf: &TrapFrame) {
    error!("\nEXCEPTION: Double Fault\n{:#x?}", tf);
    loop {}
}

fn page_fault(tf: &mut TrapFrame) {
    use x86_64::registers::control_regs::cr2;
    let addr = cr2().0;
    error!("\nEXCEPTION: Page Fault @ {:#x}, code: {:#x}", addr, tf.error_code);

    use memory::page_fault_handler;
    if page_fault_handler(addr) {
        return;
    }

    error(tf);
}

use super::consts::*;

fn keyboard() {
    use arch::driver::keyboard;
    info!("\nInterupt: Keyboard");
    let c = keyboard::get();
    info!("Key = '{}' {}", c as u8 as char, c);
}

fn com1() {
    use arch::driver::serial::COM1;
    info!("\nInterupt: COM1");
    COM1.lock().receive();
}

fn com2() {
    use arch::driver::serial::COM2;
    info!("\nInterupt: COM2");
    COM2.lock().receive();
}

fn timer() {
    // use process::PROCESSOR;
    // let mut processor = PROCESSOR.try().unwrap().lock();
    // processor.tick();
}

fn to_user(tf: &mut TrapFrame) {
    use arch::gdt;
    info!("\nInterupt: To User");
    tf.cs = gdt::UCODE_SELECTOR.0 as usize;
    tf.ss = gdt::UDATA_SELECTOR.0 as usize;
    tf.rflags |= 3 << 12;   // 设置EFLAG的I/O特权位，使得在用户态可使用in/out指令
}

fn to_kernel(tf: &mut TrapFrame) {
    use arch::gdt;
    info!("\nInterupt: To Kernel");
    tf.cs = gdt::KCODE_SELECTOR.0 as usize;
    tf.ss = gdt::KDATA_SELECTOR.0 as usize;
}

fn syscall(tf: &mut TrapFrame) {
    info!("\nInterupt: Syscall {:#x?}", tf.rax);
    use syscall::syscall;
    let ret = syscall(tf, false);
    tf.rax = ret as usize;
}

fn syscall32(tf: &mut TrapFrame) {
    //    info!("\nInterupt: Syscall {:#x?}", tf.rax);
    use syscall::syscall;
    let ret = syscall(tf, true);
    tf.rax = ret as usize;
}

fn error(tf: &TrapFrame) {
    use process::PROCESSOR;
    if let Some(processor) = PROCESSOR.try() {
        let mut processor = processor.lock();
        let pid = processor.current_pid();
        error!("Process {} error:\n{:#x?}", pid, tf);
        processor.exit(pid, 0x100); // TODO: Exit code for error
    } else {
        error!("Exception {:#x} when processor not inited\n{:#x?}", tf.trap_num, tf);
        loop {}
    }
}

#[no_mangle]
pub extern fn set_return_rsp(tf: &TrapFrame) {
    use arch::gdt::Cpu;
    use core::mem::size_of;
    if tf.cs & 0x3 == 3 {
        Cpu::current().set_ring0_rsp(tf as *const _ as usize + size_of::<TrapFrame>());
    }
}
