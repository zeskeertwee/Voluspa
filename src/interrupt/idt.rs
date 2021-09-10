use super::pic;
use crate::{gdt, hlt_loop, print, println};
use lazy_static::lazy_static;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        pic::init_idt_interrupt_handlers(&mut idt);

        idt
    };
}

pub fn init() {
    print!("Initializing IDT...   ");

    IDT.load();

    println!("[Ok]");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed adress: {:?}", Cr2::read());
    println!("Error code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn breakpoint_exception_handling() {
    x86_64::instructions::interrupts::int3();
}
