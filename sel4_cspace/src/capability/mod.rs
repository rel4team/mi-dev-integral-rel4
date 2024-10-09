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

use sel4_common::structures_gen::{cap, cap_Splayed, cap_null_cap, cap_tag};
use sel4_common::{sel4_config::*, MASK};

use crate::arch::arch_same_object_as;

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
pub trait cap_pub_func {
    fn update_data(&self, preserve: bool, new_data: u64) -> Self;
    fn get_cap_size_bits(&self) -> usize;
    fn get_cap_is_physical(&self) -> bool;
    fn isArchCap(&self) -> bool;
}
pub trait cap_arch_func {
    fn get_cap_ptr(&self) -> usize;
    fn is_vtable_root(&self) -> bool;
    fn is_valid_native_root(&self) -> bool;
    fn is_valid_vtable_root(&self) -> bool;
}

impl cap_pub_func for cap {
    fn update_data(&self, preserve: bool, new_data: u64) -> Self {
        if self.isArchCap() {
            return self.clone();
        }
        match self.splay() {
            cap_Splayed::endpoint_cap(data) => {
                if !preserve && data.get_capEPBadge() == 0 {
                    let mut new_cap = data.clone();
                    new_cap.set_capEPBadge(new_data);
                    new_cap.unsplay()
                } else {
                    cap_null_cap::new().unsplay()
                }
            }

            cap_Splayed::notification_cap(data) => {
                if !preserve && data.get_capNtfnBadge() == 0 {
                    let mut new_cap = data.clone();
                    new_cap.set_capNtfnBadge(new_data);
                    new_cap.unsplay()
                } else {
                    cap_null_cap::new().unsplay()
                }
            }

            cap_Splayed::cnode_cap(data) => {
                let w = CNodeCapData::new(new_data as usize);
                let guard_size = w.get_guard_size();
                if guard_size + data.get_capCNodeRadix() as usize > wordBits {
                    return cap_null_cap::new().unsplay();
                }
                let guard = w.get_guard() & MASK!(guard_size);
                let mut new_cap = data.clone();
                new_cap.set_capCNodeGuard(guard as u64);
                new_cap.set_capCNodeGuardSize(guard_size as u64);
                new_cap.unsplay()
            }
            _ => self.clone(),
        }
    }

    fn get_cap_size_bits(&self) -> usize {
        match self.splay() {
            cap_Splayed::untyped_cap(data) => data.get_capBlockSize() as usize,
            cap_Splayed::endpoint_cap(_) => seL4_EndpointBits,
            cap_Splayed::notification_cap(_) => seL4_NotificationBits,
            cap_Splayed::cnode_cap(data) => data.get_capCNodeRadix() as usize + seL4_SlotBits,
            cap_Splayed::page_table_cap(_) => PT_SIZE_BITS,
            cap_Splayed::reply_cap(_) => seL4_ReplyBits,
            _ => 0,
        }
    }

    fn get_cap_is_physical(&self) -> bool {
        matches!(
            self.get_tag(),
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

    fn isArchCap(&self) -> bool {
        self.get_tag() as usize % 2 != 0
    }
}

/// 判断两个cap指向的内核对象是否是同一个内存区域
pub fn same_region_as(cap1: &cap, cap2: &cap) -> bool {
    match cap1.splay() {
        cap_Splayed::untyped_cap(data1) => {
            if cap2.get_cap_is_physical() {
                let aBase = data1.get_capPtr() as usize;
                let bBase = cap2.get_cap_ptr();

                let aTop = aBase + MASK!(data1.get_capBlockSize());
                let bTop = bBase + MASK!(cap2.get_cap_size_bits());
                return (aBase <= bBase) && (bTop <= aTop) && (bBase <= bTop);
            }

            false
        }
        cap_Splayed::endpoint_cap(_)
        | cap_Splayed::notification_cap(_)
        | cap_Splayed::page_table_cap(_)
        | cap_Splayed::asid_pool_cap(_)
        | cap_Splayed::thread_cap(_) => {
            if cap2.get_tag() == cap1.get_tag() {
                return cap1.get_cap_ptr() == cap2.get_cap_ptr();
            }
            false
        }
        cap_Splayed::asid_control_cap(_) | cap_Splayed::domain_cap(_) => {
            if cap2.get_tag() == cap1.get_tag() {
                return true;
            }
            false
        }
        cap_Splayed::cnode_cap(data1) => match cap2.splay() {
            cap_Splayed::cnode_cap(data2) => {
                return (data1.get_capCNodePtr() == data2.get_capCNodePtr())
                    && (data1.get_capCNodeRadix() == data2.get_capCNodeRadix());
            }
            _ => return false,
        },
        cap_Splayed::irq_control_cap(_) => {
            matches!(
                cap2.get_tag(),
                cap_tag::cap_irq_control_cap | cap_tag::cap_irq_handler_cap
            )
        }
        cap_Splayed::irq_handler_cap(data1) => match cap2.splay() {
            cap_Splayed::irq_handler_cap(data2) => {
                return data1.get_capIRQ() == data2.get_capIRQ();
            }
            _ => return false,
        },
        _ => false,
    }
}

/// Check whether two caps point to the same kernel object, if not,
///  whether two kernel objects use the same memory region.
///
/// A special case is that cap2 is a untyped_cap derived from cap1, in this case, cap1 will excute
/// setUntypedCapAsFull, so you can assume cap1 and cap2 are different.
pub fn same_object_as(cap1: &cap, cap2: &cap) -> bool {
    if cap1.get_tag() == cap_tag::cap_untyped_cap {
        return false;
    }
    if cap1.get_tag() == cap_tag::cap_irq_control_cap
        && cap2.get_tag() == cap_tag::cap_irq_handler_cap
    {
        return false;
    }
    if cap1.isArchCap() && cap2.isArchCap() {
        return arch_same_object_as(cap1, cap2);
    }
    same_region_as(cap1, cap2)
}

/// 判断一个`capability`是否是可撤销的
pub fn is_cap_revocable(derived_cap: &cap, src_cap: &cap) -> bool {
    if derived_cap.isArchCap() {
        return false;
    }

    match derived_cap.splay() {
        cap_Splayed::endpoint_cap(data1) => match src_cap.splay() {
            cap_Splayed::endpoint_cap(data2) => {
                return data1.get_capEPBadge() != data2.get_capEPBadge()
            }
            _ => {
                assert_eq!(src_cap.get_tag(), cap_tag::cap_endpoint_cap);
                false
            }
        },

        cap_Splayed::notification_cap(data1) => match src_cap.splay() {
            cap_Splayed::notification_cap(data2) => {
                return data1.get_capNtfnBadge() != data2.get_capNtfnBadge()
            }
            _ => {
                assert_eq!(src_cap.get_tag(), cap_tag::cap_notification_cap);
                false
            }
        },

        cap_Splayed::irq_handler_cap(_) => src_cap.get_tag() == cap_tag::cap_irq_control_cap,

        cap_Splayed::untyped_cap(_) => true,

        _ => false,
    }
}
