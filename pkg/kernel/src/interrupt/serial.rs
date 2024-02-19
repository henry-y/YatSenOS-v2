use crate::{drivers::input::push_key, serial::get_serial_for_sure};
use super::consts::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as usize + Irq::Serial0 as usize]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    // trace!("step receive function"); 
    let mut uart = get_serial_for_sure();
    let c = uart.receive();
    drop(uart);
    if c != None {
        push_key(c.unwrap() as u8);
        print!("{}", c.unwrap() as char);
        // trace!("receive: {}", c.unwrap());
    }
    // if let Some(c) = get_serial_for_sure().receive() {
    //     push_key(c);
    //     trace!("receive: {}", c);
    // }
}