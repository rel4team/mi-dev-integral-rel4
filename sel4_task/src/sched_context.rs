use sel4_common::{
    arch::{getMaxTicksToUs, ticksToUs},
    platform::time_def::{ticks_t, time_t},
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
impl sched_context {
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
