#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks, abi_x86_interrupt, asm)]
#![test_runner(crate::runner)]
#![reexport_test_harness_main = "test_main"]

pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod serial;
pub mod tests;
pub mod vga;
pub mod bga;

use bootloader::{entry_point, BootInfo};
pub use tests::runner::runner;

pub fn start_kernel(boot_info: &'static BootInfo) -> ! {
    use x86_64::registers::control::Cr3;

    init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_page_table(phys_mem_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            serial_println!("L4 page table entry {}: {:?}", i, entry);
        }
    }

    println!("Hello World from Voluspa!");

    let mut bga_controller = bga::BgaController::init();
    bga_controller.set_res(1024, 768, 0x20);
    bga_controller.write_gibberish();
    //bga_controller.clear_screen(Pixel::new(255, 255, 255));

    hlt_loop()
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    serial_println!("lib::_start");
    init();
    test_main();
    hlt_loop()
}

pub fn init() {
    println!("Voluspa is starting...");

    interrupt::init();
    gdt::init_gdt();

    println!("Voluspa startup sequence complete!");
}

use crate::memory::active_level_4_page_table;
use core::panic::PanicInfo;
use x86_64::VirtAddr;
use crate::bga::Pixel;

pub fn vga_panic_handler(info: &PanicInfo) -> ! {
    use vga::{Color, ColorCode, VgaChar, WRITER};

    // clear the screen with a blue background
    let clear_char = VgaChar::new(b' ', ColorCode::new(Color::White, Color::Blue));
    WRITER.lock().fill_screen(clear_char);

    let text_color = ColorCode::new(Color::Red, Color::Blue);
    WRITER.lock().set_color(text_color);
    WRITER
        .lock()
        .write_row(15, "                                     PANIC");

    let normal_text_color = ColorCode::new(Color::White, Color::Blue);
    WRITER.lock().set_color(normal_text_color);

    println!("{}", info);

    hlt_loop()
}

pub fn serial_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("\n\n\n-- VOLUSPA KERNEL PANIC --");
    serial_println!("{}", info);

    hlt_loop();
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[FAILED]\n");
    serial_println!("Error: {}", info);
    tests::runner::isa_debug_exit_qemu(tests::runner::QemuExitCode::Failure);
    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
