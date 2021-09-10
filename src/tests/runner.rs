// cargo complains about this being unused because the runner function isn't compiled when not testing
#[allow(unused_imports)]
use crate::{serial_print, serial_println};

pub fn runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    isa_debug_exit_qemu(QemuExitCode::Success);
}

pub trait Testable {
    fn run(&self) -> ();
    fn test_name() -> &'static str
    where
        Self: Sized,
    {
        core::any::type_name::<Self>()
    }
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", T::test_name());
        self();
        serial_println!("[Ok]");
    }
}

#[allow(dead_code)]
pub fn isa_debug_exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}
