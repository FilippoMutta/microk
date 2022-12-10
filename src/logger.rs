use core::fmt::{Display, Write};

use bootloader_api::info::FrameBuffer;
use uart_16550::SerialPort;

use crate::graphical;

#[derive(Clone, Copy)]
pub enum Color {
    White,
    Red,
    Green,
    Blue,
    Yellow,
}

static mut SERIAL_PORT: SerialPort = unsafe { SerialPort::new(0x03F8) };

pub fn init(framebuffer: Option<FrameBuffer>) {
    graphical::init_framebuffer(framebuffer);
}

pub fn log(display: impl Display, color: Color) {
    graphical::write(&display, color);

    unsafe {
        write!(SERIAL_PORT, "{}{display}\r\n", color.escape_sequence()).unwrap();
    }
}

pub fn printk(display: impl Display) {
    graphical::write(&display, Color::White);

    unsafe {
        write!(SERIAL_PORT, "{}{display}\r\n", Color::White.escape_sequence()).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logger::printk(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

impl Color {
    fn escape_sequence(&self) -> &'static str {
        match self {
            Color::White => "\x1b[m\x1b[97m",
            Color::Red => "\x1b[m\x1b[91m",
            Color::Green => "\x1b[m\x1b[92m",
            Color::Blue => "\x1b[m\x1b[94m",
            Color::Yellow => "\x1b[m\x1b[93m",
        }
    }
}
