use sel4_common::{
    arch::maskVMRights,
    cap_rights::seL4_CapRights_t,
    structures::exception_t,
    structures_gen::{
        cap, cap_asid_control_cap, cap_asid_pool_cap, cap_cnode_cap, cap_domain_cap,
        cap_endpoint_cap, cap_frame_cap, cap_irq_control_cap, cap_irq_handler_cap,
        cap_notification_cap, cap_null_cap, cap_page_table_cap, cap_reply_cap, cap_tag,
        cap_thread_cap, cap_untyped_cap, cap_zombie_cap,
    },
    utils::{pageBitsForSize, ptr_to_mut},
    vm_rights::vm_rights_from_word,
    MASK,
};

use crate::{
    capability::{cap_arch_func, zombie::zombie_func},
    cte::deriveCap_ret,
    interface::cte_t,
};

impl cap_arch_func for cap {
    fn get_cap_ptr(&self) -> usize {
        match self.get_tag() {
            cap_tag::cap_untyped_cap => cap::to_cap_untyped_cap(self).get_capPtr() as usize,
            cap_tag::cap_endpoint_cap => cap::to_cap_endpoint_cap(self).get_capEPPtr() as usize,
            cap_tag::cap_notification_cap => {
                cap::to_cap_notification_cap(self).get_capNtfnPtr() as usize
            }
            cap_tag::cap_cnode_cap => cap::to_cap_cnode_cap(self).get_capCNodePtr() as usize,
            cap_tag::cap_thread_cap => cap::to_cap_thread_cap(self).get_capTCBPtr() as usize,
            cap_tag::cap_zombie_cap => cap::to_cap_zombie_cap(self).get_zombie_ptr() as usize,
            cap_tag::cap_frame_cap => cap::to_cap_frame_cap(self).get_capFBasePtr() as usize,
            cap_tag::cap_page_table_cap => {
                cap::to_cap_page_table_cap(self).get_capPTBasePtr() as usize
            }
            cap_tag::cap_asid_pool_cap => {
                cap::to_cap_asid_pool_cap(self).get_capASIDPool() as usize
            }
            _ => 0,
        }
    }

    fn is_vtable_root(&self) -> bool {
        todo!()
    }

    fn is_valid_native_root(&self) -> bool {
        todo!()
    }

    fn is_valid_vtable_root(&self) -> bool {
        todo!()
    }
}

