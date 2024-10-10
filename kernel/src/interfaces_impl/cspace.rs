use crate::config::CONFIG_MAX_NUM_WORK_UNITS_PER_PREEMPTION;
// use crate::ffi::tcbDebugRemove;
use crate::interrupt::{deletingIRQHandler, isIRQPending, setIRQState, IRQState};
use crate::kernel::boot::current_lookup_fault;
use crate::syscall::safe_unbind_notification;
use sel4_common::sel4_config::{tcbCNodeEntries, tcbCTable, tcbVTable};
use sel4_common::structures::exception_t;
use sel4_common::structures_gen::{cap_Splayed, cap_tag};
use sel4_common::utils::convert_to_mut_type_ref;
use sel4_common::{
    structures_gen::{cap, cap_null_cap},
    utils::ptr_to_mut,
};
use sel4_cspace::capability::cap_pub_func;
use sel4_cspace::compatibility::{ZombieType_ZombieTCB, Zombie_new};
use sel4_cspace::interface::finaliseCap_ret;
use sel4_ipc::{endpoint_t, notification_t, Transfer};
use sel4_task::{get_currenct_thread, ksWorkUnitsCompleted, tcb_t};
#[cfg(target_arch = "riscv64")]
use sel4_vspace::find_vspace_for_asid;
#[cfg(target_arch = "aarch64")]
use sel4_vspace::unmap_page_table;
use sel4_vspace::{asid_pool_t, asid_t, delete_asid, delete_asid_pool, unmapPage, PTE};

#[cfg(target_arch = "riscv64")]
#[no_mangle]
pub fn Arch_finaliseCap(capability: &cap, final_: bool) -> finaliseCap_ret {
    let mut fc_ret = finaliseCap_ret::default();
    match capability.get_cap_type() {
        cap_tag::cap_frame_cap => {
            if capability.get_frame_mapped_asid() != 0 {
                match unmapPage(
                    capability.get_frame_size(),
                    capability.get_frame_mapped_asid(),
                    capability.get_frame_mapped_address(),
                    capability.get_frame_base_ptr(),
                ) {
                    Err(lookup_fault) => unsafe { current_lookup_fault = lookup_fault },
                    _ => {}
                }
            }
        }

        cap_tag::cap_page_table_cap => {
            if final_ && capability.get_pt_is_mapped() != 0 {
                let asid = capability.get_pt_mapped_asid();
                let find_ret = find_vspace_for_asid(asid);
                let pte = capability.get_pt_base_ptr();
                if find_ret.status == exception_t::EXCEPTION_NONE
                    && find_ret.vspace_root.unwrap() as usize == pte
                {
                    deleteASID(asid, pte as *mut PTE);
                } else {
                    convert_to_mut_type_ref::<PTE>(pte)
                        .unmap_page_table(asid, capability.get_pt_mapped_address());
                }
                if let Some(lookup_fault) = find_ret.lookup_fault {
                    unsafe {
                        current_lookup_fault = lookup_fault;
                    }
                }
            }
        }

        cap_tag::cap_asid_pool_cap => {
            if final_ {
                deleteASIDPool(
                    capability.get_asid_base(),
                    capability.get_asid_pool() as *mut asid_pool_t,
                );
            }
        }
        _ => {}
    }
    fc_ret.remainder = cap_t::new_null_cap();
    fc_ret.cleanupInfo = cap_t::new_null_cap();
    fc_ret
}

