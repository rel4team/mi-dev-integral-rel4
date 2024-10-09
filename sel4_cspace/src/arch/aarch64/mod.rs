use sel4_common::{
    arch::maskVMRights,
    cap_rights::seL4_CapRights_t,
    structures::exception_t,
    structures_gen::{cap, cap_Splayed, cap_null_cap, cap_tag},
    utils::pageBitsForSize,
    vm_rights::vm_rights_from_word,
    MASK,
};

use crate::{
    capability::{cap_arch_func, zombie::cap_zombie_func},
    cte::{cte_t, deriveCap_ret},
};

// plus_define_bitfield! {
//     cap_t, 2, 0, 59, 5 => {
//         new_null_cap, cap_tag::cap_null_cap as usize => {},
//         new_untyped_cap, cap_tag::cap_untyped_cap as usize => {
//             capFreeIndex, get_untyped_free_index, set_untyped_free_index, 1, 16, 48, 0, false,
//             capIsDevice, get_untyped_is_device, set_untyped_is_device, 1, 6, 1, 0, false,
//             capBlockSize, get_untyped_block_size, set_untyped_block_size, 1, 0, 6, 0, false,
//             capPtr, get_untyped_ptr, set_untyped_ptr, 0, 0, 48, 0, true
//         },
//         new_endpoint_cap, cap_tag::cap_endpoint_cap as usize => {
//             capEPBadge, get_ep_badge, set_ep_badge, 1, 0, 64, 0, false,
//             capCanGrantReply, get_ep_can_grant_reply, set_ep_can_grant_reply, 0, 58, 1, 0, false,
//             capCanGrant, get_ep_can_grant, set_ep_can_grant, 0, 57, 1, 0, false,
//             capCanSend, get_ep_can_send, set_ep_can_send, 0, 55, 1, 0, false,
//             capCanReceive, get_ep_can_receive, set_ep_can_receive, 0, 56, 1, 0, false,
//             capEPPtr, get_ep_ptr, set_ep_ptr, 0, 0, 48, 0, true
//         },
//         new_notification_cap, cap_tag::cap_notification_cap as usize => {
//             capNtfnBadge, get_nf_badge, set_nf_badge, 1, 0, 64, 0, false,
//             capNtfnCanReceive, get_nf_can_receive, set_nf_can_receive, 0, 58, 1, 0, false,
//             capNtfnCanSend, get_nf_can_send, set_nf_can_send, 0, 57, 1, 0, false,
//             capNtfnPtr, get_nf_ptr, set_nf_ptr, 0, 0, 48, 0, true
//         },
//         new_reply_cap, cap_tag::cap_reply_cap as usize => {
//             capReplyCanGrant, get_reply_can_grant, set_reply_can_grant, 0, 1, 1, 0, false,
//             capReplyMaster, get_reply_master, set_reply_master, 0, 0, 1, 0, false,
//             capTCBPtr, get_reply_tcb_ptr, set_reply_tcb_ptr, 1, 0, 64, 0, false
//         },
//         new_cnode_cap, cap_tag::cap_cnode_cap as usize => {
//             capCNodeRadix, get_cnode_radix, set_cnode_radix, 0, 47, 6, 0, false,
//             capCNodeGuardSize, get_cnode_guard_size, set_cnode_guard_size, 0, 53, 6, 0, false,
//             capCNodeGuard, get_cnode_guard, set_cnode_guard, 1, 0, 64, 0, false,
//             capCNodePtr, get_cnode_ptr, set_cnode_ptr, 0, 0, 47, 1, true
//         },
//         new_thread_cap, cap_tag::cap_thread_cap as usize => {
//             capTCBPtr, get_tcb_ptr, set_tcb_ptr, 0, 0, 48, 0, true
//         },
//         new_irq_control_cap, cap_tag::cap_irq_control_cap as usize => {},
//         new_irq_handler_cap, cap_tag::cap_irq_handler_cap as usize => {
//             capIRQ, get_irq_handler, set_irq_handler, 1, 0, 12, 0, false
//         },
//         new_zombie_cap, cap_tag::cap_zombie_cap as usize => {
//             capZombieID, get_zombie_id, set_zombie_id, 1, 0, 64, 0, false,
//             capZombieType, get_zombie_type, set_zombie_type, 0, 0, 7, 0, false
//         },
//         new_domain_cap, cap_tag::cap_domain_cap as usize => {},
//         new_frame_cap, cap_tag::cap_frame_cap as usize => {
//             capFIsDevice, get_frame_is_device,set_frame_is_device, 0, 6, 1, 0, false,
//             capFVMRights,get_frame_vm_rights, set_frame_vm_rights, 0, 7, 2, 0, false,
//             capFMappedAddress, get_frame_mapped_address, set_frame_mapped_address, 0, 9, 48, 0, true,
//             capFSize, get_frame_size, set_frame_size, 0, 57, 2, 0, false,
//             capFMappedASID, get_frame_mapped_asid, set_frame_mapped_asid, 1, 48, 16, 0, false,
//             capFBasePtr, get_frame_base_ptr, set_frame_base_ptr, 1, 0, 48, 0, true

