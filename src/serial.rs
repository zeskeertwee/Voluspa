use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

const SERIAL_PORT_1_ADDRESS: u16 = 0x3f8;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_PORT_1_ADDRESS) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    interrupts::without_interrupts(|| match SERIAL1.lock().write_fmt(args) {
        Ok(_) => (),
        Err(e) => panic!("Writing to SERIAL1 failed: {}", e),
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n\r"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n\r")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n\r"), $($arg)*));
}
