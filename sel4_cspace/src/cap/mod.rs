//! 该模块定义了几乎全部的`capability`，可以在`sel4_common`中找到`plus_define_bitfield!`宏的具体实现，
//! 该宏在生成`capability`的同时，会生成每个字段的`get``set`方法
//! cap_t 表示一个capability，由两个机器字组成，包含了类型、对象元数据以及指向内核对象的指针。
//! 每个类型的capability的每个字段都实现了get和set方法。
//!
//! 记录在阅读代码段过程中用到的`cap`的特定字段含义：
//!
//! ```
//! untyped_cap:
//!  - capFreeIndex：从capPtr到可用的块的偏移，单位是2^seL4_MinUntypedBits大小的块数。如果seL4_MinUntypedBits是4，那么2^seL4_MinUntypedBits就是16字节。如果一个64字节的内存块已经分配了前32字节，则CapFreeIndex会存储2，因为已经使用了2个16字节的块。
//!  - capBlockSize：当前untyped块中剩余空间大小
//! endpoint_cap:
//!  - capEPBadge：当使用Mint方法创建一个新的endpoint_cap时，可以设置badge，用于表示派生关系，例如一个进程可以与多个进程通信，为了判断消息究竟来自哪个进程，就可以使用badge区分。
//! ```
//! Represent a capability, composed by two words. Different cap can contain different bit fields.

pub mod zombie;

use sel4_common::structures_gen::cap_tag;
use sel4_common::{sel4_config::*, MASK};

use crate::arch::{arch_same_object_as, cap_t};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CNodeCapData {
    pub words: [usize; 1],
}

impl CNodeCapData {
    #[inline]
    pub fn new(data: usize) -> Self {
        CNodeCapData { words: [data] }
    }

    #[inline]
    pub fn get_guard(&self) -> usize {
        (self.words[0] & 0xffffffffffffffc0usize) >> 6
    }

    #[inline]
    pub fn get_guard_size(&self) -> usize {
        self.words[0] & 0x3fusize
    }
}

// cap_t 表示一个capability，由两个机器字组成，包含了类型、对象元数据以及指向内核对象的指针。
// 每个类型的capability的每个字段都实现了get和set方法。
// plus_define_bitfield! {
//     cap_t, 2, 0, 59, 5 => {
//         new_null_cap, cap_tag::CapNullCap as usize => {},
//         new_untyped_cap, cap_tag::CapUntypedCap as usize => {
//             capFreeIndex, get_untyped_free_index, set_untyped_free_index, 1, 25, 39, 0, false,
//             capIsDevice, get_untyped_is_device, set_untyped_is_device, 1, 6, 1, 0, false,
//             capBlockSize, get_untyped_block_size, set_untyped_block_size, 1, 0, 6, 0, false,
//             capPtr, get_untyped_ptr, set_untyped_ptr, 0, 0, 39, 0, true
//         },
//         new_endpoint_cap, cap_tag::CapEndpointCap as usize => {
//             capEPBadge, get_ep_badge, set_ep_badge, 1, 0, 64, 0, false,
//             capCanGrantReply, get_ep_can_grant_reply, set_ep_can_grant_reply, 0, 58, 1, 0, false,
//             capCanGrant, get_ep_can_grant, set_ep_can_grant, 0, 57, 1, 0, false,
//             capCanSend, get_ep_can_send, set_ep_can_send, 0, 55, 1, 0, false,
//             capCanReceive, get_ep_can_receive, set_ep_can_receive, 0, 56, 1, 0, false,
//             capEPPtr, get_ep_ptr, set_ep_ptr, 0, 0, 39, 0, true
//         },
//         new_notification_cap, cap_tag::CapNotificationCap as usize => {
//             capNtfnBadge, get_nf_badge, set_nf_badge, 1, 0, 64, 0, false,
//             capNtfnCanReceive, get_nf_can_receive, set_nf_can_receive, 0, 58, 1, 0, false,
//             capNtfnCanSend, get_nf_can_send, set_nf_can_send, 0, 57, 1, 0, false,
//             capNtfnPtr, get_nf_ptr, set_nf_ptr, 0, 0, 39, 0, true
//         },
//         new_reply_cap, cap_tag::CapReplyCap as usize => {
//             capReplyCanGrant, get_reply_can_grant, set_reply_can_grant, 0, 1, 1, 0, false,
//             capReplyMaster, get_reply_master, set_reply_master, 0, 0, 1, 0, false,
//             capTCBPtr, get_reply_tcb_ptr, set_reply_tcb_ptr, 1, 0, 64, 0, false
//         },
//         new_cnode_cap, cap_tag::CapCNodeCap as usize => {
//             capCNodeRadix, get_cnode_radix, set_cnode_radix, 0, 47, 6, 0, false,
//             capCNodeGuardSize, get_cnode_guard_size, set_cnode_guard_size, 0, 53, 6, 0, false,
//             capCNodeGuard, get_cnode_guard, set_cnode_guard, 1, 0, 64, 0, false,
//             capCNodePtr, get_cnode_ptr, set_cnode_ptr, 0, 0, 38, 1, true
//         },
//         new_thread_cap, cap_tag::CapThreadCap as usize => {
//             capTCBPtr, get_tcb_ptr, set_tcb_ptr, 0, 0, 39, 0, true
//         },
//         new_irq_control_cap, cap_tag::CapIrqControlCap as usize => {},
//         new_irq_handler_cap, cap_tag::CapIrqHandlerCap as usize => {
//             capIRQ, get_irq_handler, set_irq_handler, 1, 0, 12, 0, false
//         },
//         new_zombie_cap, cap_tag::CapZombieCap as usize => {
//             capZombieID, get_zombie_id, set_zombie_id, 1, 0, 64, 0, false,
//             capZombieType, get_zombie_type, set_zombie_type, 0, 0, 7, 0, false
//         },
//         new_domain_cap, cap_tag::CapDomainCap as usize => {}
//     }
// }

