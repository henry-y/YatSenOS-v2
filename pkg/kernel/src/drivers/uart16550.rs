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

bitflags! {
    struct LineControlFlags: u8 {
        const ENABLE_DLAB = 0x80;
    }
}

bitflags! {
    struct FifoControlFlags: u8 {
        const ENABLE_FIFO    = 0x01;
        const CLEAR_RX_FIFO  = 0x02;
        const CLEAR_TX_FIFO  = 0x04;
        const ENABLE_64_BYTE_FIFO = 0x20;
        const DEFAULT = Self::ENABLE_FIFO.bits() | Self::CLEAR_RX_FIFO.bits() | Self::CLEAR_TX_FIFO.bits() | Self::ENABLE_64_BYTE_FIFO.bits();
    }
}

bitflags! {
    struct ModemControlFlags: u8 {
        const DATA_TERMINAL_READY = 0x01;
        const REQUEST_TO_SEND     = 0x02;
        const AUX_OUTPUT1         = 0x04;
        const AUX_OUTPUT2         = 0x08;
        const LOOPBACK_MODE       = 0x10;
        const AUTOFLOW_CONTROL    = 0x20;
    }
}

bitflags! {
    struct InterruptEnableFlags: u8 {
        const RECEIVED_DATA_AVAILABLE = 0x01;
    }
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
    // pub fn init(&mut self) {
    //     // FIXME: Initialize the serial port
    //     unsafe {
            
    //         self.port_int_en.write(0x00);
    //         self.port_line_ctrl.write(0x80);
    //         self.port_data.write(0x03);
    //         self.port_int_en.write(0x00);

    //         self.port_line_ctrl.write(0x03);

    //         self.port_fifo_ctrl.write(0xC7);

    //         self.port_modem_ctrl.write(0x0B);
            
    //         self.port_modem_ctrl.write(0x1E);
    //         self.port_data.write(0xAE);

    //         if self.port_data.read() != 0xAE {
    //             panic!("Serial port not found");
    //         }

    //         self.port_modem_ctrl.write(0x0F);
    //         self.port_int_en.write(0x01);
    //     }
    // }

    pub fn init(&mut self) {
        unsafe {
            self.port_int_en.write(InterruptEnableFlags::empty().bits());
            self.port_line_ctrl.write(LineControlFlags::ENABLE_DLAB.bits());
            self.port_data.write(0x03); // 这里假设0x03代表某种特定的波特率配置
            self.port_int_en.write(InterruptEnableFlags::empty().bits());
    
            self.port_line_ctrl.write(0x03); // 假设这是8位无奇偶校验，1个停止位的配置
    
            self.port_fifo_ctrl.write(FifoControlFlags::DEFAULT.bits());
    
            self.port_modem_ctrl.write(ModemControlFlags::DATA_TERMINAL_READY.bits() | ModemControlFlags::REQUEST_TO_SEND.bits() | ModemControlFlags::AUX_OUTPUT1.bits());
            
            self.port_modem_ctrl.write(ModemControlFlags::LOOPBACK_MODE.bits());
            self.port_data.write(0xAE); // 测试用的数据写入
    
            if self.port_data.read() != 0xAE {
                panic!("Serial port not found");
            }
    
            self.port_modem_ctrl.write(ModemControlFlags::DATA_TERMINAL_READY.bits() | ModemControlFlags::REQUEST_TO_SEND.bits() | ModemControlFlags::AUX_OUTPUT2.bits());
            self.port_int_en.write(InterruptEnableFlags::RECEIVED_DATA_AVAILABLE.bits());
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        unsafe {
            // Wait until the port is ready to send
            while (PortGeneric::read(&mut self.port_line_sts) & 0x20) == 0 {}

            // Send the byte
            PortGeneric::write(&mut self.port_data, data);
        }
        
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        unsafe {
            // trace!("receive something in uart");
            // Check if the port has data to receive
            if (self.port_line_sts.read() & 0x01) == 0 {
                return None;
            }

            // Receive the byte
            Some(self.port_data.read())
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
