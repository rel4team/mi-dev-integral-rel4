use sel4_common::{
    platform::time_def::ticks_t,
    structures::exception_t,
    structures_gen::{cap, cap_tag},
    utils::convert_to_mut_type_ref,
};
use sel4_task::{sched_context::sched_context, tcb_t};

pub fn invokeSchedContext_UnbindObject(sc: &mut sched_context, capability: cap) -> exception_t {
    match capability.get_tag() {
        cap_tag::cap_thread_cap => {
            sc.schedContext_unbindTCB(convert_to_mut_type_ref::<tcb_t>(sc.scTcb));
        }
        cap_tag::cap_notification_cap => {
            sc.schedContext_unbindNtfn();
        }
        _ => {
            panic!("invalid cap type");
        }
    }
    exception_t::EXCEPTION_NONE
}

pub fn invokeSchedContext_Bind(sc: &mut sched_context) -> exception_t {
    // TODO: MCS
    exception_t::EXCEPTION_NONE
}
pub fn invokeSchedContext_Unbind(sc: &mut sched_context) -> exception_t {
    // TODO: MCS
    exception_t::EXCEPTION_NONE
}
pub fn invokeSchedContext_Consumed(sc: &mut sched_context) -> exception_t {
    // TODO: MCS
    exception_t::EXCEPTION_NONE
}
pub fn invokeSchedContext_YieldTo(sc: &mut sched_context) -> exception_t {
    // TODO: MCS
    exception_t::EXCEPTION_NONE
}
pub fn invokeSchedControl_ConfigureFlags(
    sc: &mut sched_context,
    core: usize,
    budget: ticks_t,
    period: ticks_t,
    max_refills: usize,
    badge: usize,
    flags: usize,
) -> exception_t {
    // TODO: MCS
    exception_t::EXCEPTION_NONE
}
