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
    memory::{
        MemoryManager,
    },
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
    println!("Started framebuffer.");

    println!("Loading the GDT...");
    mutta_os::gdt::init();
    println!("GDT loaded.");

    println!("Loading interrupts...");
    mutta_os::interrupts::init_idt();
    x86_64::instructions::interrupts::enable();
    println!("Interrupts loaded.");
   
    println!("Parsing memory...");
    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
    
    let mut mem_manager = MemoryManager::init(boot_info);

    // Allocate some memory
    //let ptr = mem_manager.alloc(1024).unwrap();

    // Deallocate the memory
    //mem_manager.dealloc(ptr).unwrap();
    // Merge contiguous memory regions of the same kind and log them.
    
    println!("Done parsing memory.");
    
    println!("Starting memory managment...");
    // TODO: Memory managment here
    println!("Memory managment initialized.");

    println!("Loading ACPI...");
    let rsdp_addr = boot_info
        .rsdp_addr
        .into_option()
        .expect("The bootloader should give us the address to ACPI rsdp.");
    println!("RSDP addr: {:#018x}", rsdp_addr);
    println!("ACPI initialized.");

    // PCI, SMP,XHCI, USB all TODO

    println!("Bootup complete.");

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

    mutta_os::hlt_loop();
}
