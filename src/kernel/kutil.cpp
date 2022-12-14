#include <kutil.h>

KernelInfo kInfo;
IDTR idtr;

void SetIDTGate(void *handler, uint8_t entry_offset, uint8_t type_attr, uint8_t selector) {
        IDTDescEntry *interrupt = (IDTDescEntry*)(idtr.Offset + entry_offset * sizeof(IDTDescEntry));
        interrupt->SetOffset((uint64_t)handler);
        interrupt->type_attr = type_attr;
        interrupt->selector = selector;
}

void PrepareMemory(BootInfo *bootInfo) {
        // Loading the GDT
        GDTDescriptor gdtDescriptor;
        gdtDescriptor.size = sizeof(GDT) - 1;
        gdtDescriptor.offset = (uint64_t)&DefaultGDT;
        init_tss();
        LoadGDT(&gdtDescriptor);

        // Starting printk
        printk_init_serial();
        printk_init_fb(bootInfo->framebuffer, bootInfo->psf1_Font);
        printk("MicroK Loading...\n");

        // Initializing the GlobalAllocator with EFI Memory data
        GlobalAllocator = PageFrameAllocator();
        GlobalAllocator.ReadEFIMemoryMap(bootInfo->mMap, bootInfo->mMapSize, bootInfo->mMapDescSize);
  
        // Locking kernel pages
        kInfo.kernel_size = (uint64_t)&kernel_end - (uint64_t)&kernel_start;
        uint64_t kernel_pages = (uint64_t)kInfo.kernel_size / 4096 + 1;
        GlobalAllocator.LockPages(&kernel_start, kernel_pages);

        // New page table
        PageTable *PML4 = (PageTable*)GlobalAllocator.RequestPage();
        memset(PML4, 0, 0x1000);
        PageTableManager newPageTableManager = PageTableManager(PML4);
        GlobalPageTableManager = newPageTableManager;

        // Identity mapping
        uint64_t mMapEntries = bootInfo->mMapSize / bootInfo->mMapDescSize;
        uint64_t memSize = get_memory_size(bootInfo->mMap, mMapEntries, bootInfo->mMapDescSize);
        for (uint64_t i = 0; i < memSize; i+= 0x1000) {
                GlobalPageTableManager.MapMemory((void*)i, (void*)i);
        }

        // Make sure the framebuffer is in the correct page
        uint64_t fbBase = (uint64_t)bootInfo->framebuffer->BaseAddress;
        uint64_t fbSize = (uint64_t)bootInfo->framebuffer->BufferSize + 0x1000;
        GlobalAllocator.LockPages((void*)fbBase, fbSize / 0x1000 + 1);
        for (uint64_t i = fbBase; i < fbSize; i+=0x1000) {
                GlobalPageTableManager.MapMemory((void*)i, (void*)i);
        }

        // Loading the page table
        asm ("mov %0, %%cr3" : : "r" (PML4));
}

void PrepareInterrupts(BootInfo *bootInfo) {
        // Preparing interrupts
        idtr.Limit = 0x0FFF;
        idtr.Offset = (uint64_t)GlobalAllocator.RequestPage();

        // Insert interrupts
        SetIDTGate((void*)PageFault_handler, 0xE, IDT_TA_InterruptGate, 0x08);
        SetIDTGate((void*)DoubleFault_handler, 0x8, IDT_TA_InterruptGate, 0x08);
        SetIDTGate((void*)GPFault_handler, 0xD, IDT_TA_InterruptGate, 0x08);
        SetIDTGate((void*)KeyboardInt_handler, 0x21, IDT_TA_InterruptGate, 0x08);
        SetIDTGate((void*)MouseInt_handler, 0x2C, IDT_TA_InterruptGate, 0x08);
        SetIDTGate((void*)PITInt_handler, 0x20, IDT_TA_InterruptGate, 0x08);

        // Load the IDT
        asm("lidt %0" : : "m" (idtr));

        // Starting the mouse
        InitPS2Mouse();

        // Setting up the PIC for basic early input and the PIT
        RemapPIC();
        outb(PIC1_DATA, 0b11111000);
        outb(PIC2_DATA, 0b11101111);

        // Enabling maskable interrupts
        asm("sti");
}

void PrepareACPI(BootInfo *bootInfo) {
        ACPI::SDTHeader *xsdt = (ACPI::SDTHeader*)(bootInfo->rsdp->XSDTAddress);

        int entries = (xsdt->Length - sizeof(ACPI::SDTHeader)) / 8;
        printk("Available ACPI tables: %d\n", entries);

        printk("Loading the FADT table...\n");
        ACPI::FindTable(xsdt, (char*)"FADT");

        printk("Loading the MADT table...\n");
        ACPI::FindTable(xsdt, (char*)"MADT");

        printk("Loading the MCFG table...\n");
        ACPI::MCFGHeader *mcfg = (ACPI::MCFGHeader*)ACPI::FindTable(xsdt, (char*)"MCFG");

        printk("Loading the HPET table...\n");
        ACPI::FindTable(xsdt, (char*)"HPET");

        printk("Enumerating PCI devices...\n");
        PCI::EnumeratePCI(mcfg);
}

KernelInfo kinit(BootInfo *bootInfo) {
        // Memory initialization
        PrepareMemory(bootInfo);

        // Clearing framebuffer
        GlobalRenderer.print_set_color(0xf0f0f0f0, 0x0f0f0f0f);
        GlobalRenderer.print_clear();
 
        // Perhaps do it like linux does (1 per cpu core?)
        // We start by rendering one.
        print_image(1);
        printk("\n\n\n\n\n\n\n\n");

        // Init heap
        printk("Initializing the heap...\n");
        InitializeHeap((void*)0x000010000000000, 0x10);
        void *address_one = malloc(0x8000);
        printk("malloc() address: 0x%x\n", (uint64_t)address_one);
        free(address_one);
        printk("free().\n");
        address_one = 0;
        address_one = malloc(0x8000);
        printk("malloc() address: 0x%x\n", (uint64_t)address_one);
        free(address_one);
        printk("free().\n");

        // Interrupt initialization
        PrepareInterrupts(bootInfo);

        // Setting the timer frequency
        PIT::SetFrequency(1);

        // Starting the modules subsystem
        GlobalModuleManager = ModuleManager();

        // Starting the filesystem Manager
        //GlobalFSManager = Filesystem::FSManager();

        // ACPI initialization
        PrepareACPI(bootInfo);

        // Starting a TTY
        GlobalTTY = TTY();
        


        return kInfo;
}
