#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#include "dwarf_addr2line.h"

void print_location(const char* binary, uint64_t address) {
    printf("\nLooking up address 0x%" PRIx64 " in %s\n", address, binary);

    DwarfContext* ctx = dwarf_context_new(binary);
    if (!ctx) {
        const char* error = dwarf_last_error();
        fprintf(stderr, "ERROR: Failed to create context: %s\n",
                error ? error : "unknown error");
        return;
    }

    DwarfLocation loc;
    int result = dwarf_find_location(ctx, address, &loc);

    if (result != 0) {
        const char* error = dwarf_last_error();
        fprintf(stderr, "ERROR: Failed to find location: %s\n",
                error ? error : "unknown error");
    } else if (loc.is_valid && loc.file) {
        printf("  File: %s\n", loc.file);
        printf("  Line: %u\n", loc.line);
        printf("  Column: %u\n", loc.column);
    } else {
        printf("  No debug info found for this address\n");
        printf("  (This address may not contain code, or may lack debug info)\n");
    }

    dwarf_context_free(ctx);
}

int main(int argc, char** argv) {
    printf("DWARF addr2line Test Application\n");
    printf("=================================\n");

    if (argc < 2) {
        fprintf(stderr, "Usage: %s <elf_file> [address1] [address2] ...\n", argv[0]);
        fprintf(stderr, "Example: %s ./my_program 0x1234 0x5678\n", argv[0]);
        return 1;
    }

    const char* binary_path = argv[1];
    printf("Binary: %s\n", binary_path);

    if (argc == 2) {
        // No addresses provided, just test loading the file
        printf("\nTesting file loading...\n");
        DwarfContext* ctx = dwarf_context_new(binary_path);
        if (!ctx) {
            const char* error = dwarf_last_error();
            fprintf(stderr, "ERROR: Failed to load file: %s\n",
                    error ? error : "unknown error");
            return 1;
        }
        printf("SUCCESS: File loaded and DWARF data parsed\n");
        dwarf_context_free(ctx);

        printf("\nTo look up addresses, provide them as additional arguments.\n");
        printf("\nFirst, find function addresses using 'nm':\n");
        printf("  nm %s | grep ' T '\n", binary_path);
        printf("\nThen use those addresses:\n");
        printf("  %s %s 0x<address1> 0x<address2>\n", argv[0], binary_path);
    } else {
        // Addresses provided, look them up
        for (int i = 2; i < argc; i++) {
            uint64_t address;
            if (sscanf(argv[i], "0x%" PRIx64, &address) == 1 ||
                sscanf(argv[i], "%" PRIu64, &address) == 1) {
                print_location(binary_path, address);
            } else {
                fprintf(stderr, "WARNING: Invalid address format: %s\n", argv[i]);
            }
        }
    }

    printf("\nDone!\n");
    return 0;
}
