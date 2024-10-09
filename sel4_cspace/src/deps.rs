use sel4_common::structures_gen::cap;
use crate::structures::finaliseCap_ret;
use sel4_common::structures::exception_t;

extern "C" {
    pub fn finaliseCap(cap: &cap, _final: bool, _exposed: bool) -> finaliseCap_ret;

    pub fn post_cap_deletion(cap: &cap);

    pub fn preemptionPoint() -> exception_t;
}
