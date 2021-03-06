pub mod vga;
pub mod acpi;
pub mod apic;
pub mod mp;
pub mod serial;
pub mod pic;
pub mod keyboard;
pub mod pit;
pub mod ide;

pub fn init(mut page_map: impl FnMut(usize)) -> acpi::AcpiResult {
    assert_has_not_been_called!();

    // TODO Handle this temp page map.
    page_map(0); // EBDA
    for addr in (0xE0000 .. 0x100000).step_by(0x1000) {
        page_map(addr);
    }
    page_map(0x7fe1000); // RSDT

    let acpi = acpi::init().expect("Failed to init ACPI");
    assert_eq!(acpi.lapic_addr as usize, 0xfee00000);
    trace!("acpi = {:?}", acpi);

    if cfg!(feature = "use_apic") {
        pic::disable();

        page_map(0xfee00000);  // LAPIC
        page_map(0xFEC00000);  // IOAPIC

        apic::init(acpi.lapic_addr, acpi.ioapic_id);
    } else {
        pic::init();
    }
    pit::init();
    serial::init();
    keyboard::init();
    acpi
}