use crate::MASK;
use crate::{
    config::seL4_MsgLengthBits,
    syscall::{slowpath, SysCall, SysReplyRecv},
};
// #[cfg(target_arch="riscv64")]
// use crate::ffi::fastpath_restore;
use core::arch::asm;
use core::intrinsics::{likely, unlikely};
use sel4_common::arch::msgRegister;
use sel4_common::{
    fault::*,
    message_info::*,
    sel4_config::*,
    utils::{convert_to_mut_type_ref, convert_to_option_mut_type_ref},
};
use sel4_cspace::interface::*;
use sel4_ipc::*;
use sel4_task::*;
use sel4_vspace::*;

#[inline]
#[no_mangle]
pub fn lookup_fp(_cap: &cap_t, cptr: usize) -> cap_t {
    let mut cap = _cap.clone();
    let mut bits = 0;
    let mut guardBits: usize;
    let mut radixBits: usize;
    let mut cptr2: usize;
    let mut capGuard: usize;
    let mut radix: usize;
    let mut slot: *mut cte_t;
    if unlikely(!(cap.get_cap_type() == CapTag::CapCNodeCap)) {
        return cap_t::new_null_cap();
    }
    loop {
        guardBits = cap.get_cnode_guard_size();
        radixBits = cap.get_cnode_radix();
        cptr2 = cptr << bits;
        capGuard = cap.get_cnode_guard();
        if likely(guardBits != 0) && unlikely(cptr2 >> (wordBits - guardBits) != capGuard) {
            return cap_t::new_null_cap();
        }

        radix = cptr2 << guardBits >> (wordBits - radixBits);
        slot = unsafe { (cap.get_cnode_ptr() as *mut cte_t).add(radix) };
        cap = unsafe { (*slot).cap };
        bits += guardBits + radixBits;

        if likely(!(bits < wordBits && cap.get_cap_type() == CapTag::CapCNodeCap)) {
            break;
        }
    }
    if bits > wordBits {
        return cap_t::new_null_cap();
    }
    return cap;
}

#[inline]
#[no_mangle]
pub fn thread_state_ptr_mset_blockingObject_tsType(
    ptr: &mut thread_state_t,
    ep: usize,
    tsType: usize,
) {
    (*ptr).words[0] = ep | tsType;
}

#[inline]
#[no_mangle]
pub fn endpoint_ptr_mset_epQueue_tail_state(ptr: *mut endpoint_t, tail: usize, state: usize) {
    unsafe {
        (*ptr).words[0] = tail | state;
    }
}

#[inline]
#[no_mangle]
pub fn switchToThread_fp(thread: *mut tcb_t, vroot: *mut PTE, stored_hw_asid: PTE) {
    let asid = stored_hw_asid.0;
    unsafe {
        #[cfg(target_arch = "riscv64")]
        setVSpaceRoot(pptr_to_paddr(vroot as usize), asid);
        #[cfg(target_arch = "aarch64")]
        setCurrentUserVSpaceRoot(ttbr_new(asid, pptr_to_paddr(vroot as usize)));
        // panic!("switchToThread_fp");
        // ksCurThread = thread as usize;
        set_current_thread(&*thread);
    }
}

#[inline]
#[no_mangle]
pub fn mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(
    ptr: &mut mdb_node_t,
    mdbNext: usize,
    mdbRevocable: usize,
    mdbFirstBadged: usize,
) {
    ptr.words[1] = mdbNext | (mdbRevocable << 1) | mdbFirstBadged;
}

#[inline]
#[no_mangle]
pub fn isValidVTableRoot_fp(cap: &cap_t) -> bool {
    // cap_capType_equals(cap, cap_page_table_cap) && cap.get_pt_is_mapped() != 0
    cap.get_cap_type() == CapTag::CapPageTableCap && cap.get_pt_is_mapped() != 0
}

#[inline]
#[no_mangle]
pub fn fastpath_mi_check(msgInfo: usize) -> bool {
    (msgInfo & MASK!(seL4_MsgLengthBits + seL4_MsgExtraCapBits)) > 4
}

#[inline]
#[no_mangle]
pub fn fastpath_copy_mrs(length: usize, src: &mut tcb_t, dest: &mut tcb_t) {
    dest.tcbArch
        .copy_range(&src.tcbArch, msgRegister[0]..msgRegister[0] + length);
}

