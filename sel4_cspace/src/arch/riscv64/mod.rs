use sel4_common::{
    arch::maskVMRights, cap_rights::seL4_CapRights_t, plus_define_bitfield,
    structures::exception_t, structures_gen::cap_tag, utils::pageBitsForSize,
    vm_rights::vm_rights_from_word, MASK,
};

use crate::{cte::deriveCap_ret, interface::cte_t};
// cap_t 表示一个capability，由两个机器字组成，包含了类型、对象元数据以及指向内核对象的指针。
// 每个类型的capability的每个字段都实现了get和set方法。
plus_define_bitfield! {
    cap_t, 2, 0, 59, 5 => {
        new_null_cap, cap_tag::cap_null_cap as usize => {},
        new_untyped_cap, cap_tag::cap_untyped_cap as usize => {
            capFreeIndex, get_untyped_free_index, set_untyped_free_index, 1, 25, 39, 0, false,
            capIsDevice, get_untyped_is_device, set_untyped_is_device, 1, 6, 1, 0, false,
            capBlockSize, get_untyped_block_size, set_untyped_block_size, 1, 0, 6, 0, false,
            capPtr, get_untyped_ptr, set_untyped_ptr, 0, 0, 39, 0, true
        },
        new_endpoint_cap, cap_tag::cap_endpoint_cap as usize => {
            capEPBadge, get_ep_badge, set_ep_badge, 1, 0, 64, 0, false,
            capCanGrantReply, get_ep_can_grant_reply, set_ep_can_grant_reply, 0, 58, 1, 0, false,
            capCanGrant, get_ep_can_grant, set_ep_can_grant, 0, 57, 1, 0, false,
            capCanSend, get_ep_can_send, set_ep_can_send, 0, 55, 1, 0, false,
            capCanReceive, get_ep_can_receive, set_ep_can_receive, 0, 56, 1, 0, false,
            capEPPtr, get_ep_ptr, set_ep_ptr, 0, 0, 39, 0, true
        },
        new_notification_cap, cap_tag::cap_notification_cap as usize => {
            capNtfnBadge, get_nf_badge, set_nf_badge, 1, 0, 64, 0, false,
            capNtfnCanReceive, get_nf_can_receive, set_nf_can_receive, 0, 58, 1, 0, false,
            capNtfnCanSend, get_nf_can_send, set_nf_can_send, 0, 57, 1, 0, false,
            capNtfnPtr, get_nf_ptr, set_nf_ptr, 0, 0, 39, 0, true
        },
        new_reply_cap, cap_tag::cap_reply_cap as usize => {
            capReplyCanGrant, get_reply_can_grant, set_reply_can_grant, 0, 1, 1, 0, false,
            capReplyMaster, get_reply_master, set_reply_master, 0, 0, 1, 0, false,
            capTCBPtr, get_reply_tcb_ptr, set_reply_tcb_ptr, 1, 0, 64, 0, false
        },
        new_cnode_cap, cap_tag::cap_cnode_cap as usize => {
            capCNodeRadix, get_cnode_radix, set_cnode_radix, 0, 47, 6, 0, false,
            capCNodeGuardSize, get_cnode_guard_size, set_cnode_guard_size, 0, 53, 6, 0, false,
            capCNodeGuard, get_cnode_guard, set_cnode_guard, 1, 0, 64, 0, false,
            capCNodePtr, get_cnode_ptr, set_cnode_ptr, 0, 0, 38, 1, true
        },
        new_thread_cap, cap_tag::cap_thread_cap as usize => {
            capTCBPtr, get_tcb_ptr, set_tcb_ptr, 0, 0, 39, 0, true
        },
        new_irq_control_cap, cap_tag::cap_irq_control_cap as usize => {},
        new_irq_handler_cap, cap_tag::cap_irq_handler_cap as usize => {
            capIRQ, get_irq_handler, set_irq_handler, 1, 0, 12, 0, false
        },
        new_zombie_cap, cap_tag::cap_zombie_cap as usize => {
            capZombieID, get_zombie_id, set_zombie_id, 1, 0, 64, 0, false,
            capZombieType, get_zombie_type, set_zombie_type, 0, 0, 7, 0, false
        },
        new_domain_cap, cap_tag::cap_domain_cap as usize => {},
        new_frame_cap, cap_tag::cap_frame_cap as usize => {
            capFMappedASID, get_frame_mapped_asid, set_frame_mapped_asid, 1, 48, 16, 0, false,
            capFBasePtr, get_frame_base_ptr, set_frame_base_ptr, 1, 9, 39, 0, true,
            capFSize, get_frame_size, set_frame_size, 0, 57, 2, 0, false,
            capFVMRights, get_frame_vm_rights, set_frame_vm_rights, 0, 55, 2, 0, false,
            capFIsDevice, get_frame_is_device, set_frame_is_device, 0, 54, 1, 0, false,
            capFMappedAddress, get_frame_mapped_address, set_frame_mapped_address, 0, 0, 39, 0, true
        },
        new_page_table_cap, cap_tag::cap_page_table_cap as usize => {
            capPTMappedASID, get_pt_mapped_asid, set_pt_mapped_asid, 1, 48, 16, 0, false,
            capPTBasePtr, get_pt_base_ptr, set_pt_base_ptr, 1, 9, 39, 0, true,
            capPTIsMapped, get_pt_is_mapped, set_pt_is_mapped, 0, 39, 1, 0, false,
            capPTMappedAddress, get_pt_mapped_address, set_pt_mapped_address, 0, 0, 39, 0, true
        },
        new_asid_control_cap, cap_tag::cap_asid_control_cap as usize => {},
        new_asid_pool_cap, cap_tag::cap_asid_pool_cap as usize => {
            capASIDBase, get_asid_base, set_asid_base, 0, 43, 16, 0, false,
            capASIDPool, get_asid_pool, set_asid_pool, 0, 0, 37, 2, true
        }
    }
}

