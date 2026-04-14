cmake_minimum_required(VERSION 3.14)
project(dwarf_addr2line_test C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Add the test executable
add_executable(test_addr2line test_addr2line.c)

# Include the header directory
target_include_directories(test_addr2line PRIVATE ${CMAKE_SOURCE_DIR})

# Link against the Rust library
target_link_libraries(test_addr2line
    PRIVATE
    ${CMAKE_SOURCE_DIR}/dwarf_addr2line/target/release/libdwarf_addr2line.so
    pthread
    dl
    m
)

# Set RPATH so the executable can find the .so at runtime
set_target_properties(test_addr2line PROPERTIES
    BUILD_RPATH "${CMAKE_SOURCE_DIR}/dwarf_addr2line/target/release"
    INSTALL_RPATH "${CMAKE_SOURCE_DIR}/dwarf_addr2line/target/release"
)

# Print helpful information
message(STATUS "Rust library: ${CMAKE_SOURCE_DIR}/dwarf_addr2line/target/release/libdwarf_addr2line.so")
message(STATUS "Build with: cmake -B build && cmake --build build")
message(STATUS "Run with: ./build/test_addr2line <elf_file> [addresses...]")
