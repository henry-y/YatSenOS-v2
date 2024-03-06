use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;
use super::super::consts::{Interrupts, Irq};

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.addr + reg as u64) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        write_volatile((self.addr + reg as u64) as *mut u32, value);
        self.read(0x20);
    }
}

use bitflags::bitflags;

bitflags! {
    struct SpuriousInterruptFlags: u32 {
        const ENABLE = 1 << 8;
    }
}

bitflags! {
    struct LvtTimerFlags: u32 {
        const MASK = 1 << 16;
        const PERIODIC_MODE = 1 << 17;
    }
}

bitflags! {
    struct DivideConfigurationFlags: u32 {
        const DIVIDE_BY_1 = 0b1011;
    }
}

bitflags! {
    struct DeliveryStatus: u32 {
        const PENDING = 1 << 12;
    }

    struct InitLevelDeAssertFlags: u32 {
        const BCAST = 1 << 19;
        const INIT = 5 << 8;
        const LEVEL = 1 << 15;
    }
}

bitflags! {
    struct LvtLintFlags: u32 {
        const MASK = 1 << 16;
    }
}

bitflags! {
    struct LvtErrorFlags: u32 {
        const MASK = 1 << 16;
    }
}

bitflags! {
    struct ApicRegister: u32 {
        const SPIV = 0xF0;       // Spurious Interrupt Vector Register
        const LVT_TIMER = 0x320; // Local Vector Table (LVT) Timer Register
        const TIMER_DIVIDE = 0x3E0; // Timer Divide Configuration Register
        const INITIAL_COUNT = 0x380; // Initial Count Register (Timer)
        const LINT0 = 0x350;     // Local Vector Table (LVT) LINT0 Register
        const LINT1 = 0x360;     // Local Vector Table (LVT) LINT1 Register
        const PCINT = 0x340;     // Performance Counter LVT Register
        const ERROR = 0x370;     // LVT Error Register
        const ESR = 0x280;       // Error Status Register
        const EOI = 0xB0;        // End Of Interrupt Register
        const ICR_LOW = 0x300;   // Interrupt Command Register (ICR) Low
        const ICR_HIGH = 0x310;  // Interrupt Command Register (ICR) High
        const TPR = 0x80;        // Task Priority Register (TPR)
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXED: Check CPUID to see if xAPIC is supported.
        CpuId::new().get_feature_info().map(
            |f| f.has_apic()
        ).unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    // fn cpu_init(&mut self) {
    //     unsafe {
    //         info!("checking xAPIC support...");
    //         if !Self::support() {
    //             panic!("xAPIC is not supported.");
    //         }
    //         info!("xAPIC is supported.");

    //         // FIXED: Enable local APIC; set spurious interrupt vector.
    //         let mut spiv = self.read(0xF0);
    //         spiv |= 1 << 8;
    //         spiv &= !(0xFF);
    //         spiv |= Interrupts::IrqBase as u32 + Irq::Spurious as u32;
    //         self.write(0xF0, spiv); 
            
    //         // FIXED: The timer repeatedly counts down at bus frequency
    //         let mut lvt_timer = self.read(0x320);
    //         // clear and set Vector
    //         lvt_timer &= !(0xFF);
    //         lvt_timer |= Interrupts::IrqBase as u32 + Irq::Timer as u32;
    //         lvt_timer &= !(1 << 16); // clear Mask
    //         lvt_timer |= 1 << 17; // set Timer Periodic Mode
    //         self.write(0x320, lvt_timer);
            
    //         // set timer related registers
    //         self.write(0x3E0, 0b1011); // set Timer Divide to 1
    //         self.write(0x380, 0x20000); // set initial count to 0x20000
            
    //         // FIXME: Disable logical interrupt lines (LINT0, LINT1)
    //         self.write(0x350, 1 << 16); // disable LINT0
    //         self.write(0x360, 1 << 16); // disable LINT1
    //         // FIXME: Disable performance counter overflow interrupts (PCINT)
    //         self.write(0x340, 1 << 16); // disable PCINT
            
    //         // FIXME: Map error interrupt to IRQ_ERROR.
    //         let mut lvt_error = self.read(0x370);
    //         lvt_error &= !(0xFF);
    //         lvt_error |= Interrupts::IrqBase as u32 + Irq::Error as u32;
    //         lvt_error &= !(1 << 16); // clear Mask
    //         // timer periodic mode == only
    //         self.write(0x370, lvt_error);
            
    //         // FIXME: Clear error status register (requires back-to-back writes).
    //         self.write(0x280, 0);
    //         self.write(0x280, 0);
            
    //         // FIXED: Ack any outstanding interrupts.
    //         // clear eoi register
    //         self.write(0xB0, 0);

    //         // FIXED: Send an Init Level De-Assert to synchronise arbitration ID's.
    //         self.write(0x310, 0); // set ICR 0x310
    //         const BCAST: u32 = 1 << 19;
    //         const INIT: u32 = 5 << 8;
    //         const TMLV: u32 = 1 << 15; // TM = 1, LV = 0
    //         self.write(0x300, BCAST | INIT | TMLV); // set ICR 0x300
    //         const DS: u32 = 1 << 12;
    //         while self.read(0x300) & DS != 0 {} // wait for delivery status
            
    //         // FIXED: Enable interrupts on the APIC (but not on the processor).
    //         // set TPR to zero
    //         self.write(0x80, 0);
    //     }

        // NOTE: Try to use bitflags! macro to set the flags.

        fn cpu_init(&mut self) {
            unsafe {
                info!("checking xAPIC support...");
                if !Self::support() {
                    panic!("xAPIC is not supported.");
                }
                info!("xAPIC is supported.");
        
                // Enable local APIC; set spurious interrupt vector.
                let mut spiv = self.read(ApicRegister::SPIV.bits());
                spiv |= SpuriousInterruptFlags::ENABLE.bits();
                spiv &= !(0xFF);
                spiv |= Interrupts::IrqBase as u32 + Irq::Spurious as u32;
                self.write(ApicRegister::SPIV.bits(), spiv);
        
                // The timer repeatedly counts down at bus frequency
                let mut lvt_timer = self.read(ApicRegister::LVT_TIMER.bits());
                lvt_timer &= !(0xFF);
                lvt_timer |= Interrupts::IrqBase as u32 + Irq::Timer as u32;
                lvt_timer &= !LvtTimerFlags::MASK.bits();
                lvt_timer |= LvtTimerFlags::PERIODIC_MODE.bits();
                self.write(ApicRegister::LVT_TIMER.bits(), lvt_timer);
        
                // Set timer related registers
                self.write(ApicRegister::TIMER_DIVIDE.bits(), DivideConfigurationFlags::DIVIDE_BY_1.bits());
                self.write(ApicRegister::INITIAL_COUNT.bits(), 0x20000);
        
                // Disable logical interrupt lines (LINT0, LINT1)
                self.write(ApicRegister::LINT0.bits(), LvtLintFlags::MASK.bits());
                self.write(ApicRegister::LINT1.bits(), LvtLintFlags::MASK.bits());
        
                // Disable performance counter overflow interrupts (PCINT)
                self.write(ApicRegister::PCINT.bits(), LvtErrorFlags::MASK.bits());
        
                // Map error interrupt to IRQ_ERROR.
                let mut lvt_error = self.read(ApicRegister::ERROR.bits());
                lvt_error &= !(0xFF);
                lvt_error |= Interrupts::IrqBase as u32 + Irq::Error as u32;
                lvt_error &= !LvtErrorFlags::MASK.bits();
                self.write(ApicRegister::ERROR.bits(), lvt_error);
        
                // Clear error status register (requires back-to-back writes).
                self.write(ApicRegister::ESR.bits(), 0);
                self.write(ApicRegister::ESR.bits(), 0);
        
                // Ack any outstanding interrupts.
                self.write(ApicRegister::EOI.bits(), 0);
        
                // Send an Init Level De-Assert to synchronise arbitration ID's.
                self.write(ApicRegister::ICR_HIGH.bits(), 0); // ICR High - not used in this context
                self.write(ApicRegister::ICR_LOW.bits(), InitLevelDeAssertFlags::BCAST.bits() | InitLevelDeAssertFlags::INIT.bits() | InitLevelDeAssertFlags::LEVEL.bits()); // ICR Low
        
                // Wait for delivery status to clear
                while self.read(ApicRegister::ICR_LOW.bits()) & DeliveryStatus::PENDING.bits() != 0 {}
        
                // Enable interrupts on the APIC (but not on the processor).
                self.write(ApicRegister::TPR.bits(), 0);
            }
        }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
