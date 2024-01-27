use core::fmt;

use x86_64::instructions::port::{PortGeneric, ReadWriteAccess, WriteOnlyAccess, ReadOnlyAccess};
/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    port_data: PortGeneric<u8, ReadWriteAccess>,
    port_int_en: PortGeneric<u8, WriteOnlyAccess>,
    port_fifo_ctrl: PortGeneric<u8, WriteOnlyAccess>,
    port_line_ctrl: PortGeneric<u8, WriteOnlyAccess>,
    port_modem_ctrl: PortGeneric<u8, WriteOnlyAccess>,
    port_line_sts: PortGeneric<u8, ReadOnlyAccess>,
}

impl SerialPort {
    
    pub const fn new(port: u16) -> Self {
        Self {
            port_data: PortGeneric::<u8, ReadWriteAccess>::new(port),
            port_int_en: PortGeneric::<u8, WriteOnlyAccess>::new(port + 1),
            port_fifo_ctrl: PortGeneric::<u8, WriteOnlyAccess>::new(port + 2),
            port_line_ctrl: PortGeneric::<u8, WriteOnlyAccess>::new(port + 3),
            port_modem_ctrl: PortGeneric::<u8, WriteOnlyAccess>::new(port + 4),
            port_line_sts: PortGeneric::<u8, ReadOnlyAccess>::new(port + 5),
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) {
        // FIXME: Initialize the serial port
        unsafe {
            
            self.port_int_en.write(0x00);
            self.port_line_ctrl.write(0x80);
            self.port_data.write(0x03);
            self.port_int_en.write(0x00);

            self.port_line_ctrl.write(0x03);

            self.port_fifo_ctrl.write(0xC7);

            self.port_modem_ctrl.write(0x0B);
            self.port_int_en.write(0x01);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        unsafe {
            // Wait until the port is ready to send
            while PortGeneric::read(&mut self.port_line_sts) & 0x20 == 0 {}

            // Send the byte
            PortGeneric::write(&mut self.port_data, data);
        }
        
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        unsafe {
            // Check if the port has data to receive
            if PortGeneric::read(&mut self.port_line_sts) & 0x01 == 0 {
                return None;
            }

            // Receive the byte
            Some(PortGeneric::read(&mut self.port_data))
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
