use core::mem::transmute;
use x86_64::instructions::port::Port;
use volatile::Volatile;

use crate::serial_print;

const VBE_DISPI_IOPORT_INDEX: u16 = 0x01CE;
const VBE_DISPI_IOPORT_DATA: u16 = 0x01CF;

const KB64: u16 = u16::MAX;

pub struct BgaController {
    port_reg: Port<u16>,
    port_data: Port<u16>,
    resolution: (u16,u16),
    n_banks: u16,
}

impl BgaController {
    pub fn init() -> Self {
        let mut controler = Self {
            port_reg: Port::new(VBE_DISPI_IOPORT_INDEX),
            port_data: Port::new(VBE_DISPI_IOPORT_DATA),
            resolution: (640,480),
            n_banks: 0,
        };
        controler.set_res(controler.resolution.0, controler.resolution.1, 0x20);
        //controler.write_gibberish();
        controler
    }

    fn set_lfb_mode(&mut self) {
    }

    pub fn set_res(&mut self, x: u16, y: u16, depth: u16) {
        //disable bga
        self.write_to_reg(0x04, 0x00);
        //set xres
        self.write_to_reg(0x01, x);
        //set yres
        self.write_to_reg(0x02, y);
        //set collor depth to 15 bit
        self.write_to_reg(0x03, depth);
        //enable bga
        self.write_to_reg(0x04, 0x01 | 0x40);

        self.n_banks = (((x as usize) * (y as usize) * (depth as usize)) as f64 / KB64 as f64) as u16;
    }

    pub fn write_to_reg(&mut self, reg: u16, data: u16) {
        unsafe {
            self.port_reg.write(reg);
            self.port_data.write(data);
        }
    }

    pub fn read_from_reg(&mut self, reg: u16) -> u16 {
        unsafe {
            self.port_reg.write(reg);
            return self.port_data.read();
        }
    }

    pub fn write_gibberish(&mut self) {
        for b in 0..self.n_banks {
           let bank = unsafe{
                &mut *(0xA0000 as *mut [Volatile<u32>; (KB64 / 4) as usize])
            };

            self.write_to_reg(0x05, b);
            for i in 0..(KB64 / 4) {
                //bank[i as usize].write(i as u32);
                bank[i as usize].write(Pixel::new(255, 0, 255).into())
            }
        }
        //for x in 0..self.resolution.0 {
        //    for y in 0..self.resolution.1 {
        //        self.set_pixel(x, y, x as u32 >> 16 | y as u32 ).unwrap();
        //    }
        //}
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, pixel: u32) -> Result<(),VbaError> {
        if x > self.resolution.0 {
            return Err(VbaError::PixelOutOfBound);
        }
        if y > self.resolution.1 {
            return Err(VbaError::PixelOutOfBound);
        }
        let offset = x + y * self.resolution.0;
        let bank_num_f = (offset as f32 / (KB64 / 4) as f32);
        let bank_num = unsafe { bank_num_f.to_int_unchecked::<u16>() };
        self.write_to_reg(0x05, bank_num);
        let bank_pixel_pos = offset - bank_num * (KB64 / 4);
        let bank = unsafe{
            &mut *(0xA0000 as *mut [Volatile<u32>; (KB64 / 4) as usize])
        };
        bank[bank_pixel_pos as usize].write(pixel);
        Ok(())
    }

    pub fn clear_screen(&mut self, color: Pixel) {
        for x in 0..self.resolution.0 {
            for y in 0..self.resolution.1 {
                self.set_pixel(x, y, color.into()).unwrap();
            }
        }
    }
}

#[repr(transparent)]
struct Bank {
    bank: [Volatile<u32>; 16384]
}

#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct Pixel {
    b: u8,
    g: u8,
    r: u8,
    pad: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,g,b, pad: 0
        }
    }

    pub fn from_u32(input: u32) -> Self {
        Self { 
            r: input as u8,
            g: (input >> 8) as u8,
            b: (input >> 16) as u8,
            pad: 0,
        }
    }
}

impl Into<u32> for Pixel {
    fn into(self) -> u32 {
        unsafe { transmute(self) }
    }
}

#[derive(Debug)]
pub enum VbaError {
    PixelOutOfBound,
}