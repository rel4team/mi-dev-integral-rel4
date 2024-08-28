use super::{device::KDEV_BASE, utils::RISCV_GET_LVL_PGSIZE_BITS};
use crate::arch::riscv64::pagetable::{KERNEL_LEVEL2_PAGE_TABLE, KERNEL_ROOT_PAGE_TABLE};
use crate::{pptr_t, pptr_to_paddr, sfence, PTEFlags, PTE, RISCV_GET_PT_INDEX};
use sel4_common::{
    arch::vm_rights_t,
    sel4_config::{seL4_PageBits, RISCVMegaPageBits, RISCVPageBits},
    utils::convert_to_mut_type_ref,
    ROUND_DOWN,
};
use sel4_cspace::arch::cap_t;

#[no_mangle]
#[link_section = ".boot.text"]
pub fn map_kernel_frame(paddr: usize, vaddr: usize, _vm_rights: vm_rights_t) {
    if vaddr >= KDEV_BASE {
        let paddr = ROUND_DOWN!(paddr, RISCV_GET_LVL_PGSIZE_BITS(1));
        unsafe {
            KERNEL_LEVEL2_PAGE_TABLE.map_next_table(RISCV_GET_PT_INDEX(vaddr, 0), paddr, true);
        }
    } else {
        let paddr = ROUND_DOWN!(paddr, RISCV_GET_LVL_PGSIZE_BITS(0));
        unsafe {
            KERNEL_ROOT_PAGE_TABLE.map_next_table(RISCV_GET_PT_INDEX(vaddr, 0), paddr, true);
        }
    }
}

#[no_mangle]
#[link_section = ".boot.text"]
pub fn map_it_pt_cap(_vspace_cap: &cap_t, _pt_cap: &cap_t) {
    let vptr = _pt_cap.get_pt_mapped_address();
    let lvl1pt = convert_to_mut_type_ref::<PTE>(_vspace_cap.get_cap_ptr());
    let pt = _pt_cap.get_cap_ptr();
    let pt_ret = lvl1pt.lookup_pt_slot(vptr);
    let targetSlot = convert_to_mut_type_ref::<PTE>(pt_ret.ptSlot as usize);
    *targetSlot = PTE::new(pptr_to_paddr(pt) >> seL4_PageBits, PTEFlags::V);
    sfence();
}

#[no_mangle]
pub fn map_it_frame_cap(_vspace_cap: &cap_t, _frame_cap: &cap_t) {
    let vptr = _frame_cap.get_frame_mapped_address();
    let lvl1pt = convert_to_mut_type_ref::<PTE>(_vspace_cap.get_cap_ptr());
    let frame_pptr: usize = _frame_cap.get_cap_ptr();
    let pt_ret = lvl1pt.lookup_pt_slot(vptr);

    let targetSlot = convert_to_mut_type_ref::<PTE>(pt_ret.ptSlot as usize);

    *targetSlot = PTE::new(
        pptr_to_paddr(frame_pptr) >> seL4_PageBits,
        PTEFlags::ADUVRWX,
    );
    sfence();
}

#[no_mangle]
#[link_section = ".boot.text"]
pub fn create_mapped_it_frame_cap(
    pd_cap: &cap_t,
    pptr: usize,
    vptr: usize,
    asid: usize,
    use_large: bool,
    _exec: bool,
) -> cap_t {
    let frame_size: usize;
    if use_large {
        frame_size = RISCVMegaPageBits;
    } else {
        frame_size = RISCVPageBits;
    }
    let cap = cap_t::new_frame_cap(
        asid,
        pptr,
        frame_size,
        vm_rights_t::VMReadWrite as usize,
        0,
        vptr,
    );
    map_it_frame_cap(pd_cap, &cap);
    cap
}

pub fn create_unmapped_it_frame_cap(pptr: pptr_t, _use_large: bool) -> cap_t {
    cap_t::new_frame_cap(0, pptr, 0, 0, 0, 0)
}
