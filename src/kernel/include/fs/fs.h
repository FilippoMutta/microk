#pragma once
#include <dev/pci/pci.h>
#include <dev/ahci/ahci.h>
#include <fs/fat/fat.h>
#include <stddef.h>

namespace Filesystem {
        enum DriveType {
                AHCI,
        };

        enum PartitionType {
                MBR,
                GPT,
        };

        enum Filesystem {
                FAT,
                UNKNOWN,
        };

        struct Partition {
                uint8_t partition_number;
                Filesystem filesystem;
                FAT::FATFSDriver fatDriver;
        };
        
        struct AHCIDriver {
                AHCI::Port *port;
                uint8_t port_number;
                uint32_t buffer_size;
        };

        union Driver {
                AHCIDriver ahciDriver;
        };

        struct Drive {
                DriveType driveType;
                Driver driver;
                PartitionType partitionType;
                Partition partitions[128];
        };

        class FSManager {
        public:
                FSManager();
                void InstallDrive();
                void ListDrives();
                void AddAHCIDrive(AHCI::Port *port, int number, uint32_t buffer_size);
                // We need to improove this with cache
                uint8_t *ReadDrive(uint8_t drive_number, uint32_t start_sector, uint8_t number_sectors);
                bool WriteDrive(uint8_t drive_number, uint32_t start_sector, uint8_t number_sectors, uint8_t *buffer, size_t buffer_size);
                Drive supportedDrives[32];
                uint16_t total_drives;
        };
}

extern Filesystem::FSManager GlobalFSManager;
