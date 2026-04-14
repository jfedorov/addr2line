use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::fs;
use std::cell::RefCell;
use object::{Object, ObjectSection};

// Thread-local error storage
thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

fn set_last_error(err: String) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(CString::new(err).unwrap_or_else(|_| {
            CString::new("Invalid error string").unwrap()
        }));
    });
}

/// Opaque context handle
pub struct DwarfContext {
    context: addr2line::Context<gimli::EndianSlice<'static, gimli::RunTimeEndian>>,
    _file_data: Box<[u8]>, // Keep file data alive
}

/// C-compatible location result
#[repr(C)]
pub struct DwarfLocation {
    pub file: *const c_char,
    pub line: u32,
    pub column: u32,
    pub is_valid: c_int,
}

/// Create a new DWARF context from an ELF file path
#[no_mangle]
pub extern "C" fn dwarf_context_new(path: *const c_char) -> *mut DwarfContext {
    if path.is_null() {
        set_last_error("Null path provided".to_string());
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in path: {}", e));
            return std::ptr::null_mut();
        }
    };

    // Read the file
    let file_data = match fs::read(path_str) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Failed to read file: {}", e));
            return std::ptr::null_mut();
        }
    };

    // Parse the object file
    let file_data_box: Box<[u8]> = file_data.into_boxed_slice();
    let file_data_static: &'static [u8] = unsafe {
        std::slice::from_raw_parts(file_data_box.as_ptr(), file_data_box.len())
    };

    let object_file = match object::File::parse(file_data_static) {
        Ok(file) => file,
        Err(e) => {
            set_last_error(format!("Failed to parse ELF file: {}", e));
            return std::ptr::null_mut();
        }
    };

    // Load DWARF sections
    let endian = if object_file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let load_section = |id: gimli::SectionId| -> Result<gimli::EndianSlice<'static, gimli::RunTimeEndian>, gimli::Error> {
        let data = object_file
            .section_by_name(id.name())
            .and_then(|section| section.uncompressed_data().ok())
            .unwrap_or_else(|| std::borrow::Cow::Borrowed(&[]));

        let data_static: &'static [u8] = unsafe {
            std::slice::from_raw_parts(data.as_ptr(), data.len())
        };
        Ok(gimli::EndianSlice::new(data_static, endian))
    };

    let dwarf = match gimli::Dwarf::load(load_section) {
        Ok(dwarf) => dwarf,
        Err(e) => {
            set_last_error(format!("Failed to load DWARF data: {}", e));
            return std::ptr::null_mut();
        }
    };

    // Create addr2line context
    let context = match addr2line::Context::from_dwarf(dwarf) {
        Ok(ctx) => ctx,
        Err(e) => {
            set_last_error(format!("Failed to create addr2line context: {}", e));
            return std::ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(DwarfContext {
        context,
        _file_data: file_data_box,
    }))
}

/// Find source location for a given address
#[no_mangle]
pub extern "C" fn dwarf_find_location(
    ctx: *mut DwarfContext,
    address: u64,
    location: *mut DwarfLocation,
) -> c_int {
    if ctx.is_null() || location.is_null() {
        set_last_error("Null pointer provided".to_string());
        return -1;
    }

    let ctx = unsafe { &mut *ctx };
    let location = unsafe { &mut *location };

    // Initialize as invalid
    location.file = std::ptr::null();
    location.line = 0;
    location.column = 0;
    location.is_valid = 0;

    // Query the location
    match ctx.context.find_location(address) {
        Ok(Some(loc)) => {
            if let Some(file) = loc.file {
                // Leak the string so it lives until context is freed
                // In production, you'd want to manage this better
                let file_cstring = match CString::new(file) {
                    Ok(s) => s,
                    Err(_) => {
                        set_last_error("Invalid file path in DWARF data".to_string());
                        return -1;
                    }
                };
                location.file = file_cstring.into_raw();
            }
            location.line = loc.line.unwrap_or(0);
            location.column = loc.column.unwrap_or(0);
            location.is_valid = 1;
            0
        }
        Ok(None) => {
            // No location found for this address
            0
        }
        Err(e) => {
            set_last_error(format!("Failed to find location: {}", e));
            -1
        }
    }
}

/// Get the last error message
#[no_mangle]
pub extern "C" fn dwarf_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    })
}

/// Free the context
#[no_mangle]
pub extern "C" fn dwarf_context_free(ctx: *mut DwarfContext) {
    if !ctx.is_null() {
        unsafe {
            let _ = Box::from_raw(ctx);
        }
    }
}

/// Free a string returned by dwarf_find_location
#[no_mangle]
pub extern "C" fn dwarf_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
