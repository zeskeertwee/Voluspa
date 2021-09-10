const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER_WIDTH: usize = 80;
const VGA_SQUARE_ASCII_CODE: u8 = 0xfe;
const DEFAULT_COLOR_CODE: ColorCode = ColorCode::new(Color::White, Color::Black);

use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::init());
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // to ensure exact same memory layout as u8
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct VgaChar {
    ascii_code: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<VgaChar>; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl VgaChar {
    pub fn new(ascii_code: u8, color: ColorCode) -> Self {
        Self {
            ascii_code,
            color_code: color,
        }
    }
}

impl Writer {
    /// Initializes the writer with the buffer on `0xb8000`
    pub fn init() -> Self {
        Self {
            column_position: 0,
            color_code: DEFAULT_COLOR_CODE,
            buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if is_valid_ascii(&byte) {
                self.write_char(byte);
            } else {
                self.write_char(VGA_SQUARE_ASCII_CODE);
            }
        }
    }

    /// write a single character to the VGA buffer
    ///  - Does not check if the character is valid ASCII
    ///  - Starts a new line if VGA_BUFFER_WIDTH is exceeded
    ///  - Uses the color set with [set_color](Writer::set_color)
    pub fn write_char(&mut self, ascii_code: u8) {
        match ascii_code {
            b'\n' => self.new_line(),
            ascii_code => {
                if self.column_position >= VGA_BUFFER_WIDTH {
                    self.new_line();
                }

                let row = VGA_BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(VgaChar {
                    ascii_code: ascii_code,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..VGA_BUFFER_HEIGHT {
            for col in 0..VGA_BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        let blank = VgaChar {
            ascii_code: b' ',
            color_code: self.color_code,
        };

        self.fill_row(VGA_BUFFER_HEIGHT - 1, blank);
        self.column_position = 0;
    }

    /// fill the enite buffer with a single character
    pub fn fill_screen(&mut self, char: VgaChar) {
        for row in 0..VGA_BUFFER_HEIGHT {
            self.fill_row(row, char);
        }
    }

    pub fn fill_row(&mut self, row: usize, char: VgaChar) {
        for col in 0..VGA_BUFFER_WIDTH {
            self.buffer.chars[row][col].write(char);
        }
    }

    /// write a string to a specific row
    pub fn write_row(&mut self, row: usize, s: &str) {
        let mut len = s.len();
        if len > VGA_BUFFER_WIDTH {
            len = VGA_BUFFER_WIDTH;
        }

        for col in 0..len {
            let char = VgaChar {
                ascii_code: s.chars().nth(col).unwrap_or(VGA_SQUARE_ASCII_CODE as char) as u8,
                color_code: self.color_code,
            };
            self.buffer.chars[row][col].write(char);
        }
    }

    pub fn set_color(&mut self, color: ColorCode) {
        self.color_code = color;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[inline]
const fn is_valid_ascii(code: &u8) -> bool {
    match code {
        0x20..=0x7e | b'\n' => true,
        _ => false,
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // prevent a deadlock
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[test_case]
fn println_overflow() {
    for _ in 0..200 {
        println!("Overflow time");
    }
}

#[test_case]
fn too_long_line() {
    let s =
        "Some test string thats definitly too long for a single line and wont fit on a single line";

    assert!(s.len() > VGA_BUFFER_WIDTH);

    println!("{}", s);
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    // prevent anything else from printing (such as interrupts)
    let lock = WRITER.lock();
    for (i, c) in s.chars().enumerate() {
        let screen_char = lock.buffer.chars[VGA_BUFFER_HEIGHT - 2][i].read();
        assert_eq!(screen_char.ascii_code as char, c);
    }
}