//         },
//         new_page_table_cap, cap_tag::cap_page_table_cap as usize => {
//             capPTMappedASID, get_pt_mapped_asid, set_pt_mapped_asid, 1, 48, 16, 0, false,
//             capPTBasePtr, get_pt_base_ptr, set_pt_base_ptr, 1, 0, 48, 0, true,
//             capPTIsMapped, get_pt_is_mapped, set_pt_is_mapped, 0, 48, 1, 0, false,
//             capPTMappedAddress, get_pt_mapped_address, set_pt_mapped_address, 0, 20, 28, 20, true
//         },
// new_page_directory_cap, cap_tag::CapPageDirectoryCap as usize => {
//     capPDMappedASID, get_pd_mapped_asid, set_pd_mapped_asid, 1, 48, 16, 0, false,
//     capPDBasePtr, get_pd_base_ptr, set_pd_base_ptr, 1, 0, 48, 0, true,
//     capPDIsMapped, get_pd_is_mapped, set_pd_is_mapped, 0, 48, 1, 0, false,
//     capPDMappedAddress, get_pd_mapped_address, set_pd_mapped_address, 0, 29, 19, 0, true
// },
// new_page_upper_directory_cap, cap_tag::CapPageUpperDirectoryCap as usize => {
//     capPUDMappedASID, get_pud_mapped_asid, set_pud_mapped_asid, 1, 48, 16, 0, false,
//     capPUDBasePtr, get_pud_base_ptr, set_pud_base_ptr, 1, 0, 48, 0, true,
//     capPUDIsMapped, get_pud_is_mapped, set_pud_is_mapped, 0, 58, 1, 0, false,
//     capPUDMappedAddress, get_pud_mapped_address, set_pud_mapped_address, 0, 48, 10, 0, true
// },
// new_page_global_directory_cap, cap_tag::CapPageGlobalDirectoryCap as usize => {
//     capPGDMappedASID, get_pgd_mapped_asid, set_pgd_mapped_asid, 1, 48, 16, 0, false,
//     capPGDBasePtr, get_pgd_base_ptr, set_pgd_base_ptr, 1, 0, 48, 0, true,
//     capPGDIsMapped, get_pgd_is_mapped, set_pgd_is_mapped, 0, 58, 1, 0, false
// },
//         new_vspace_cap, cap_tag::cap_vspace_cap as usize => {
//             capVSMappedASID, get_vs_mapped_asid, set_vs_mapped_asid, 1, 48, 16, 0, false,
//             capVSBasePtr, get_vs_base_ptr, set_vs_base_ptr, 1, 0, 48, 0, true,
//             capVSIsMapped, get_vs_is_mapped, set_vs_is_mapped, 0, 58, 1, 0, false
//         },
//         new_asid_control_cap, cap_tag::cap_asid_control_cap as usize => {},
//         new_asid_pool_cap, cap_tag::cap_asid_pool_cap as usize => {
//             capASIDBase, get_asid_base, set_asid_base, 0, 43, 16, 0, false,
//             // FIXED: asid_pool need to shift left 11 bits.
//             capASIDPool, get_asid_pool, set_asid_pool, 0, 0, 37, 11, true
//         }
//     }
// }

