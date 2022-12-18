use heapless::{
    Vec,
    consts::U128,
};
use bootloader_api::{
    BootInfo,
    info::{MemoryRegion, MemoryRegionKind, Optional},
};

use crate::println;

const MAX_REGIONS: usize = 128;

pub struct MemoryManager {
    memory_regions: usize,
    regions: heapless::Vec<MemoryRegion, U128>,
    total_size: usize,
}

impl MemoryManager {
    pub fn init(boot_info: &mut BootInfo) -> MemoryManager {
        let physical_memory_offset = boot_info
            .physical_memory_offset
            .into_option()
            .expect("The bootloader should map all physical memory for us");
        
        println!("Physical memory offset: {physical_memory_offset:#018x}");

        let memory_regions = boot_info.memory_regions.len();
        if memory_regions > MAX_REGIONS {
            panic!("Too much memory!");
        }
        println!("Memory regions: {}", memory_regions);

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
                core::ptr::write_bytes(addr as *mut u8, 0x00, size as usize);
            }
        }
        /*let regions = boot_info
            .memory_regions
            .iter()
            .map(|r| MemoryRegion {
                start: r.start_addr,
                size: r.size,
            })
            .collect();
        let total_size = regions.iter().map(|r| r.size).sum();
                MemoryManager {
            memory_regions,
            regions,
            total_size
        }*/


        let regions: heapless::Vec<MemoryRegion, U128> = Vec::new();

        MemoryManager {
            memory_regions: 0,
            regions,
            total_size: 0,
        }
    }

    pub fn alloc(&mut self, size: usize) -> Result<*mut u8, &'static str> {
        // TODO: Implement memory allocation here
        unimplemented!();
    }

    pub fn dealloc(&mut self, ptr: *mut u8) -> Result<(), &'static str> {
        // TODO: Implement memory deallocation here
        unimplemented!();
    }
}
