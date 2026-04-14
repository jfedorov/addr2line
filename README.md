# DWARF addr2line Prototype

A minimal prototype demonstrating how to use Rust's `gimli` and `addr2line` crates to parse DWARF debug information from ELF files, exposed via a C FFI interface.

## Project Structure

```
.
├── dwarf_addr2line/          # Rust library (shared object)
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs           # FFI implementation
│   └── target/release/
│       ├── libdwarf_addr2line.so  (5.8 MB)
│       └── libdwarf_addr2line.a   (15 MB)
├── dwarf_addr2line.h         # C header file
├── test_addr2line.c          # C application
├── CMakeLists.txt            # Build configuration
└── sample.c                  # Test program with debug info
```

## Components

### 1. Rust Library (`dwarf_addr2line`)

**Dependencies:**
- `addr2line` 0.21 - High-level DWARF address-to-line translation
- `gimli` 0.28 - Low-level DWARF parser (zero-copy, lazy parsing)
- `object` 0.32 - ELF/Mach-O/PE binary parser

**API Functions:**
```c
DwarfContext* dwarf_context_new(const char* path);
int dwarf_find_location(DwarfContext* ctx, uint64_t address, DwarfLocation* location);
const char* dwarf_last_error(void);
void dwarf_context_free(DwarfContext* ctx);
void dwarf_free_string(char* s);
```

### 2. C Application (`test_addr2line`)

A command-line tool that demonstrates the library usage:
```bash
./test_addr2line <elf_file> [address1] [address2] ...
```

## Build Instructions

### Prerequisites
- Rust toolchain (tested with 1.75.0)
- CMake 3.14+
- GCC or Clang
- CARGO_HOME set to `/localdisk/juf/.cargo` (for this environment)

### Build Steps

1. **Build Rust library:**
```bash
export CARGO_HOME=/localdisk/juf/.cargo
cd dwarf_addr2line
cargo build --release
```

2. **Build C application:**
```bash
cd ..
cmake -B build
cmake --build build
```

## Usage Example

### Create a test binary with debug info:
```bash
gcc -g -O0 -o sample sample.c
```

### Find function addresses:
```bash
nm sample | grep -E "(add|multiply|main)"
```

Output:
```
0000000000001149 T add
0000000000001161 T multiply
0000000000001178 T main
```

### Look up addresses:
```bash
./build/test_addr2line ./sample 0x1149 0x1161 0x1178
```

Output:
```
DWARF addr2line Test Application
=================================
Binary: ./sample

Looking up address 0x1149 in ./sample
  File: /localdisk/juf/try_gimli/sample.c
  Line: 3
  Column: 23

Looking up address 0x1161 in ./sample
  File: /localdisk/juf/try_gimli/sample.c
  Line: 7
  Column: 28

Looking up address 0x1178 in ./sample
  File: /localdisk/juf/try_gimli/sample.c
  Line: 11
  Column: 12

Done!
```

## Library Sizes

- **Shared library (`.so`):** 5.8 MB
- **Static library (`.a`):** 15 MB

Both contain:
- Rust FFI code
- addr2line crate
- gimli crate (DWARF parser)
- object crate (binary parser)
- Rust standard library

## Key Features

✅ **Zero-copy parsing** - gimli doesn't copy the input data
✅ **Lazy parsing** - Only parses what's needed
✅ **Cross-platform** - Supports ELF, Mach-O, PE formats
✅ **Thread-safe** - Thread-local error handling
✅ **C-compatible ABI** - Easy integration with C/C++ projects

## Runtime Requirements

The application only needs:
- `libc.so.6` (always present)
- `libm.so.6` (math library - always present)
- `libpthread.so.0` (threading - always present)
- `libdl.so.2` (dynamic loading - always present)

**No Rust runtime or compiler needed at runtime!**

## Error Handling

The library uses thread-local storage for error messages:
```c
DwarfContext* ctx = dwarf_context_new("invalid.elf");
if (!ctx) {
    const char* error = dwarf_last_error();
    fprintf(stderr, "Error: %s\n", error);
}
```

## Memory Management

- **Context lifetime:** User must call `dwarf_context_free()`
- **String ownership:** File paths in `DwarfLocation` are leaked (simplified for prototype)
- **Production consideration:** Would need better string lifetime management

## Performance Characteristics

- **First lookup:** Parses DWARF sections (10-50ms for typical binaries)
- **Subsequent lookups:** Fast (cached context)
- **Memory:** Holds entire binary in memory (can be optimized with mmap)

## Integration with PTI SDK

This prototype demonstrates the feasibility of adding DWARF addr2line support to the PTI SDK for GPU binary symbolication.

**Potential integration points:**
1. `src/levelzero/ze_collector.h` (populate `source_file_name_` and `source_line_number_`)
2. New utility class: `src/utils/gpu_dwarf_resolver.{h,cc}`
3. Optional CMake feature: `PTI_ENABLE_DWARF_ADDR2LINE`

## Next Steps for Production

1. **Better string management** - Use string pool or arena allocator
2. **Inline function support** - `find_frames()` for inlined functions
3. **Symbol caching** - Cache frequently looked-up addresses
4. **Error recovery** - Graceful handling of corrupted DWARF
5. **GPU binary support** - Test with Intel GPU ELF binaries (`.spv`, `.bin`)
6. **Address translation** - Map GPU virtual addresses to DWARF addresses

## License

This prototype uses:
- `gimli` - Apache-2.0 / MIT
- `addr2line` - Apache-2.0 / MIT
- `object` - Apache-2.0 / MIT

All dependencies are dual-licensed under Apache-2.0 and MIT.