impl cap_arch_func for cap {
    fn get_cap_ptr(&self) -> usize {
        match self.splay() {
            cap_Splayed::untyped_cap(data) => data.get_capPtr() as usize,
            cap_Splayed::endpoint_cap(data) => data.get_capEPPtr() as usize,
            cap_Splayed::notification_cap(data) => data.get_capNtfnPtr() as usize,
            cap_Splayed::cnode_cap(data) => data.get_capCNodePtr() as usize,
            cap_Splayed::thread_cap(data) => data.get_capTCBPtr() as usize,
            cap_Splayed::zombie_cap(data) => data.get_zombie_ptr() as usize,
            cap_Splayed::frame_cap(data) => data.get_capFBasePtr() as usize,
            cap_Splayed::page_table_cap(data) => data.get_capPTBasePtr() as usize,
            cap_Splayed::vspace_cap(data) => data.get_capVSBasePtr() as usize,
            cap_Splayed::asid_control_cap(_) => 0,
            cap_Splayed::asid_pool_cap(data) => data.get_capASIDPool() as usize,
            _ => 0,
        }
        // match self.get_cap_type() {
        //     cap_tag::cap_untyped_cap => self.get_untyped_ptr(),
        //     cap_tag::cap_endpoint_cap => self.get_ep_ptr(),
        //     cap_tag::cap_notification_cap => self.get_nf_ptr(),
        //     cap_tag::cap_cnode_cap => self.get_cnode_ptr(),
        //     cap_tag::cap_thread_cap => self.get_tcb_ptr(),
        //     cap_tag::cap_zombie_cap => self.get_zombie_ptr(),
        //     cap_tag::cap_frame_cap => self.get_frame_base_ptr(),
        //     cap_tag::cap_page_table_cap => self.get_pt_base_ptr(),
        //     cap_tag::cap_vspace_cap => self.get_vs_base_ptr(),
        //     // cap_tag::CapPageDirectoryCap => self.get_pd_base_ptr(),
        //     // cap_tag::CapPageUpperDirectoryCap => self.get_pud_base_ptr(),
        //     // cap_tag::CapPageGlobalDirectoryCap => self.get_pgd_base_ptr(),
        //     cap_tag::cap_asid_control_cap => 0,
        //     cap_tag::cap_asid_pool_cap => self.get_asid_pool(),
        //     _ => 0,
        // }
    }

    #[inline]
    fn is_vtable_root(&self) -> bool {
        self.get_tag() == cap_tag::cap_vspace_cap
    }

    #[inline]
    fn is_valid_native_root(&self) -> bool {
        match self.splay() {
            cap_Splayed::vspace_cap(data) => self.is_vtable_root() && data.get_capVSIsMapped() != 0,
            _ => false,
        }
    }

    #[inline]
    fn is_valid_vtable_root(&self) -> bool {
        self.is_valid_native_root()
    }
}

