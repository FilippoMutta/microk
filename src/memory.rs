use heapless::{
    Vec,
    consts::U512,
};
use bootloader_api::{
    BootInfo,
    info::{MemoryRegion, MemoryRegionKind},
};

use crate::println;

pub struct MemoryManager {
    pub total_size: u64,
    pub physical_memory_offset: u64,
    pub regions: heapless::Vec<MemoryRegion, U512>,
}

impl MemoryManager {
    pub fn init(boot_info: &mut BootInfo) -> MemoryManager {
        let physical_memory_offset = boot_info
            .physical_memory_offset
            .into_option()
            .expect("The bootloader should map all physical memory for us");

        if boot_info.memory_regions.len() > 512 {
            panic!("Too much memory!");
        }

        println!("Physical memory offset: {physical_memory_offset:#018x}");

        let total_size = boot_info.memory_regions.iter().map(|memory_regions| memory_regions.end - memory_regions.start).sum();
        println!("Memory total size: {}MB", total_size / (1024 * 1024));

        boot_info
            .memory_regions
            .sort_unstable_by_key(|memory_regions| memory_regions.start);
        
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

        println!("Catalouging memory regions...");

        let mut regions: heapless::Vec<MemoryRegion, U512> = Vec::new();

        for region in boot_info
            .memory_regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
        {
            regions.push(*region);
        }
        
        MemoryManager {
            total_size,
            physical_memory_offset,
            regions,
        }

    }

    pub fn alloc(&mut self, size: usize) -> Result<*mut u8, &'static str> {
        // TODO: Implement memory deallocation here
        unimplemented!("Alloc is not yet implemented");
    }

    pub fn dealloc(&mut self, ptr: *mut u8) -> Result<(), &'static str> {
        // TODO: Implement memory deallocation here
        unimplemented!("Dealloc is not yet implemented");
    }
}
