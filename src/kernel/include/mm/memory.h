#pragma once
#include <stdint.h>
#include <stddef.h>
#include <mm/efimem.h>

uint64_t get_memory_size(EFI_MEMORY_DESCRIPTOR *mMap, uint64_t mMapEntries, uint64_t mMapDescSize);
extern "C" void memset(void *start, uint8_t value, uint64_t num);
extern "C" void memcpy(void *dest, void *src, size_t n);
