use sel4_common::structures_gen::{cap, cap_page_table_cap};

use crate::{map_it_pt_cap, pptr_t, vptr_t};

#[no_mangle]
#[link_section = ".boot.text"]
pub fn create_it_pt_cap(vspace_cap: &cap, pptr: pptr_t, vptr: vptr_t, asid: usize) -> cap {
    let capability = cap_page_table_cap::new(asid as u64, pptr as u64, 1, vptr as u64).unsplay();
    map_it_pt_cap(vspace_cap, &capability);
    return capability;
}