/// cap 的公用方法
impl cap_t {
    pub fn update_data(&self, preserve: bool, new_data: usize) -> Self {
        if self.isArchCap() {
            return self.clone();
        }
        match self.get_cap_type() {
            cap_tag::cap_endpoint_cap => {
                if !preserve && self.get_ep_badge() == 0 {
                    let mut new_cap = self.clone();
                    new_cap.set_ep_badge(new_data);
                    new_cap
                } else {
                    cap_t::new_null_cap()
                }
            }

            cap_tag::cap_notification_cap => {
                if !preserve && self.get_nf_badge() == 0 {
                    let mut new_cap = self.clone();
                    new_cap.set_nf_badge(new_data);
                    new_cap
                } else {
                    cap_t::new_null_cap()
                }
            }

            cap_tag::cap_cnode_cap => {
                let w = CNodeCapData::new(new_data);
                let guard_size = w.get_guard_size();
                if guard_size + self.get_cnode_radix() > wordBits {
                    return cap_t::new_null_cap();
                }
                let guard = w.get_guard() & MASK!(guard_size);
                let mut new_cap = self.clone();
                new_cap.set_cnode_guard(guard);
                new_cap.set_cnode_guard_size(guard_size);
                new_cap
            }
            _ => self.clone(),
        }
    }

    pub fn get_cap_type(&self) -> u64 {
        self.get_type() as u64
    }

    pub fn get_cap_size_bits(&self) -> usize {
        match self.get_cap_type() {
            cap_tag::cap_untyped_cap => self.get_untyped_block_size(),
            cap_tag::cap_endpoint_cap => seL4_EndpointBits,
            cap_tag::cap_notification_cap => seL4_NotificationBits,
            cap_tag::cap_cnode_cap => self.get_cnode_radix() + seL4_SlotBits,
            cap_tag::cap_page_table_cap => PT_SIZE_BITS,
            cap_tag::cap_reply_cap => seL4_ReplyBits,
            _ => 0,
        }
    }

    pub fn get_cap_is_physical(&self) -> bool {
        matches!(
            self.get_cap_type(),
            cap_tag::cap_untyped_cap
                | cap_tag::cap_endpoint_cap
                | cap_tag::cap_notification_cap
                | cap_tag::cap_cnode_cap
                | cap_tag::cap_frame_cap
                | cap_tag::cap_asid_pool_cap
                | cap_tag::cap_page_table_cap
                | cap_tag::cap_zombie_cap
                | cap_tag::cap_thread_cap
        )
    }

