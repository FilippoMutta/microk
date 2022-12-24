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
    print,
    interrupts,
    memory::{
        MemoryManager,
    },
};

const CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    const STACK_SIZE: u64 = 1024 * 256;

    config.kernel_stack_size = STACK_SIZE;
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.aslr = true;

    config
};

entry_point!(main, config = &CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = core::mem::replace(&mut boot_info.framebuffer, Optional::None);
    let framebuffer = framebuffer.into_option();
    mutta_os::logger::init(framebuffer);
    println!("Started framebuffer.");

    print!("Loading the GDT...");
    mutta_os::gdt::init();
    println!("GDT loaded.");

    print!("Loading interrupts...");
    mutta_os::interrupts::init_idt();
    unsafe { interrupts::PICS.lock().disable() };
    x86_64::instructions::interrupts::enable();
    println!("Interrupts loaded.");
   
    println!("Parsing memory...");
    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
    
    let mut mem_manager = MemoryManager::init(boot_info);

/*    for region in mem_manager.regions {
        let addr = mem_manager.physical_memory_offset + region.start;
        let size = region.end - region.start;
        println!("Region! Addr: {:#018x} Size: {:#018x}", addr, size);
    }*/

    // Allocate some memory
//    let ptr = mem_manager.alloc(1024).unwrap();

    // Deallocate the memory
//    mem_manager.dealloc(ptr).unwrap();
    println!("Memory managment initialized.");

    print!("Loading ACPI...");
    let rsdp_addr = boot_info
        .rsdp_addr
        .into_option()
        .expect("The bootloader should give us the address to ACPI rsdp.");
    println!("ACPI initialized.");
    println!("RSDP addr: {:#018x}", rsdp_addr);

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

    println!("Done.");

    mutta_os::hlt_loop();
}
