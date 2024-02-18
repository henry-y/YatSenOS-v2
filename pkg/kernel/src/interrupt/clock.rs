use super::consts::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::memory::gdt;
use crate::{proc::ProcessContext, proc::switch};

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as usize + Irq::Timer as usize]
        .set_handler_fn(clock_handler)
        .set_stack_index(gdt::CLOCK_INTERRUPT_IST_INDEX as u16);
    // 将新生成的tss栈加入到时钟中断idt中
}

pub extern "C" fn clock(mut context: ProcessContext) {
    crate::proc::switch(&mut context);
    super::ack();
}

as_handler!(clock);

// pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         if inc_counter() % 0x10000 == 0 {
//             // info!("Tick! @{}", read_counter());
//         }
//         super::ack();
//     });
// }

// pub static COUNTER: AtomicU64 = AtomicU64::new(0);

// #[inline]
// pub fn read_counter() -> u64 {
//     // FIXME: load counter value
//     COUNTER.load(Ordering::SeqCst) as u64
// }

// #[inline]
// pub fn inc_counter() -> u64 {
//     // FIXME: read counter value and increase it
//     let val = COUNTER.fetch_add(1, Ordering::SeqCst) as u64;
//     COUNTER.store(val + 1, Ordering::SeqCst);
//     val + 1
// }