    pub fn isArchCap(&self) -> bool {
        self.get_cap_type() as usize % 2 != 0
    }
}

/// 判断两个cap指向的内核对象是否是同一个内存区域
pub fn same_region_as(cap1: &cap_t, cap2: &cap_t) -> bool {
    match cap1.get_cap_type() {
        cap_tag::cap_untyped_cap => {
            if cap2.get_cap_is_physical() {
                let aBase = cap1.get_untyped_ptr();
                let bBase = cap2.get_cap_ptr();

                let aTop = aBase + MASK!(cap1.get_untyped_block_size());
                let bTop = bBase + MASK!(cap2.get_cap_size_bits());
                return (aBase <= bBase) && (bTop <= aTop) && (bBase <= bTop);
            }

            false
        }

        cap_tag::cap_endpoint_cap
        | cap_tag::cap_notification_cap
        | cap_tag::cap_page_table_cap
        | cap_tag::cap_asid_pool_cap
        | cap_tag::cap_thread_cap => {
            if cap2.get_cap_type() == cap1.get_cap_type() {
                return cap1.get_cap_ptr() == cap2.get_cap_ptr();
            }
            false
        }
        cap_tag::cap_asid_control_cap | cap_tag::cap_domain_cap => {
            if cap2.get_cap_type() == cap1.get_cap_type() {
                return true;
            }
            false
        }
        cap_tag::cap_cnode_cap => {
            if cap2.get_cap_type() == cap_tag::cap_cnode_cap {
                return (cap1.get_cnode_ptr() == cap2.get_cnode_ptr())
                    && (cap1.get_cnode_radix() == cap2.get_cnode_radix());
            }
            false
        }
        cap_tag::cap_irq_control_cap => {
            matches!(
                cap2.get_cap_type(),
                cap_tag::cap_irq_control_cap | cap_tag::cap_irq_handler_cap
            )
        }
        cap_tag::cap_irq_handler_cap => {
            if cap2.get_cap_type() == cap_tag::cap_irq_handler_cap {
                return cap1.get_irq_handler() == cap2.get_irq_handler();
            }
            false
        }
        _ => false,
    }
}

/// Check whether two caps point to the same kernel object, if not,
///  whether two kernel objects use the same memory region.
///
/// A special case is that cap2 is a untyped_cap derived from cap1, in this case, cap1 will excute
/// setUntypedCapAsFull, so you can assume cap1 and cap2 are different.
pub fn same_object_as(cap1: &cap_t, cap2: &cap_t) -> bool {
    if cap1.get_cap_type() == cap_tag::cap_untyped_cap {
        return false;
    }
    if cap1.get_cap_type() == cap_tag::cap_irq_control_cap
        && cap2.get_cap_type() == cap_tag::cap_irq_handler_cap
    {
        return false;
    }
    if cap1.isArchCap() && cap2.isArchCap() {
        return arch_same_object_as(cap1, cap2);
    }
    same_region_as(cap1, cap2)
}

/// 判断一个`capability`是否是可撤销的
pub fn is_cap_revocable(derived_cap: &cap_t, src_cap: &cap_t) -> bool {
    if derived_cap.isArchCap() {
        return false;
    }

    match derived_cap.get_cap_type() {
        cap_tag::cap_endpoint_cap => {
            assert_eq!(src_cap.get_cap_type(), cap_tag::cap_endpoint_cap);
            derived_cap.get_ep_badge() != src_cap.get_ep_badge()
        }

        cap_tag::cap_notification_cap => {
            assert_eq!(src_cap.get_cap_type(), cap_tag::cap_notification_cap);
            derived_cap.get_nf_badge() != src_cap.get_nf_badge()
        }

        cap_tag::cap_irq_handler_cap => src_cap.get_cap_type() == cap_tag::cap_irq_control_cap,

        cap_tag::cap_untyped_cap => true,

        _ => false,
    }
}
