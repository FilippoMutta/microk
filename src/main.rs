#![no_std]
#![no_main]

use bootloader_api::{
    config::Mapping,
    entry_point,
    info::{MemoryRegionKind, Optional},
    BootInfo, BootloaderConfig,
};
use mutta_os::{
    println,
};

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(Mapping::Dynamic);

    config
};

entry_point!(main, config = &CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = core::mem::replace(&mut boot_info.framebuffer, Optional::None);
    let framebuffer = framebuffer.into_option();
    mutta_os::logger::init(framebuffer);
    mutta_os::gdt::init();
    mutta_os::interrupts::init_idt();
    x86_64::instructions::interrupts::enable();

    println!(" __  __  _                _  __        ___   ___ ");
    println!("|  \\/  |(_) __  _ _  ___ | |/ /       / _ \\ / __|");
    println!("| |\\/| || |/ _|| '_|/ _ \\|   <       | (_) |\\__ \\");
    println!("|_|  |_||_|\\__||_|  \\___/|_|\\_\\       \\___/ |___/");
    println!("MicroKernel version {}.{}.{} {}", 0, 1, 0, "alpha");


    let prelease_str = if boot_info.api_version.pre_release() {
        "(prerelease)"
    } else {
        ""
    };

    println!(
            "Bootloader version: {}.{}.{} {}",
            boot_info.api_version.version_major(),
            boot_info.api_version.version_minor(),
            boot_info.api_version.version_patch(),
            prelease_str
    );
    
    let physical_memory_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("The bootloader should map all physical memory for us");
    println!("Physical memory offset: {physical_memory_offset:#018x}");

    println!("Memory regions: {}", boot_info.memory_regions.len());

    // Merge contiguous memory regions of the same kind and log them.
    boot_info
        .memory_regions
        .sort_unstable_by_key(|region| region.start);
    let mut iter = boot_info.memory_regions.iter().copied();
    if let Some(mut prev) = iter.next() {
        for next in iter {
            if prev.end != next.start || prev.kind != next.kind {
                println!("{:#018x} - {:#018x}: {:?}", prev.start, prev.end, prev.kind);

                prev = next;
            } else {
                prev.end = next.end;
            }
        }

        println!("{:#018x} - {:#018x}: {:?}", prev.start, prev.end, prev.kind);
    }

    println!("Writing to usable memory regions");

    for region in boot_info
        .memory_regions
        .iter()
        .filter(|region| region.kind == MemoryRegionKind::Usable)
    {
        let addr = physical_memory_offset + region.start;
        let size = region.end - region.start;
        unsafe {
            core::ptr::write_bytes(addr as *mut u8, 0xff, size as usize);
        }
    }

    println!("Done!");

    loop {}
}
