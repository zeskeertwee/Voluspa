#![no_std]
#![no_main]

use core::panic::PanicInfo;
use voluspa_kernel::tests::{isa_debug_exit_qemu, QemuExitCode};
use voluspa_kernel::{serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    voluspa_kernel::init();

    serial_print!("{}...\t", "stack_overflow::_start");
    stack_overflow();

    // this should not be reached since
    // a interrupt occurs,
    // and since the stack overflow interrupt is not set
    // the stack overflow causes a interrupt
    serial_println!("[FAILED]");
    isa_debug_exit_qemu(QemuExitCode::Failure);

    voluspa_kernel::hlt_loop()
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[Ok]");

    isa_debug_exit_qemu(QemuExitCode::Success);

    voluspa_kernel::hlt_loop()
}
