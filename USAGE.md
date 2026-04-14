# Quick Usage Guide

## Finding and Resolving Addresses

### Step 1: Find Function Addresses

Use `nm` to list symbols and their addresses:

```bash
$ nm sample | grep ' T '
0000000000001149 T add
0000000000001178 T main
0000000000001161 T multiply
```

The format is: `<address> <type> <symbol_name>`
- `T` = function in text section (code)
- Address is in hexadecimal

### Step 2: Look Up Source Location

Use the addresses found in step 1:

```bash
$ ./build/test_addr2line sample 0x1149 0x1161 0x1178
```

Output:
```
Looking up address 0x1149 in sample
  File: /localdisk/juf/try_gimli/sample.c
  Line: 3
  Column: 23

Looking up address 0x1161 in sample
  File: /localdisk/juf/try_gimli/sample.c
  Line: 7
  Column: 28

Looking up address 0x1178 in sample
  File: /localdisk/juf/try_gimli/sample.c
  Line: 11
  Column: 12
```

## Why Some Addresses Don't Work

Addresses must:
1. ✅ Be within the `.text` section (where code lives)
2. ✅ Have associated DWARF debug info (compiled with `-g`)
3. ✅ Be actual instruction addresses (not random values)

### Example: Invalid Addresses

```bash
$ ./build/test_addr2line sample 0x1234 0x5678
```

Output:
```
Looking up address 0x1234 in sample
  No debug info found for this address
  (This address may not contain code, or may lack debug info)
```

Why? Check the code section range:

```bash
$ readelf -S sample | grep .text
  [16] .text   PROGBITS   0000000000001060  00001060
       000000000000016b  0000000000000000  AX    0   0  16
```

- `.text` starts at: `0x1060`
- `.text` size: `0x16b` bytes
- `.text` ends at: ~`0x11CB`

So:
- ✅ `0x1149` is inside [0x1060, 0x11CB] → **VALID**
- ❌ `0x1234` is outside [0x1060, 0x11CB] → **INVALID** (not code)
- ❌ `0x5678` is way beyond → **INVALID** (not code)

## Checking Debug Info

Verify a binary has debug info:

```bash
$ readelf -S sample | grep debug
  [28] .debug_aranges
  [29] .debug_info
  [30] .debug_abbrev
  [31] .debug_line
  [32] .debug_str
  [33] .debug_line_str
```

If you don't see `.debug_*` sections, the binary wasn't compiled with `-g`.

## Common Use Cases

### 1. Find All Functions
```bash
nm sample | grep ' T ' | awk '{print $1, $3}'
```

### 2. Look Up Specific Function
```bash
# Get address of 'main'
addr=$(nm sample | grep ' T main$' | awk '{print "0x"$1}')
./build/test_addr2line sample $addr
```

### 3. Look Up Multiple Functions
```bash
./build/test_addr2line sample \
  $(nm sample | grep ' T add$' | awk '{print "0x"$1}') \
  $(nm sample | grep ' T multiply$' | awk '{print "0x"$1}') \
  $(nm sample | grep ' T main$' | awk '{print "0x"$1}')
```

### 4. Get Line Numbers for All Functions
```bash
nm sample | grep ' T ' | while read addr type name; do
  echo "=== $name ==="
  ./build/test_addr2line sample 0x$addr | grep -E "(File|Line)"
done
```

Output:
```
=== add ===
  File: /localdisk/juf/try_gimli/sample.c
  Line: 3
=== main ===
  File: /localdisk/juf/try_gimli/sample.c
  Line: 11
=== multiply ===
  File: /localdisk/juf/try_gimli/sample.c
  Line: 7
```

## Understanding the Output

```
File: /localdisk/juf/try_gimli/sample.c
Line: 3
Column: 23
```

- **File**: Source file path (absolute or relative to compile directory)
- **Line**: Line number where the function/code starts
- **Column**: Column number (character position in line)

These correspond to the source code:

```c
// sample.c
1: #include <stdio.h>
2:
3: int add(int a, int b) {      // Line 3, column 23 = "int add(...)"
4:     return a + b;
5: }
```

## Tips

1. **Always use `nm` first** to find real addresses
2. **Addresses in hex** start with `0x` prefix
3. **Check `.text` range** if addresses don't resolve
4. **Verify debug sections** exist with `readelf -S`
5. **Compile with `-g -O0`** for best debug info

## For GPU Binaries

When working with GPU binaries (Intel GPUs):

1. Extract the kernel binary from Level-Zero module
2. Verify it's in ELF format: `file kernel.bin`
3. Check for DWARF sections: `readelf -S kernel.bin | grep debug`
4. Find kernel function addresses: `nm kernel.bin`
5. Look up addresses as shown above

GPU kernels compiled with `-g` in SYCL/DPC++ will have DWARF debug info.
