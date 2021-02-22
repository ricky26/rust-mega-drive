#![no_std]

pub mod z80;
pub mod vdp;
pub mod ports;
pub mod fm;

extern "C" {
    static _data_src: *const u32;
    static _data_start: *mut u32;
    static _data_end: *mut u32;
    static _bss_start: *mut u32;
    static _bss_end: *mut u32;
}

#[no_mangle]
fn _init_runtime() {
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
