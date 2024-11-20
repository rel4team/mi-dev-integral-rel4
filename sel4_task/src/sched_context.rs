use log::debug;
use sel4_common::{
    arch::{getMaxTicksToUs, ticksToUs},
    platform::time_def::{ticks_t, time_t},
    structures::exception_t,
    structures_gen::{cap, cap_tag, notification, notification_t},
    utils::{convert_to_mut_type_ref, global_ops},
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
    pub fn schedContext_resume(&mut self) {}
    pub fn schedContext_bindTCB(&mut self, tcb: &mut tcb_t) {}
    pub fn schedContext_unbindTCB(&mut self, tcb: &mut tcb_t) {}
    pub fn schedContext_unbindAllTCBs(&mut self) {}
    pub fn schedContext_donate(&mut self, to: &mut tcb_t) {
        if self.scTcb != 0 {
            let from: &mut tcb_t = convert_to_mut_type_ref::<tcb_t>(self.scTcb);
            from.sched_dequeue();
            from.tcbSchedContext = 0;
        }
    }
    pub fn schedContext_bindNtfn(&mut self, ntfn: &mut notification_t) {
        ntfn.set_ntfnSchedContext(self as *mut _ as u64);
        self.scNotification = ntfn as *mut _ as usize;
    }
    pub fn schedContext_unbindNtfn(&mut self) {
        if self.scNotification != 0 {
            convert_to_mut_type_ref::<notification>(self.scNotification).set_ntfnSchedContext(0);
            self.scNotification = 0;
        }
    }
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