#[cfg(target_arch = "aarch64")]
pub fn Arch_finaliseCap(capability: &cap, final_: bool) -> finaliseCap_ret {
    use sel4_common::structures_gen::cap_Splayed;

    let mut fc_ret = finaliseCap_ret {
        remainder: cap_null_cap::new().unsplay(),
        cleanupInfo: cap_null_cap::new().unsplay(),
    };
    match capability.splay() {
        cap_Splayed::frame_cap(data) => {
            if data.get_capFMappedASID() != 0 {
                match unmapPage(
                    data.get_capFSize() as usize,
                    data.get_capFMappedASID() as usize,
                    data.get_capFMappedAddress() as usize,
                    data.get_capFBasePtr() as usize,
                ) {
                    Err(fault) => unsafe { current_lookup_fault = fault },
                    _ => {}
                }
            }
        }
        cap_Splayed::vspace_cap(data) => {
            if final_ && data.get_capVSIsMapped() == 1 {
                deleteASID(
                    data.get_capVSIsMapped() as usize,
                    data.get_capVSBasePtr() as _,
                );
            }
        }
        // cap_tag::CapPageGlobalDirectoryCap => {
        //     if final_ && capability.get_pgd_is_mapped() == 1 {
        //         deleteASID(capability.get_pgd_is_mapped(), capability.get_pgd_base_ptr() as _);
        //     }
        // }
        // cap_tag::CapPageUpperDirectoryCap => {
        //     if final_ && capability.get_pud_is_mapped() == 1 {
        //         let pud = ptr_to_mut(capability.get_pt_base_ptr() as *mut PUDE);
        //         unmap_page_upper_directory(
        //             capability.get_pud_mapped_asid(),
        //             capability.get_pud_mapped_address(),
        //             pud,
        //         );
        //     }
        // }
        // cap_tag::CapPageDirectoryCap => {
        //     if final_ && capability.get_pd_is_mapped() == 1 {
        //         let pd = ptr_to_mut(capability.get_pt_base_ptr() as *mut PDE);
        //         unmap_page_directory(capability.get_pd_mapped_asid(), capability.get_pd_mapped_address(), pd);
        //     }
        // }
        cap_Splayed::page_table_cap(data) => {
            if final_ && data.get_capPTIsMapped() == 1 {
                let pte = ptr_to_mut(data.get_capPTBasePtr() as *mut PTE);
                unmap_page_table(
                    data.get_capPTMappedASID() as usize,
                    data.get_capPTMappedAddress() as usize,
                    pte,
                );
            }
        }
        cap_Splayed::asid_pool_cap(data) => {
            if final_ {
                deleteASIDPool(
                    data.get_capASIDBase() as usize,
                    data.get_capASIDPool() as *mut asid_pool_t,
                );
            }
        }
        cap_Splayed::asid_control_cap(_) => {}
        _ => unimplemented!("finaliseCap: {:?}", capability.get_tag()),
    }
    fc_ret.remainder = cap_null_cap::new().unsplay();
    fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
    fc_ret
}

