use core::intrinsics::likely;

use log::debug;
use sel4_common::{
    arch::{getMaxTicksToUs, ticksToUs},
    platform::time_def::{ticks_t, time_t},
    structures::exception_t,
    structures_gen::{cap, cap_tag, notification, notification_t},
    utils::{convert_to_mut_type_ref, global_ops},
};

use crate::{get_currenct_thread, ksCurSC, ksSchedulerAction, rescheduleRequired, tcb, tcb_t};

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
    #[inline]
    pub fn get_ptr(&self) -> usize {
        self as *const sched_context_t as usize
    }
    #[inline]
    pub fn sc_active(&self) -> bool {
        self.scRefillMax > 0
    }
    #[inline]
    pub fn sc_sporadic(&self) -> bool {
        self.get_ptr() != 0 && self.sc_active() && self.scSporadic
    }
    #[inline]
    pub fn postpone(&self) {
        // TODO: MCS
    }
    #[inline]
    pub fn refill_unblock_check(&mut self) {
        // TODO: MCS
    }
    #[inline]
    pub fn refill_ready(&mut self) -> bool {
        // TODO: MCS
        true
    }
    #[inline]
    pub fn refill_sufficient(&mut self, usage: ticks_t) -> bool {
        // TODO: MCS
        true
    }

    pub fn schedContext_resume(&mut self) {
        if likely(self.get_ptr() != 0) && convert_to_mut_type_ref::<tcb_t>(self.scTcb).is_runnable()
        {
            if !(self.refill_ready() && self.refill_sufficient(0)) {
                assert!(
                    convert_to_mut_type_ref::<tcb_t>(self.scTcb)
                        .tcbState
                        .get_tcbQueued()
                        == 0
                );
                self.postpone();
            }
        }
    }
    pub fn schedContext_bindTCB(&mut self, tcb: &mut tcb_t) {
        tcb.tcbSchedContext = self.get_ptr();
        self.scTcb = tcb.get_ptr();
        if self.sc_sporadic() && self.sc_active() && self.get_ptr() != unsafe { ksCurSC } {
            self.refill_unblock_check()
        }
        self.schedContext_resume();
        if tcb.is_runnable() {
            tcb.sched_enqueue();
            rescheduleRequired();
        }
    }
    pub fn schedContext_unbindTCB(&mut self, tcb: &mut tcb_t) {
        if tcb.is_current() {
            rescheduleRequired();
        }
        convert_to_mut_type_ref::<tcb_t>(self.scTcb).sched_dequeue();
        convert_to_mut_type_ref::<tcb_t>(self.scTcb).Release_Remove();
        convert_to_mut_type_ref::<tcb_t>(self.scTcb).tcbSchedContext = 0;
        self.scTcb = 0;
    }
    pub fn schedContext_unbindAllTCBs(&mut self) {
        if self.scTcb != 0 {
            self.schedContext_unbindTCB(convert_to_mut_type_ref::<tcb_t>(self.scTcb));
        }
    }
    pub fn schedContext_donate(&mut self, to: &mut tcb_t) {
        if self.scTcb != 0 {
            let from: &mut tcb_t = convert_to_mut_type_ref::<tcb_t>(self.scTcb);
            from.sched_dequeue();
            from.tcbSchedContext = 0;
            if from.is_current() || from.get_ptr() == unsafe { ksSchedulerAction } {
                rescheduleRequired();
            }
        }
        self.scTcb = to.get_ptr();
        to.tcbSchedContext = self.get_ptr()
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
    pub fn setConsumed(&mut self, buffer: usize) {
        // TODO: MCS
    }
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