// #[inline]
// #[no_mangle]
// pub fn fastpath_restore(badge: usize, msgInfo: usize, cur_thread: *mut tcb_t) {
//     let cur_thread_regs = unsafe { (*cur_thread).tcbArch.get_register().as_ptr() as usize };
//     extern "C" {
//         pub fn __restore_fp(badge: usize, msgInfo: usize, cur_thread_reg: usize);
//         fn fastpath_restore(badge: usize, msgInfo: usize, cur_thread: usize);
//     }
//     unsafe {
//         __restore_fp(badge, msgInfo, cur_thread_regs);
//     }
// }
#[inline]
#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub fn fastpath_restore(badge: usize, msgInfo: usize, cur_thread: *mut tcb_t) {
    unsafe {
        (*cur_thread).tcbArch.load_thread_local();
        asm!(
            "mov     sp, {}                     \n",
            /* Restore thread's SPSR, LR, and SP */
            "ldp     x21, x22, [sp, #31 * 8]  \n",
            "ldr     x23, [sp, #33 * 8]     \n",
            "msr     sp_el0, x21                \n",
            // #ifdef CONFIG_ARM_HYPERVISOR_SUPPORT
            // 		"msr     elr_el2, x22               \n"
            // 		"msr     spsr_el2, x23              \n"
            // #else
            "msr     elr_el1, x22               \n",
            "msr     spsr_el1, x23              \n",
            // #endif

            /* Restore remaining registers */
            "ldp     x2,  x3,  [sp, #16 * 1]    \n",
            "ldp     x4,  x5,  [sp, #16 * 2]    \n",
            "ldp     x6,  x7,  [sp, #16 * 3]    \n",
            "ldp     x8,  x9,  [sp, #16 * 4]    \n",
            "ldp     x10, x11, [sp, #16 * 5]    \n",
            "ldp     x12, x13, [sp, #16 * 6]    \n",
            "ldp     x14, x15, [sp, #16 * 7]    \n",
            "ldp     x16, x17, [sp, #16 * 8]    \n",
            "ldp     x18, x19, [sp, #16 * 9]    \n",
            "ldp     x20, x21, [sp, #16 * 10]   \n",
            "ldp     x22, x23, [sp, #16 * 11]   \n",
            "ldp     x24, x25, [sp, #16 * 12]   \n",
            "ldp     x26, x27, [sp, #16 * 13]   \n",
            "ldp     x28, x29, [sp, #16 * 14]   \n",
            "ldr     x30, [sp, #30 * 8]           \n",
            "eret                                 ",
            in(reg) (*cur_thread).tcbArch.raw_ptr()
        );
    }
    panic!("unreachable")
}

#[inline]
#[no_mangle]
#[cfg(target_arch = "riscv64")]
pub fn fastpath_restore(badge: usize, msgInfo: usize, cur_thread: *mut tcb_t) {
    #[cfg(feature = "ENABLE_SMP")]
    {}
	extern "C" {
		pub fn __fastpath_restore(badge: usize, msgInfo: usize, cur_thread_reg: usize);
	}
	include_str!("./fastpath_restore.S");
	unsafe {
		__fastpath_restore(badge,msgInfo,(*cur_thread).tcbArch.raw_ptr());
	}
	panic!("unreachable")
}

#[inline]
#[no_mangle]
pub fn fastpath_call(cptr: usize, msgInfo: usize) {
    let current = get_currenct_thread();
    let mut info = seL4_MessageInfo_t::from_word(msgInfo);
    let length = info.get_length();

    if fastpath_mi_check(msgInfo) || current.tcbFault.get_fault_type() != FaultType::NullFault {
        slowpath(SysCall as usize);
    }
    let ep_cap = lookup_fp(&current.get_cspace(tcbCTable).cap, cptr);
    if unlikely(
        !(ep_cap.get_cap_type() == CapTag::CapEndpointCap) || (ep_cap.get_ep_can_send() == 0),
    ) {
        slowpath(SysCall as usize);
    }
    let ep = convert_to_mut_type_ref::<endpoint_t>(ep_cap.get_ep_ptr());

    if unlikely(ep.get_state() != EPState::Recv) {
        slowpath(SysCall as usize);
    }

    let dest = convert_to_mut_type_ref::<tcb_t>(ep.get_queue_head());
    let new_vtable = dest.get_cspace(tcbVTable).cap;

    if unlikely(!isValidVTableRoot_fp(&new_vtable)) {
        slowpath(SysCall as usize);
    }

    let dom = 0;
    if unlikely(dest.tcbPriority < current.tcbPriority && !isHighestPrio(dom, dest.tcbPriority)) {
        slowpath(SysCall as usize);
    }
    if unlikely((ep_cap.get_ep_can_grant() == 0) && (ep_cap.get_ep_can_grant_reply() == 0)) {
        slowpath(SysCall as usize);
    }
    #[cfg(feature = "ENABLE_SMP")]
    if unlikely(get_currenct_thread().tcbAffinity != dest.tcbAffinity) {
        slowpath(SysCall as usize);
    }

    // debug!("enter fast path");

    ep.set_queue_head(dest.tcbEPNext);
    if unlikely(dest.tcbEPNext != 0) {
        convert_to_mut_type_ref::<tcb_t>(dest.tcbEPNext).tcbEPNext = 0;
    } else {
        ep.set_queue_tail(0);
        ep.set_state(EPState::Idle as usize);
    }

    current.tcbState.words[0] = ThreadState::ThreadStateBlockedOnReply as usize;

    let reply_slot = current.get_cspace_mut_ref(tcbReply);
    let caller_slot = dest.get_cspace_mut_ref(tcbCaller);
    let reply_can_grant = dest.tcbState.get_blocking_ipc_can_grant();

    caller_slot.cap = cap_t::new_reply_cap(reply_can_grant, 0, current.get_ptr());
    caller_slot.cteMDBNode.words[0] = reply_slot.get_ptr();
    mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(
        &mut reply_slot.cteMDBNode,
        caller_slot.get_ptr(),
        1,
        1,
    );
    fastpath_copy_mrs(length, current, dest);
    dest.tcbState.words[0] = ThreadState::ThreadStateRunning as usize;
    let cap_pd = new_vtable.get_pt_base_ptr() as *mut PTE;
    let stored_hw_asid: PTE = PTE(new_vtable.get_pt_mapped_asid());
    switchToThread_fp(dest as *mut tcb_t, cap_pd, stored_hw_asid);
    info.set_caps_unwrapped(0);
    let msgInfo1 = info.to_word();
    let badge = ep_cap.get_ep_badge();
    unsafe {
        fastpath_restore(badge, msgInfo1, get_currenct_thread());
    }
}

