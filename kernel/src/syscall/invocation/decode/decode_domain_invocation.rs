use core::intrinsics::unlikely;

use log::debug;
use sel4_common::structures_gen::{cap, cap_tag, cap_thread_cap};
use sel4_common::{
    arch::MessageLabel,
    sel4_config::*,
    structures::{exception_t, seL4_IPCBuffer},
    utils::convert_to_mut_type_ref,
};
use sel4_task::{get_currenct_thread, set_thread_state, tcb_t, ThreadState};

use crate::{
    kernel::boot::{current_syscall_error, get_extra_cap_by_index},
    syscall::get_syscall_arg,
};

pub fn decode_domain_invocation(
    invLabel: MessageLabel,
    length: usize,
    buffer: &seL4_IPCBuffer,
) -> exception_t {
    if invLabel != MessageLabel::DomainSetSet {
        unsafe {
            current_syscall_error._type = seL4_IllegalOperation;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if length == 0 {
        debug!("Domain Configure: Truncated message.");
        unsafe {
            current_syscall_error._type = seL4_TruncatedMessage;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let domain = get_syscall_arg(0, buffer);
    if domain >= 1 {
        debug!("Domain Configure: invalid domain ({} >= 1).", domain);
        unsafe {
            current_syscall_error._type = seL4_InvalidArgument;
            current_syscall_error.invalidArgumentNumber = 0;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    if get_extra_cap_by_index(0).is_none() {
        debug!("Domain Configure: Truncated message.");
        unsafe {
            current_syscall_error._type = seL4_TruncatedMessage;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }
    let thread_cap = get_extra_cap_by_index(0).unwrap().capability;
    if unlikely(thread_cap.get_tag() != cap_tag::cap_thread_cap) {
        debug!("Domain Configure: thread cap required.");
        unsafe {
            current_syscall_error._type = seL4_InvalidArgument;
            current_syscall_error.invalidArgumentNumber = 1;
        }
        return exception_t::EXCEPTION_SYSCALL_ERROR;
    }

    set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
    convert_to_mut_type_ref::<tcb_t>(unsafe { core::mem::transmute::<cap, cap_thread_cap>(thread_cap) }.get_capTCBPtr() as usize).set_domain(domain);
    exception_t::EXCEPTION_NONE
}