#[no_mangle]
pub fn finaliseCap(capability: &cap, _final: bool, _exposed: bool) -> finaliseCap_ret {
    let mut fc_ret = finaliseCap_ret {
        remainder: cap_null_cap::new().unsplay(),
        cleanupInfo: cap_null_cap::new().unsplay(),
    };

    if capability.isArchCap() {
        // For Removing Warnings
        // #[cfg(target_arch = "aarch64")]
        // unsafe {
        //     return Arch_finaliseCap(capability, _final);
        // }
        // #[cfg(target_arch = "riscv64")]
        return Arch_finaliseCap(capability, _final);
    }
    match capability.splay() {
        cap_Splayed::endpoint_cap(data) => {
            if _final {
                // cancelAllIPC(cap.get_ep_ptr() as *mut endpoint_t);
                convert_to_mut_type_ref::<endpoint_t>(data.get_capEPPtr() as usize).cancel_all_ipc()
            }
            fc_ret.remainder = cap_null_cap::new().unsplay();
            fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
            return fc_ret;
        }
        cap_Splayed::notification_cap(data) => {
            if _final {
                let ntfn =
                    convert_to_mut_type_ref::<notification_t>(data.get_capNtfnPtr() as usize);
                ntfn.safe_unbind_tcb();
                ntfn.cacncel_all_signal();
            }
            fc_ret.remainder = cap_null_cap::new().unsplay();
            fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
            return fc_ret;
        }
        cap_Splayed::reply_cap(_) | cap_Splayed::null_cap(_) | cap_Splayed::domain_cap(_) => {
            fc_ret.remainder = cap_null_cap::new().unsplay();
            fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
            return fc_ret;
        }
        _ => {
            if _exposed {
                panic!("finaliseCap: failed to finalise immediately.");
            }
        }
    }

    match capability.splay() {
        cap_Splayed::cnode_cap(data) => {
            return if _final {
                fc_ret.remainder = Zombie_new(
                    1usize << data.get_capCNodeRadix() as usize,
                    data.get_capCNodeRadix() as usize,
                    data.get_capCNodePtr() as usize,
                );
                fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
                fc_ret
            } else {
                fc_ret.remainder = cap_null_cap::new().unsplay();
                fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
                fc_ret
            }
        }
        cap_Splayed::thread_cap(data) => {
            if _final {
                let tcb = convert_to_mut_type_ref::<tcb_t>(data.get_capTCBPtr() as usize);
                #[cfg(feature = "ENABLE_SMP")]
                unsafe {
                    crate::ffi::remoteTCBStall(tcb)
                };
                let cte_ptr = tcb.get_cspace_mut_ref(tcbCTable);
                safe_unbind_notification(tcb);
                tcb.cancel_ipc();
                tcb.suspend();
                // #[cfg(feature="DEBUG_BUILD")]
                // unsafe {
                //     tcbDebugRemove(tcb as *mut tcb_t);
                // }
                fc_ret.remainder =
                    Zombie_new(tcbCNodeEntries, ZombieType_ZombieTCB, cte_ptr.get_ptr());
                fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
                return fc_ret;
            }
        }
        cap_Splayed::zombie_cap(_) => {
            fc_ret.remainder = capability.clone();
            fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
            return fc_ret;
        }
        cap_Splayed::irq_handler_cap(data) => {
            if _final {
                let irq = data.get_capIRQ();
                deletingIRQHandler(irq as usize);
                fc_ret.remainder = cap_null_cap::new().unsplay();
                fc_ret.cleanupInfo = capability.clone();
                return fc_ret;
            }
        }
        _ => {
            fc_ret.remainder = cap_null_cap::new().unsplay();
            fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
            return fc_ret;
        }
    }
    fc_ret.remainder = cap_null_cap::new().unsplay();
    fc_ret.cleanupInfo = cap_null_cap::new().unsplay();
    return fc_ret;
}

#[no_mangle]
pub fn post_cap_deletion(capability: &cap) {
    match capability.splay() {
        cap_Splayed::irq_handler_cap(data) => {
            let irq = data.get_capIRQ() as usize;
            setIRQState(IRQState::IRQInactive, irq);
        }
        _ => {}
    }
}

#[no_mangle]
pub fn preemptionPoint() -> exception_t {
    unsafe {
        ksWorkUnitsCompleted += 1;
        if ksWorkUnitsCompleted >= CONFIG_MAX_NUM_WORK_UNITS_PER_PREEMPTION {
            ksWorkUnitsCompleted = 0;

            if isIRQPending() {
                return exception_t::EXCEPTION_PREEMTED;
            }
        }
        exception_t::EXCEPTION_NONE
    }
}

#[no_mangle]
#[cfg(target_arch = "riscv64")]
pub fn deleteASID(asid: asid_t, vspace: *mut PTE) {
    unsafe {
        if let Err(lookup_fault) = delete_asid(
            asid,
            vspace,
            &get_currenct_thread().get_cspace(tcbVTable).capability,
        ) {
            current_lookup_fault = lookup_fault;
        }
    }
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub fn deleteASID(asid: asid_t, vspace: *mut PTE) {
    unsafe {
        if let Err(lookup_fault) = delete_asid(
            asid,
            vspace,
            &get_currenct_thread().get_cspace(tcbVTable).capability,
        ) {
            current_lookup_fault = lookup_fault;
        }
    }
}

#[no_mangle]
pub fn deleteASIDPool(asid_base: asid_t, pool: *mut asid_pool_t) {
    unsafe {
        if let Err(lookup_fault) = delete_asid_pool(
            asid_base,
            pool,
            &get_currenct_thread().get_cspace(tcbVTable).capability,
        ) {
            current_lookup_fault = lookup_fault;
        }
    }
}
