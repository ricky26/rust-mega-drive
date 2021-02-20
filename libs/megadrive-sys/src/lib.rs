#![no_std]

pub mod vdp;
pub mod ports;

extern "C" {
    static _data_src: *const u32;
    static _data_start: *mut u32;
    static _data_end: *mut u32;
    static _bss_start: *mut u32;
    static _bss_end: *mut u32;
}

#[no_mangle]
fn _init_runtime() {
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