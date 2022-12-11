#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod graphical;
pub mod logger;
pub mod interrupts;
pub mod gdt;
pub mod memory;

use core::panic::PanicInfo;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    use crate::logger::{log, Color};
    log("! PANIK !", Color::Red);
    log(format_args!("{info}"), Color::Red);
    log("[Hanging now...]", Color::Red);

    hlt_loop();
}
