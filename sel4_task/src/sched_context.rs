use core::{
    intrinsics::{likely, unlikely},
    mem::size_of,
};

use sel4_common::{
    arch::{getKernelWcetTicks, getKernelWcetUs, getMaxTicksToUs, ticksToUs, ArchReg::MsgInfo},
    message_info::seL4_MessageInfo_func,
    platform::time_def::{ticks_t, time_t},
    sel4_config::CONFIG_KERNEL_WCET_SCALE,
    shared_types_bf_gen::seL4_MessageInfo,
    structures_gen::{notification, notification_t},
    utils::convert_to_mut_type_ref,
};

use crate::{
    get_currenct_thread, ksCurSC, ksCurTime, ksReprogram, ksSchedulerAction, rescheduleRequired,
    tcb_t,
};

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
pub(crate) const MIN_REFILLS: usize = 2;
pub(crate) type refill_t = refill;
#[repr(C)]
#[derive(Debug, Clone)]
pub struct refill {
    rTime: ticks_t,
    rAmount: ticks_t,
}
pub fn MIN_BUDGET_US() -> time_t {
    2 * getKernelWcetUs() * CONFIG_KERNEL_WCET_SCALE
}
pub fn MIN_BUDGET() -> time_t {
    2 * getKernelWcetTicks() * CONFIG_KERNEL_WCET_SCALE
}

impl sched_context {
    #[inline]
    pub fn get_ptr(&self) -> usize {
        self as *const sched_context_t as usize
    }
    #[inline]
    pub fn is_round_robin(&self) -> bool {
        self.scPeriod == 0
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
        convert_to_mut_type_ref::<tcb_t>(self.scTcb).sched_dequeue();
        convert_to_mut_type_ref::<tcb_t>(self.scTcb).Release_Enqueue();
        unsafe { ksReprogram = true };
    }
    #[inline]
    fn refill_pop_head(&mut self) -> *mut refill_t {
        assert!(!self.refill_single());
        let prev_size = self.refill_size();
        let refill_res = self.refill_head();
        let refill_head = self.scRefillHead;
        self.scRefillHead = self.refill_next(refill_head);
        assert!(prev_size == self.refill_size() + 1);
        assert!(self.scRefillHead < self.scRefillMax);
        refill_res
    }
    #[inline]
    pub fn refill_next(&self, index: usize) -> usize {
        if index == self.scRefillMax - 1 {
            0
        } else {
            index + 1
        }
    }
    #[inline]
    pub fn refill_add_tail(&mut self, rTime: ticks_t, rAmount: ticks_t) {
        assert!(self.refill_size() < self.scRefillMax);
        let new_tail = self.refill_next(self.scRefillTail);
        self.scRefillTail = new_tail;
        unsafe {
            (*self.refill_tail()).rAmount = rAmount;
            (*self.refill_tail()).rTime = rTime;
        }
        assert!(new_tail < self.scRefillMax);
    }
    #[inline]
    pub fn maybe_add_empty_tail(&mut self) {
        if self.is_round_robin() {
            self.refill_add_tail(unsafe { (*self.refill_head()).rTime }, 0);
        }
    }
    #[inline]
    pub fn refill_new(&mut self, max_refills: usize, budget: usize, period: ticks_t) {
        self.scPeriod = period;
        self.scRefillHead = 0;
        self.scRefillTail = 0;
        self.scRefillMax = max_refills;
        assert!(budget >= MIN_BUDGET());
        unsafe {
            (*self.refill_head()).rAmount = budget;
            (*self.refill_head()).rTime = ksCurTime;
        }
        self.maybe_add_empty_tail();
    }
    #[inline]
    pub fn refill_head_overlapping(&mut self) -> bool {
        if !self.refill_single() {
            let amount = unsafe { (*self.refill_head()).rAmount };
            let tail = unsafe { (*self.refill_head()).rTime } + amount;
            return unsafe { (*self.refill_index(self.refill_next(self.scRefillHead))).rTime }
                <= tail;
        } else {
            return false;
        }
    }
    #[inline]
    pub fn refill_unblock_check(&mut self) {
        if self.is_round_robin() {
            return;
        }
        if self.refill_ready() {
            unsafe { (*self.refill_head()).rTime = ksCurTime };
            unsafe { ksReprogram = true };
            while self.refill_head_overlapping() {
                let old_head = self.refill_pop_head();
                unsafe {
                    (*self.refill_head()).rTime = (*old_head).rTime;
                    (*self.refill_head()).rAmount += (*old_head).rAmount;
                }
            }
        }
    }
    #[inline]
    pub fn refill_ready(&mut self) -> bool {
        unsafe { (*self.refill_head()).rTime <= ksCurTime + getKernelWcetTicks() }
    }
    #[inline]
    fn refill_index(&self, index: usize) -> *mut refill_t {
        //&mut refill_t {
        convert_to_mut_type_ref::<refill_t>(
            (self.get_ptr() + size_of::<sched_context_t>()) + index * size_of::<refill_t>(),
        ) as *mut refill_t
    }
    #[inline]
    fn refill_head(&self) -> *mut refill_t {
        self.refill_index(self.scRefillHead)
    }
    #[inline]
    fn refill_tail(&self) -> *mut refill_t {
        self.refill_index(self.scRefillTail)
    }
    pub fn refill_size(&mut self) -> usize {
        if self.scRefillHead <= self.scRefillTail {
            return self.scRefillTail - self.scRefillHead + 1;
        }
        return self.scRefillTail + 1 + (self.scRefillMax - self.scRefillHead);
    }
    pub fn refill_full(&mut self) -> bool {
        self.refill_size() == self.scRefillMax
    }
    pub fn refill_single(&mut self) -> bool {
        self.scRefillHead == self.scRefillTail
    }
    #[inline]
    pub fn refill_capacity(&mut self, usage: ticks_t) -> ticks_t {
        if unlikely(usage > unsafe { (*self.refill_head()).rAmount }) {
            return 0;
        }
        return unsafe { (*self.refill_head()).rAmount } - usage;
    }
    #[inline]
    pub fn refill_sufficient(&mut self, usage: ticks_t) -> bool {
        self.refill_capacity(usage) >= MIN_BUDGET()
    }

    pub fn schedContext_resume(&mut self) {
        assert!(self.get_ptr() != 0 || self.scTcb != 0);
        if likely(self.get_ptr() != 0) && convert_to_mut_type_ref::<tcb_t>(self.scTcb).is_runnable()
        {
            if !(self.refill_ready() && self.refill_sufficient(0)) {
                assert!(
                    convert_to_mut_type_ref::<tcb_t>(self.scTcb)
                        .tcbState
                        .get_tcbQueued()
                        != 0
                );
                self.postpone();
            }
        }
    }
    pub fn schedContext_bindTCB(&mut self, tcb: &mut tcb_t) {
        assert!(self.scTcb == 0);
        assert!(tcb.tcbSchedContext == 0);
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
        assert!(self.scTcb == tcb.get_ptr());
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
        assert!(self.get_ptr() != 0);
        assert!(to.get_ptr() != 0);
        assert!(to.tcbSchedContext == 0);
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
    pub fn setConsumed(&mut self) {
        let consumed = self.schedContext_updateConsumed();
        let length = get_currenct_thread().set_mr(0, consumed);
        get_currenct_thread().tcbArch.set_register(
            MsgInfo,
            seL4_MessageInfo::new(0, 0, 0, length as u64).to_word(),
        );
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