#[inline]
#[no_mangle]
pub fn fastpath_reply_recv(cptr: usize, msgInfo: usize) {
    // debug!("enter fastpath_reply_recv");
    let current = get_currenct_thread();
    let mut info = seL4_MessageInfo_t::from_word(msgInfo);
    let length = info.get_length();
    let fault_type = current.tcbFault.get_fault_type();

    if fastpath_mi_check(msgInfo) || fault_type != FaultType::NullFault {
        slowpath(SysReplyRecv as usize);
    }

    let ep_cap = lookup_fp(&current.get_cspace(tcbCTable).cap, cptr);

    if unlikely(ep_cap.get_cap_type() != CapTag::CapEndpointCap || ep_cap.get_ep_can_send() == 0) {
        slowpath(SysReplyRecv as usize);
    }

    if let Some(ntfn) =
        convert_to_option_mut_type_ref::<notification_t>(current.tcbBoundNotification)
    {
        if ntfn.get_state() == NtfnState::Active {
            slowpath(SysReplyRecv as usize);
        }
    }

    let ep = convert_to_mut_type_ref::<endpoint_t>(ep_cap.get_ep_ptr());
    if unlikely(ep.get_state() == EPState::Send) {
        slowpath(SysReplyRecv as usize);
    }

    let caller_slot = current.get_cspace_mut_ref(tcbCaller);
    let caller_cap = &caller_slot.cap;

    if unlikely(caller_cap.get_cap_type() != CapTag::CapReplyCap) {
        slowpath(SysReplyRecv as usize);
    }

    let caller = convert_to_mut_type_ref::<tcb_t>(caller_cap.get_reply_tcb_ptr());
    if unlikely(caller.tcbFault.get_fault_type() != FaultType::NullFault) {
        slowpath(SysReplyRecv as usize);
    }

    let new_vtable = &caller.get_cspace(tcbVTable).cap;

    if unlikely(!isValidVTableRoot_fp(new_vtable)) {
        slowpath(SysReplyRecv as usize);
    }

    let dom = 0;
    if unlikely(!isHighestPrio(dom, caller.tcbPriority)) {
        slowpath(SysReplyRecv as usize);
    }
    thread_state_ptr_mset_blockingObject_tsType(
        &mut current.tcbState,
        ep.get_ptr(),
        ThreadState::ThreadStateBlockedOnReceive as usize,
    );
    current
        .tcbState
        .set_blocking_ipc_can_grant(ep_cap.get_ep_can_grant());

    if let Some(ep_tail_tcb) = convert_to_option_mut_type_ref::<tcb_t>(ep.get_queue_tail()) {
        ep_tail_tcb.tcbEPNext = current.get_ptr();
        current.tcbEPPrev = ep_tail_tcb.get_ptr();
        current.tcbEPNext = 0;
    } else {
        current.tcbEPPrev = 0;
        current.tcbEPNext = 0;
        ep.set_queue_head(current.get_ptr());
    }
    endpoint_ptr_mset_epQueue_tail_state(
        ep as *mut endpoint_t,
        get_currenct_thread().get_ptr(),
        EPState_Recv,
    );

    unsafe {
        let node = convert_to_mut_type_ref::<cte_t>(caller_slot.cteMDBNode.get_prev());
        mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(&mut node.cteMDBNode, 0, 1, 1);
        caller_slot.cap = cap_t::new_null_cap();
        caller_slot.cteMDBNode = mdb_node_t::new(0, 0, 0, 0);
        fastpath_copy_mrs(length, current, caller);

        caller.tcbState.words[0] = ThreadState::ThreadStateRunning as usize;
        let cap_pd = new_vtable.get_pt_base_ptr() as *mut PTE;
        let stored_hw_asid: PTE = PTE(new_vtable.get_pt_mapped_asid());
        switchToThread_fp(caller, cap_pd, stored_hw_asid);
        info.set_caps_unwrapped(0);
        let msg_info1 = info.to_word();
        fastpath_restore(0, msg_info1, get_currenct_thread() as *mut tcb_t);
    }
}
