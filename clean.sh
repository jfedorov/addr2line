#!/bin/bash
# Clean all build artifacts

echo "Cleaning build artifacts..."

# Remove CMake build directory
if [ -d "build" ]; then
    echo "  Removing build/ ($(du -sh build 2>/dev/null | cut -f1))"
    rm -rf build
fi

# Remove Rust target directory
if [ -d "dwarf_addr2line/target" ]; then
    echo "  Removing dwarf_addr2line/target/ ($(du -sh dwarf_addr2line/target 2>/dev/null | cut -f1))"
    rm -rf dwarf_addr2line/target
fi

# Remove Cargo.lock (regenerated on build)
if [ -f "dwarf_addr2line/Cargo.lock" ]; then
    echo "  Removing dwarf_addr2line/Cargo.lock"
    rm -f dwarf_addr2line/Cargo.lock
fi

# Remove compiled test binary
if [ -f "sample" ]; then
    echo "  Removing sample binary"
    rm -f sample
fi

echo ""
echo "Clean complete!"
echo "Project size: $(du -sh . 2>/dev/null | cut -f1)"
