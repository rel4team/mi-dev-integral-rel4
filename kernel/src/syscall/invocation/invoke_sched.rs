use sel4_common::{
    platform::time_def::ticks_t,
    structures::exception_t,
    structures_gen::{cap, cap_tag},
    utils::convert_to_mut_type_ref,
};
use sel4_task::{
    checkBudget, commitTime, ksCurSC, ksCurThread, possible_switch_to, rescheduleRequired,
    sched_context::sched_context, sched_context::MIN_REFILLS, seL4_SchedContext_Sporadic, tcb_t,
};

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
    target: &mut sched_context,
    core: usize,
    budget: ticks_t,
    period: ticks_t,
    max_refills: usize,
    badge: usize,
    flags: usize,
) -> exception_t {
    target.scBadge = badge;
    target.scSporadic = (flags & seL4_SchedContext_Sporadic) != 0;

    if target.scTcb != 0 {
        /* remove from scheduler */
        convert_to_mut_type_ref::<tcb_t>(target.scTcb).Release_Remove();
        convert_to_mut_type_ref::<tcb_t>(target.scTcb).sched_dequeue();
        /* bill the current consumed amount before adjusting the params */
        if unsafe { ksCurSC } == target.get_ptr() {
            assert!(checkBudget());
            commitTime();
        }
    }

    if budget == period {
        target.refill_new(MIN_REFILLS, budget, 0);
    } else if target.scRefillMax > 0
        && target.scTcb != 0
        && convert_to_mut_type_ref::<tcb_t>(target.scTcb).is_runnable()
    {
        target.refill_update(period, budget, max_refills);
    } else {
        /* the scheduling context isn't active - it's budget is not being used, so
         * we can just populate the parameters from now */
        target.refill_new(max_refills, budget, period);
    }

    assert!(target.scRefillMax > 0);
    if target.scTcb != 0 {
        target.schedContext_resume();
        if convert_to_mut_type_ref::<tcb_t>(target.scTcb).is_runnable()
            && target.scTcb != unsafe { ksCurThread }
        {
            possible_switch_to(convert_to_mut_type_ref::<tcb_t>(target.scTcb));
        }
        if target.scTcb == unsafe { ksCurThread } {
            rescheduleRequired();
        }
    }
    exception_t::EXCEPTION_NONE
}
