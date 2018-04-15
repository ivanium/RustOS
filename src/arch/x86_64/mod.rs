pub mod driver;
pub mod cpu;
pub mod interrupt;
pub mod paging;
pub mod gdt;
pub mod idt;

pub fn init() {
	cpu::enable_nxe_bit();
	cpu::enable_write_protect_bit();
}