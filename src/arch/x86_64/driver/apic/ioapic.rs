//! Migrate from xv6 ioapic.c

/// The I/O APIC manages hardware interrupts for an SMP system.
/// http://www.intel.com/design/chipsets/datashts/29056601.pdf
/// See also picirq.c.

use core::ptr::{Unique};
use syscall::io::{Io, Mmio};
use bit_field::BitField;
use consts::irq::T_IRQ0;

pub unsafe fn init(ioapic_id: u8)
{
	let ioapic = IOAPIC.as_mut();
	assert!(ioapic.id() == ioapic_id, "ioapicinit: id isn't equal to ioapicid; not a MP");

	// Mark all interrupts edge-triggered, active high, disabled,
	// and not routed to any CPUs.
	for i in 0.. ioapic.maxintr() + 1 {
		ioapic.write_irq(i, DISABLED, 0);
	}
}

const IOAPIC_ADDRESS  : u32 = 0xFEC00000;   // Default physical address of IO APIC

const REG_ID     : u8 = 0x00;  // Register index: ID
const REG_VER    : u8 = 0x01;  // Register index: version
const REG_TABLE  : u8 = 0x10;  // Redirection table base

// The redirection table starts at REG_TABLE and uses
// two registers to configure each interrupt.
// The first (low) register in a pair contains configuration bits.
// The second (high) register contains a bitmask telling which
// CPUs can serve that interrupt.

bitflags! {
	flags RedirectionEntry: u32 {
		const DISABLED  = 0x00010000,  // Interrupt disabled
		const LEVEL     = 0x00008000,  // Level-triggered (vs edge-)
		const ACTIVELOW = 0x00002000,  // Active low (vs high)
		const LOGICAL   = 0x00000800,  // Destination is CPU id (vs APIC ID)
		const NONE		= 0x00000000,
	}
}

static mut IOAPIC: Unique<IoApic> = unsafe{ Unique::new_unchecked(IOAPIC_ADDRESS as *mut _) };

// IO APIC MMIO structure: write reg, then read or write data.
#[repr(C)]
struct IoApic {
	reg: Mmio<u32>,
	pad: [Mmio<u32>; 3],
	data: Mmio<u32>,
}

impl IoApic {
	unsafe fn read(&mut self, reg: u8) -> u32
	{
		self.reg.write(reg as u32);
		self.data.read()
	}
	unsafe fn write(&mut self, reg: u8, data: u32)
	{
		self.reg.write(reg as u32);
		self.data.write(data);
	}
	unsafe fn write_irq(&mut self, irq: u8, flags: RedirectionEntry, dest: u8)
	{
		self.write(REG_TABLE+2*irq, (T_IRQ0 + irq) as u32 | flags.bits());
		self.write(REG_TABLE+2*irq+1, (dest as u32) << 24);
	}
	unsafe fn enable(&mut self, irq: u8, cpunum: u8)
	{
		// Mark interrupt edge-triggered, active high,
		// enabled, and routed to the given cpunum,
		// which happens to be that cpu's APIC ID.
		self.write_irq(irq, NONE, cpunum);
	}
	fn id(&mut self) -> u8 {
		unsafe{ self.read(REG_ID).get_bits(24..28) as u8 }
	}
	fn version(&mut self) -> u8 {
		unsafe{ self.read(REG_VER).get_bits(0..8) as u8 }
	}
	fn maxintr(&mut self) -> u8 {
		unsafe{ self.read(REG_VER).get_bits(16..24) as u8 }
	}
}