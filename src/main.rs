#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(voluspa_kernel::runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use voluspa_kernel;

#[cfg(not(test))]
entry_point!(kernel_main);

#[cfg(test)]
entry_point!(test_kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    voluspa_kernel::start_kernel(boot_info)
}

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    voluspa_kernel::init();
    test_main();
    voluspa_kernel::hlt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    voluspa_kernel::serial_panic_handler(info);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    voluspa_kernel::test_panic_handler(info)
}