impl cap_t {
    pub fn get_cap_ptr(&self) -> usize {
        match self.get_cap_type() {
            cap_tag::cap_untyped_cap => self.get_untyped_ptr(),
            cap_tag::cap_endpoint_cap => self.get_ep_ptr(),
            cap_tag::cap_notification_cap => self.get_nf_ptr(),
            cap_tag::cap_cnode_cap => self.get_cnode_ptr(),
            cap_tag::cap_thread_cap => self.get_tcb_ptr(),
            cap_tag::cap_zombie_cap => self.get_zombie_ptr(),
            cap_tag::cap_frame_cap => self.get_frame_base_ptr(),
            cap_tag::cap_page_table_cap => self.get_pt_base_ptr(),
            cap_tag::cap_asid_pool_cap => self.get_asid_pool(),
            _ => 0,
        }
    }
}

impl cte_t {
    pub fn arch_derive_cap(&mut self, cap: &cap_t) -> deriveCap_ret {
        let mut ret = deriveCap_ret {
            status: exception_t::EXCEPTION_NONE,
            cap: cap_t::default(),
        };
        match cap.get_cap_type() {
            cap_tag::cap_page_table_cap => {
                if cap.get_pt_is_mapped() != 0 {
                    ret.cap = cap.clone();
                    ret.status = exception_t::EXCEPTION_NONE;
                } else {
                    ret.cap = cap_t::new_null_cap();
                    ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            cap_tag::cap_frame_cap => {
                let mut newCap = cap.clone();
                newCap.set_frame_mapped_address(0);
                newCap.set_frame_mapped_asid(0);
                ret.cap = newCap;
            }
            cap_tag::cap_asid_control_cap | cap_tag::cap_asid_pool_cap => {
                ret.cap = cap.clone();
            }
            _ => {
                panic!(" Invalid arch cap type : {}", cap.get_cap_type() as usize);
            }
        }
        ret
    }
}

pub fn arch_mask_cap_rights(rights: seL4_CapRights_t, cap: &cap_t) -> cap_t {
    if cap.get_cap_type() == cap_tag::cap_frame_cap {
        let mut vm_rights = vm_rights_from_word(cap.get_frame_vm_rights());
        vm_rights = maskVMRights(vm_rights, rights);
        let mut new_cap = cap.clone();
        new_cap.set_frame_vm_rights(vm_rights as usize);
        new_cap
    } else {
        cap.clone()
    }
}

pub fn arch_same_object_as(cap1: &cap_t, cap2: &cap_t) -> bool {
    if cap1.get_cap_type() == cap_tag::cap_frame_cap
        && cap2.get_cap_type() == cap_tag::cap_frame_cap
    {
        return cap1.get_frame_base_ptr() == cap2.get_frame_base_ptr()
            && cap1.get_frame_size() == cap2.get_frame_size()
            && (cap1.get_frame_is_device() == 0) == (cap2.get_frame_is_device() == 0);
    }
    arch_same_region_as(cap1, cap2)
}

pub fn arch_same_region_as(cap1: &cap_t, cap2: &cap_t) -> bool {
    match cap1.get_cap_type() {
        cap_tag::cap_frame_cap => {
            if cap2.get_cap_type() == cap_tag::cap_frame_cap {
                let botA = cap1.get_frame_base_ptr();
                let botB = cap2.get_frame_base_ptr();
                let topA = botA + MASK!(pageBitsForSize(cap1.get_frame_size()));
                let topB = botB + MASK!(pageBitsForSize(cap2.get_frame_size()));
                return (botA <= botB) && (topA >= topB) && (botB <= topB);
            }
        }
        cap_tag::cap_page_table_cap => {
            if cap2.get_cap_type() == cap_tag::cap_page_table_cap {
                return cap1.get_pt_base_ptr() == cap2.get_pt_base_ptr();
            }
        }
        cap_tag::cap_asid_control_cap => {
            return cap2.get_cap_type() == cap_tag::cap_asid_control_cap;
        }
        cap_tag::cap_asid_pool_cap => {
            if cap2.get_cap_type() == cap_tag::cap_asid_pool_cap {
                return cap1.get_asid_pool() == cap2.get_asid_pool();
            }
        }
        _ => panic!("unknown cap"),
    }
    false
}
