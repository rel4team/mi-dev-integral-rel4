use super::boot::map_kernel_frame;
use crate::{paddr_t, pptr_t};
use sel4_common::arch::vm_rights_t::VMKernelOnly;
use sel4_common::{sel4_config::PAGE_BITS, BIT};

pub const KDEV_BASE: usize = 0xFFFFFFFFC0000000;
pub(crate) const NUM_KERNEL_DEVICE_FRAMES: usize = 0;
#[derive(Copy, Clone)]
struct kernel_frame_t {
    paddr: paddr_t,
    pptr: pptr_t,
    userAvailable: isize,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct p_region_t {
    pub start: usize,
    pub end: usize,
}
extern "C" {
    pub(self) fn reserve_region(reg: p_region_t) -> bool;
}

#[no_mangle]
#[link_section = ".boot.text"]
pub(self) static mut kernel_device_frames: [kernel_frame_t; NUM_KERNEL_DEVICE_FRAMES] = [];
#[no_mangle]
pub fn map_kernel_devices() {
    unsafe {
        for kernel_frame in kernel_device_frames {
            map_kernel_frame(kernel_frame.paddr.0, kernel_frame.pptr, VMKernelOnly);
            if kernel_frame.userAvailable == 0 {
                reserve_region(p_region_t {
                    start: kernel_frame.paddr.0,
                    end: kernel_frame.paddr.0 + BIT!(PAGE_BITS),
                });
            }
        }
    }
}