impl cte_t {
    pub fn arch_derive_cap(&mut self, capability: &cap) -> deriveCap_ret {
        let mut ret = deriveCap_ret {
            status: exception_t::EXCEPTION_NONE,
            capability: cap_null_cap::new().unsplay(),
        };
        match capability.splay() {
            // cap_tag::CapPageGlobalDirectoryCap => {
            //     if cap.get_pgd_is_mapped() != 0 {
            //         ret.cap = cap.clone();
            //         ret.status = exception_t::EXCEPTION_NONE;
            //     } else {
            //         ret.cap = cap_t::new_null_cap();
            //         ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
            //     }
            // }
            // cap_tag::CapPageUpperDirectoryCap => {
            //     if cap.get_pud_is_mapped() != 0 {
            //         ret.cap = cap.clone();
            //         ret.status = exception_t::EXCEPTION_NONE;
            //     } else {
            //         ret.cap = cap_t::new_null_cap();
            //         ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
            //     }
            // }
            // cap_tag::CapPageDirectoryCap => {
            //     if cap.get_pud_is_mapped() != 0 {
            //         ret.cap = cap.clone();
            //         ret.status = exception_t::EXCEPTION_NONE;
            //     } else {
            //         ret.cap = cap_t::new_null_cap();
            //         ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
            //     }
            // }
            cap_Splayed::vspace_cap(data) => {
                if data.get_capVSIsMapped() != 0 {
                    ret.capability = data.clone().unsplay();
                    ret.status = exception_t::EXCEPTION_NONE;
                } else {
                    ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            cap_Splayed::page_table_cap(data) => {
                if data.get_capPTIsMapped() != 0 {
                    ret.capability = data.clone().unsplay();
                    ret.status = exception_t::EXCEPTION_NONE;
                } else {
                    ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            cap_Splayed::frame_cap(data) => {
                let mut newCap = data.clone();
                newCap.set_capFMappedASID(0);
                ret.capability = newCap.unsplay();
            }
            cap_Splayed::asid_control_cap(data) => {
                ret.capability = data.clone().unsplay();
            }
            cap_Splayed::asid_pool_cap(data) => {
                ret.capability = data.clone().unsplay();
            }
            _ => {
                panic!(" Invalid arch cap type : {}", capability.get_tag() as usize);
            }
        }
        ret
    }
}

pub fn arch_mask_cap_rights(rights: seL4_CapRights_t, capability: &cap) -> cap {
    match capability.splay() {
        cap_Splayed::frame_cap(data) => {
            let mut vm_rights = vm_rights_from_word(data.get_capFVMRights() as usize);
            vm_rights = maskVMRights(vm_rights, rights);
            let mut new_cap = data.clone();
            new_cap.set_capFVMRights(vm_rights as u64);
            new_cap.unsplay()
        }
        _ => capability.clone(),
    }
}

pub fn arch_same_region_as(cap1: &cap, cap2: &cap) -> bool {
    match cap1.splay() {
        cap_Splayed::frame_cap(data1) => match cap2.splay() {
            cap_Splayed::frame_cap(data2) => {
                let botA = data1.get_capFBasePtr() as usize;
                let botB = data2.get_capFBasePtr() as usize;
                let topA = botA + MASK!(pageBitsForSize(data1.get_capFSize() as usize));
                let topB = botB + MASK!(pageBitsForSize(data2.get_capFSize() as usize));
                return (botA <= botB) && (topA >= topB) && (botB <= topB);
            }
            _ => return false,
        },
        cap_Splayed::page_table_cap(data1) => match cap2.splay() {
            cap_Splayed::page_table_cap(data2) => {
                return data1.get_capPTBasePtr() == data2.get_capPTBasePtr();
            }
            _ => return false,
        },
        cap_Splayed::vspace_cap(data1) => match cap2.splay() {
            cap_Splayed::vspace_cap(data2) => {
                return data1.get_capVSBasePtr() == data2.get_capVSBasePtr();
            }
            _ => return false,
        },
        cap_Splayed::asid_control_cap(_) => {
            return cap2.get_tag() == cap_tag::cap_asid_control_cap;
        }
        cap_Splayed::asid_pool_cap(data1) => match cap2.splay() {
            cap_Splayed::asid_pool_cap(data2) => {
                return data1.get_capASIDPool() == data2.get_capASIDPool();
            }
            _ => return false,
        },
        _ => panic!("unknown cap"),
    }
}

pub fn arch_same_object_as(cap1: &cap, cap2: &cap) -> bool {
    match cap1.splay() {
        cap_Splayed::frame_cap(data1) => match cap2.splay() {
            cap_Splayed::frame_cap(data2) => {
                return data1.get_capFBasePtr() == data2.get_capFBasePtr()
                    && data1.get_capFSize() == data2.get_capFSize()
                    && data1.get_capFIsDevice() == data2.get_capFIsDevice();
            }
            _ => {}
        },
        _ => {}
    }
    arch_same_region_as(cap1, cap2)
}
