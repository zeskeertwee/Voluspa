use crate::{print, println};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::{interrupts, port::Port};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
const KEYBOARD_SCANCODE_PORT: u16 = 0x60;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
}

pub fn init() {
    print!("Initializing PIC...   ");

    unsafe {
        PICS.lock().initialize();
    }

    // enable external interrupts
    interrupts::enable();

    println!("[Ok]")
}

pub(super) fn init_idt_interrupt_handlers(idt: &mut InterruptDescriptorTable) {
    idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // implicitly gets value of Timer + 1
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();

    let mut port = Port::<u8>::new(KEYBOARD_SCANCODE_PORT);
    let scancode = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(keycode) => print!("{:?}", keycode),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
