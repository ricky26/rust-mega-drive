#![no_std]

use core::ptr::{read_volatile, write_volatile};

pub mod z80;
pub mod vdp;
pub mod ports;
pub mod fm;
pub mod psg;

extern "C" {
    static _data_src: *const u32;
    static _data_start: *mut u32;
    static _data_end: *mut u32;
    static _bss_start: *mut u32;
    static _bss_end: *mut u32;
    static _heap_start: *mut u8;
    static _heap_end: *mut u8;
}

#[no_mangle]
fn _init_runtime() {
    // Implement SEGA copy protection.
    init_tmss();

    // Shutdown the Z80 and set it up for RAM access.
    // This is required to access peripherals on the Z80 bus.
    // More consideration will be needed here when the Z80 is considered for
    // use proper.
    z80::halt(true);
    z80::request_bus(true);
    z80::halt(false);

    // Copy .data & zero .bss.
    unsafe {
        let data_count = _data_end.offset_from(_data_start) as usize;
        let data_src = core::slice::from_raw_parts(_data_src, data_count);
        let data_dest = core::slice::from_raw_parts_mut(_data_start, data_count);
        for (dst, src) in data_dest.into_iter().zip(data_src.into_iter()) {
            *dst = *src;
        }

        let bss_count = _bss_end.offset_from(_bss_start) as usize;
        let bss = core::slice::from_raw_parts_mut(_bss_start, bss_count);
        for b in bss {
            *b = 0;
        }
    }
}

/// Fetch the area of RAM not used by either the stack or statically allocated data.
///
/// This can be used to allocate dynamic memory.
pub unsafe fn heap() -> &'static mut [u8] {
    let len = _heap_end.offset_from(_heap_start) as usize;
    core::slice::from_raw_parts_mut(_heap_start, len)
}

/// An enum for the various region variants of the Mega Drive.
#[derive(Clone, Copy, Debug)]
pub enum Region {
    Invalid,
    Japan,
    Europe,
    USA,
}

/// A struct containing version information extracted from the console.
///
/// This can be used to determine region, resolution, frame rate and hardware
/// revision.
#[derive(Clone, Copy, Debug)]
pub struct Version(u8);

impl Version {
    /// Retrieve the hardware revision.
    pub fn hardware_revision(self) -> u8 { self.0 & 0xf }

    /// Check if a FDD is attached.
    pub fn has_fdd(self) -> bool { (self.0 & 0x20) != 0 }

    /// Returns true if this is a PAL system.
    pub fn is_pal(self) -> bool { (self.0 & 0x40) != 0 }

    /// Returns true if this is a NTSC system.
    pub fn is_ntsc(self) -> bool { !self.is_pal() }

    /// Returns true if this is an 'overseas' model, i.e. not for use in Japan.
    pub fn is_overseas(self) -> bool { (self.0 & 0x80) != 0 }

    /// Return the region variation of this console.
    pub fn region(self) -> Region {
        match (self.is_pal(), self.is_overseas()) {
            (false, false) => Region::Japan,
            (false, true) => Region::USA,
            (true, false) => Region::Europe,
            (true, true) => Region::Invalid,
        }
    }
}

const VERSION_REG: *mut u8 = (0xa10001) as _;

/// Read the console version information.
pub fn version() -> Version {
    let v = unsafe { read_volatile(VERSION_REG) };
    Version(v)
}

// TMSS - copy protection for the Mega Drive.
const TMSS_CODE: &'static [u8; 4] = b"SEGA";
const TMSS_REG: *mut u32 = 0xa14000 as _;

fn init_tmss() {
    if version().hardware_revision() > 0 {
        unsafe {
            let tmss_code: *const u32 = core::mem::transmute(&TMSS_CODE[0]);
            write_volatile(TMSS_REG, *tmss_code);
        }
    }
}