impl cte_t {
    pub fn arch_derive_cap(&self, capability: &cap) -> deriveCap_ret {
        let mut ret = deriveCap_ret {
            status: exception_t::EXCEPTION_NONE,
            capability: cap_null_cap::new().unsplay(),
        };
        match capability.get_tag() {
            cap_tag::cap_page_table_cap => {
                if cap::to_cap_page_table_cap(capability).get_capPTIsMapped() != 0 {
                    ret.capability = capability.clone();
                    ret.status = exception_t::EXCEPTION_NONE;
                } else {
                    ret.capability = cap_null_cap::new().unsplay();
                    ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            cap_tag::cap_frame_cap => {
                let newCap = capability.clone();
                cap::to_cap_frame_cap(&newCap).set_capFMappedAddress(0);
                cap::to_cap_frame_cap(&newCap).set_capFMappedASID(0);
                ret.capability = newCap;
            }
            cap_tag::cap_asid_control_cap | cap_tag::cap_asid_pool_cap => {
                ret.capability = capability.clone();
            }
            _ => {
                panic!(" Invalid arch cap type : {}", capability.get_tag() as usize);
            }
        }
        ret
    }
}

pub fn arch_mask_cap_rights(rights: seL4_CapRights_t, capability: &cap) -> cap {
    if capability.get_tag() == cap_tag::cap_frame_cap {
        let mut vm_rights =
            vm_rights_from_word(cap::to_cap_frame_cap(capability).get_capFVMRights() as usize);
        vm_rights = maskVMRights(vm_rights, rights);
        let new_cap = capability.clone();
        cap::to_cap_frame_cap(&new_cap).set_capFVMRights(vm_rights as u64);
        new_cap
    } else {
        capability.clone()
    }
}

pub fn arch_same_object_as(cap1: &cap, cap2: &cap) -> bool {
    if cap1.get_tag() == cap_tag::cap_frame_cap && cap2.get_tag() == cap_tag::cap_frame_cap {
        return cap::to_cap_frame_cap(cap1).get_capFBasePtr()
            == cap::to_cap_frame_cap(cap2).get_capFBasePtr()
            && cap::to_cap_frame_cap(cap1).get_capFSize()
                == cap::to_cap_frame_cap(cap2).get_capFSize()
            && (cap::to_cap_frame_cap(cap1).get_capFIsDevice() == 0)
                == (cap::to_cap_frame_cap(cap2).get_capFIsDevice() == 0);
    }
    arch_same_region_as(cap1, cap2)
}

pub fn arch_same_region_as(cap1: &cap, cap2: &cap) -> bool {
    match cap1.get_tag() {
        cap_tag::cap_frame_cap => {
            if cap2.get_tag() == cap_tag::cap_frame_cap {
                let botA = cap::to_cap_frame_cap(cap1).get_capFBasePtr() as usize;
                let botB = cap::to_cap_frame_cap(cap2).get_capFBasePtr() as usize;
                let topA = botA
                    + MASK!(pageBitsForSize(
                        cap::to_cap_frame_cap(cap1).get_capFSize() as usize
                    ));
                let topB = botB
                    + MASK!(pageBitsForSize(
                        cap::to_cap_frame_cap(cap2).get_capFSize() as usize
                    ));
                return (botA <= botB) && (topA >= topB) && (botB <= topB);
            }
        }
        cap_tag::cap_page_table_cap => {
            if cap2.get_tag() == cap_tag::cap_page_table_cap {
                return cap::to_cap_page_table_cap(cap1).get_capPTBasePtr()
                    == cap::to_cap_page_table_cap(cap2).get_capPTBasePtr();
            }
        }
        cap_tag::cap_asid_control_cap => {
            return cap2.get_tag() == cap_tag::cap_asid_control_cap;
        }
        cap_tag::cap_asid_pool_cap => {
            if cap2.get_tag() == cap_tag::cap_asid_pool_cap {
                return cap::to_cap_asid_pool_cap(cap1).get_capASIDPool()
                    == cap::to_cap_asid_pool_cap(cap2).get_capASIDPool();
            }
        }
        _ => panic!("unknown cap"),
    }
    false
}

pub trait cap_trans {
    fn to_cap_null_cap(capability: &Self) -> &mut cap_null_cap;
    fn to_cap_untyped_cap(capability: &Self) -> &mut cap_untyped_cap;
    fn to_cap_endpoint_cap(capability: &Self) -> &mut cap_endpoint_cap;
    fn to_cap_notification_cap(capability: &Self) -> &mut cap_notification_cap;
    fn to_cap_reply_cap(capability: &Self) -> &mut cap_reply_cap;
    fn to_cap_cnode_cap(capability: &Self) -> &mut cap_cnode_cap;
    fn to_cap_thread_cap(capability: &Self) -> &mut cap_thread_cap;
    fn to_cap_irq_control_cap(capability: &Self) -> &mut cap_irq_control_cap;
    fn to_cap_irq_handler_cap(capability: &Self) -> &mut cap_irq_handler_cap;
    fn to_cap_zombie_cap(capability: &Self) -> &mut cap_zombie_cap;
    fn to_cap_domain_cap(capability: &Self) -> &mut cap_domain_cap;
    fn to_cap_frame_cap(capability: &Self) -> &mut cap_frame_cap;
    fn to_cap_page_table_cap(capability: &Self) -> &mut cap_page_table_cap;
    fn to_cap_asid_control_cap(capability: &Self) -> &mut cap_asid_control_cap;
    fn to_cap_asid_pool_cap(capability: &Self) -> &mut cap_asid_pool_cap;
}
impl cap_trans for cap {
    fn to_cap_null_cap(capability: &Self) -> &mut cap_null_cap {
        ptr_to_mut(capability as *const _ as *mut cap_null_cap)
    }
    fn to_cap_untyped_cap(capability: &Self) -> &mut cap_untyped_cap {
        ptr_to_mut(capability as *const _ as *mut cap_untyped_cap)
    }
    fn to_cap_endpoint_cap(capability: &Self) -> &mut cap_endpoint_cap {
        ptr_to_mut(capability as *const _ as *mut cap_endpoint_cap)
    }
    fn to_cap_notification_cap(capability: &Self) -> &mut cap_notification_cap {
        ptr_to_mut(capability as *const _ as *mut cap_notification_cap)
    }
    fn to_cap_reply_cap(capability: &Self) -> &mut cap_reply_cap {
        ptr_to_mut(capability as *const _ as *mut cap_reply_cap)
    }
    fn to_cap_cnode_cap(capability: &Self) -> &mut cap_cnode_cap {
        ptr_to_mut(capability as *const _ as *mut cap_cnode_cap)
    }
    fn to_cap_thread_cap(capability: &Self) -> &mut cap_thread_cap {
        ptr_to_mut(capability as *const _ as *mut cap_thread_cap)
    }
    fn to_cap_irq_control_cap(capability: &Self) -> &mut cap_irq_control_cap {
        ptr_to_mut(capability as *const _ as *mut cap_irq_control_cap)
    }
    fn to_cap_irq_handler_cap(capability: &Self) -> &mut cap_irq_handler_cap {
        ptr_to_mut(capability as *const _ as *mut cap_irq_handler_cap)
    }
    fn to_cap_zombie_cap(capability: &Self) -> &mut cap_zombie_cap {
        ptr_to_mut(capability as *const _ as *mut cap_zombie_cap)
    }
    fn to_cap_domain_cap(capability: &Self) -> &mut cap_domain_cap {
        ptr_to_mut(capability as *const _ as *mut cap_domain_cap)
    }
    fn to_cap_frame_cap(capability: &Self) -> &mut cap_frame_cap {
        ptr_to_mut(capability as *const _ as *mut cap_frame_cap)
    }
    fn to_cap_page_table_cap(capability: &Self) -> &mut cap_page_table_cap {
        ptr_to_mut(capability as *const _ as *mut cap_page_table_cap)
    }
    fn to_cap_asid_control_cap(capability: &Self) -> &mut cap_asid_control_cap {
        ptr_to_mut(capability as *const _ as *mut cap_asid_control_cap)
    }
    fn to_cap_asid_pool_cap(capability: &Self) -> &mut cap_asid_pool_cap {
        ptr_to_mut(capability as *const _ as *mut cap_asid_pool_cap)
    }
}
