use core::intrinsics::unlikely;

use log::debug;
use sel4_common::{
    arch::{usToTicks, MessageLabel},
    platform::time_def::time_t,
    println,
    sel4_config::{seL4_IllegalOperation, seL4_RangeError, seL4_TruncatedMessage, TIME_ARG_SIZE},
    structures::{exception_t, seL4_IPCBuffer},
    structures_gen::{cap, cap_sched_context_cap, cap_sched_control_cap, cap_tag},
    utils::{convert_to_mut_type_ref, global_ops},
};
use sel4_cspace::interface::cte_t;
use sel4_task::{
    get_currenct_thread,
    sched_context::{
        refill_absolute_max, sched_context, sched_context_t, MAX_PERIOD_US, MIN_BUDGET,
        MIN_BUDGET_US, MIN_REFILLS,
    },
    set_thread_state, ThreadState,
};

use crate::{
    kernel::boot::{current_extra_caps, current_syscall_error},
    syscall::{get_syscall_arg, invocation::invoke_sched::invokeSchedControl_ConfigureFlags},
};

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
    match inv_label {
        MessageLabel::SchedControlConfigureFlags => {
            if global_ops!(current_extra_caps.excaprefs[0] == 0) {
                debug!("SchedControl_ConfigureFlags: Truncated message.");
                unsafe {
                    current_syscall_error._type = seL4_TruncatedMessage;
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }

            if length < (TIME_ARG_SIZE * 2) + 3 {
                debug!("SchedControl_configureFlags: truncated message.");
                unsafe {
                    current_syscall_error._type = seL4_TruncatedMessage;
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }

            let budget_us: time_t = get_syscall_arg(0, buffer);
            let budget_ticks = usToTicks(budget_us);
            let period_us = get_syscall_arg(TIME_ARG_SIZE, buffer);
            let period_ticks = usToTicks(period_us);
            let extra_refills = get_syscall_arg(TIME_ARG_SIZE * 2, buffer);
            let badge = get_syscall_arg(TIME_ARG_SIZE * 2 + 1, buffer);
            let flags = get_syscall_arg(TIME_ARG_SIZE * 2 + 2, buffer);

            let targetCap =
                &convert_to_mut_type_ref::<cte_t>(unsafe { current_extra_caps.excaprefs[0] })
                    .capability;
            if unlikely(targetCap.get_tag() != cap_tag::cap_sched_context_cap) {
                debug!("SchedControl_ConfigureFlags: budget out of range.");
                unsafe {
                    current_syscall_error._type = seL4_RangeError;
                    current_syscall_error.rangeErrorMin = MIN_BUDGET_US();
                    current_syscall_error.rangeErrorMax = MAX_PERIOD_US();
                    return exception_t::EXCEPTION_SYSCALL_ERROR;
                }
            }
            if budget_us > MAX_PERIOD_US() || budget_ticks < MIN_BUDGET() {
                debug!("SchedControl_ConfigureFlags: budget out of range.");
                unsafe {
                    current_syscall_error._type = seL4_RangeError;
                    current_syscall_error.rangeErrorMin = MIN_BUDGET_US();
                    current_syscall_error.rangeErrorMax = MAX_PERIOD_US();
                }

                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }

            if period_us > MAX_PERIOD_US() || period_ticks < MIN_BUDGET() {
                debug!("SchedControl_ConfigureFlags: period out of range.");
                unsafe {
                    current_syscall_error._type = seL4_RangeError;
                    current_syscall_error.rangeErrorMin = MIN_BUDGET_US();
                    current_syscall_error.rangeErrorMax = MAX_PERIOD_US();
                }

                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }

            if budget_ticks > period_ticks {
                debug!("SchedControl_ConfigureFlags: budget must be <= period");
                unsafe {
                    current_syscall_error._type = seL4_RangeError;
                    current_syscall_error.rangeErrorMin = MIN_BUDGET_US();
                    current_syscall_error.rangeErrorMax = period_us;
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }

            if extra_refills + MIN_REFILLS
                > refill_absolute_max(cap::cap_sched_context_cap(&targetCap))
            {
                unsafe {
                    current_syscall_error._type = seL4_RangeError;
                    current_syscall_error.rangeErrorMin = 0;
                    current_syscall_error.rangeErrorMax =
                        refill_absolute_max(cap::cap_sched_context_cap(&targetCap)) - MIN_REFILLS;
                    debug!(
                        "Max refills invalid, got {}, max {}",
                        extra_refills, current_syscall_error.rangeErrorMax
                    );
                }
                return exception_t::EXCEPTION_SYSCALL_ERROR;
            }
            set_thread_state(get_currenct_thread(), ThreadState::ThreadStateRestart);
            return invokeSchedControl_ConfigureFlags(
                convert_to_mut_type_ref::<sched_context_t>(
                    cap::cap_sched_context_cap(&targetCap).get_capSCPtr() as usize,
                ),
                capability.get_core() as usize,
                budget_ticks,
                period_ticks,
                extra_refills + MIN_REFILLS,
                badge,
                flags,
            );
        }
        _ => {
            debug!("SchedControl invocation: Illegal operation attempted.");
            unsafe {
                current_syscall_error._type = seL4_IllegalOperation;
            }
        }
    }
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
