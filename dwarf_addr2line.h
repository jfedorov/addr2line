#ifndef DWARF_ADDR2LINE_H
#define DWARF_ADDR2LINE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque context handle
typedef struct DwarfContext DwarfContext;

// Location result
typedef struct {
    const char* file;      // File path (owned by context, do not free)
    uint32_t line;         // Line number (0 if not available)
    uint32_t column;       // Column number (0 if not available)
    int is_valid;          // 1 if location found, 0 otherwise
} DwarfLocation;

/**
 * Create a new DWARF context from an ELF file path.
 *
 * @param path Path to the ELF file
 * @return Pointer to context, or NULL on error
 */
DwarfContext* dwarf_context_new(const char* path);

/**
 * Find source location for a given address.
 *
 * @param ctx DWARF context
 * @param address Address to look up
 * @param location Output location structure
 * @return 0 on success, -1 on error
 */
int dwarf_find_location(DwarfContext* ctx, uint64_t address, DwarfLocation* location);

/**
 * Get the last error message.
 *
 * @return Error string, or NULL if no error
 */
const char* dwarf_last_error(void);

/**
 * Free the DWARF context.
 *
 * @param ctx Context to free
 */
void dwarf_context_free(DwarfContext* ctx);

/**
 * Free a string returned by dwarf_find_location.
 *
 * @param s String to free
 */
void dwarf_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif // DWARF_ADDR2LINE_H
