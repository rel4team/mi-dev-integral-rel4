use sel4_common::{
    arch::{getMaxTicksToUs, ticksToUs},
    platform::time_def::{ticks_t, time_t},
    structures::exception_t,
    structures_gen::{cap, cap_tag, notification_t},
    utils::convert_to_mut_type_ref,
};

use crate::{tcb, tcb_t};

pub type sched_context_t = sched_context;
#[repr(C)]
#[derive(Debug, Clone)]
pub struct sched_context {
    // TODO: MCS
    pub scPeriod: ticks_t,
    pub scConsumed: ticks_t,
    pub scCore: usize,
    pub scTcb: usize,
    pub scReply: usize,
    pub scNotification: usize,
    pub scBadge: usize,
    pub scYieldFrom: usize,
    pub scRefillMax: usize,
    pub scRefillHead: usize,
    pub scRefillTail: usize,
    pub scSporadic: bool,
}
impl sched_context {
    pub fn invokeSchedContext_UnbindObject(&mut self, capability: cap) -> exception_t {
        match capability.get_tag() {
            cap_tag::cap_thread_cap => {
                self.schedContext_unbindTCB(convert_to_mut_type_ref::<tcb_t>(self.scTcb));
            }
            cap_tag::cap_notification_cap => {
                self.schedContext_unbindNtfn();
            }
            _ => {
                panic!("invalid cap type");
            }
        }
        exception_t::EXCEPTION_NONE
    }
    pub fn decodeSchedContext_UnbindObject(&mut self) -> exception_t {
        exception_t::EXCEPTION_NONE
    }
    pub fn invokeSchedContext_Bind(&mut self) -> exception_t {
        exception_t::EXCEPTION_NONE
    }
    pub fn decodeSchedContext_Bind(&mut self) -> exception_t {
        exception_t::EXCEPTION_NONE
    }
    pub fn invokeSchedContext_Unbind(&mut self) -> exception_t {
        exception_t::EXCEPTION_NONE
    }
    pub fn invokeSchedContext_Consumed(&mut self) -> exception_t {
        exception_t::EXCEPTION_NONE
    }
    pub fn invokeSchedContext_YieldTo(&mut self) -> exception_t {
        exception_t::EXCEPTION_NONE
    }
    pub fn schedContext_resume(&mut self) {}
    pub fn decodeSchedContext_YieldTo(&mut self) {}
    pub fn schedContext_bindTCB(&mut self, tcb: &mut tcb_t) {}
    pub fn schedContext_unbindTCB(&mut self, tcb: &mut tcb_t) {}
    pub fn schedContext_unbindAllTCBs(&mut self) {}
    pub fn schedContext_donate(&mut self, to: &mut tcb_t) {}
    pub fn schedContext_bindNtfn(&mut self, ntfn: &mut notification_t) {}
    pub fn schedContext_unbindNtfn(&mut self) {}
    pub fn setConsumed(&mut self, buffer: usize) {}
    pub fn schedContext_updateConsumed(&mut self) -> time_t {
        let consumed: ticks_t = self.scConsumed;
        if consumed >= getMaxTicksToUs() {
            self.scConsumed -= getMaxTicksToUs();
            return ticksToUs(getMaxTicksToUs());
        } else {
            self.scConsumed = 0;
            return ticksToUs(consumed);
        }
    }
}
