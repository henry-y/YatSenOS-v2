mod apic;
mod consts;
pub mod clock;
mod serial;
mod syscall;
mod exceptions;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::memory::physical_to_virtual;
use consts::*;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            // TODO: 
            clock::register_idt(&mut idt);
            // TODO: 
            serial::register_idt(&mut idt);

            syscall::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();

    // FIXME: check and init APIC
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    // info!("breakpoint1");
    lapic.cpu_init();
    // FIXME: enable serial irq with IO APIC (use enable_irq)
    enable_irq(Irq::Serial0 as u8, 0);
    
    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
