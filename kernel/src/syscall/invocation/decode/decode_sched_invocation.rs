use log::debug;
use sel4_common::{arch::MessageLabel, println, structures::{exception_t, seL4_IPCBuffer}, structures_gen::{cap_sched_context_cap, cap_sched_control_cap}, utils::global_ops};
use sel4_task::sched_context::sched_context;

use crate::kernel::boot::current_extra_caps;

pub fn decode_sched_context_invocation(
	inv_label: MessageLabel,
    capability: &cap_sched_context_cap,
    buffer: &seL4_IPCBuffer,
) -> exception_t {
	println!("go into decode sched context invocation");
    exception_t::EXCEPTION_NONE
}
pub fn decode_sched_control_invocation(
	inv_label: MessageLabel,
    length: usize,
    capability: &cap_sched_control_cap,
    buffer: &seL4_IPCBuffer,
) -> exception_t {
	println!("go into decode sched control invocation");
    exception_t::EXCEPTION_NONE
}
pub fn decodeSchedContext_UnbindObject(sc: &mut sched_context) -> exception_t {
    // TODO: MCS
    unimplemented!("MCS");
    if global_ops!(current_extra_caps.excaprefs[0] == 0) {
        debug!("")
    }
    exception_t::EXCEPTION_NONE
}
pub fn decodeSchedContext_Bind(sc: &mut sched_context) -> exception_t {
    unimplemented!("MCS");
    // TODO: MCS
    exception_t::EXCEPTION_NONE
}
pub fn decodeSchedContext_YieldTo(sc: &mut sched_context) {
    unimplemented!("MCS");
    // TODO: MCS
}
