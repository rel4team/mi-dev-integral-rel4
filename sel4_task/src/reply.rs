use sel4_common::structures_gen::call_stack;

use crate::{set_thread_state, tcb_t, ThreadState};

pub type reply_t = reply;
#[repr(C)]
#[derive(Debug, Clone)]
// TODO: MCS
pub struct reply {
    /// TCB pointed to by this reply object
    pub replyTCB: usize,
    pub replyPrev: call_stack,
    pub replyNext: call_stack,
    pub padding: usize,
}
impl reply {
    pub fn get_ptr(&mut self) -> usize {
        self as *const _ as usize
    }
    pub fn unlink(&mut self, tcb: &mut tcb_t) {
        assert!(self.replyTCB == tcb.get_ptr());
        assert!(tcb.tcbState.get_replyObject() as usize == self.get_ptr());
        tcb.tcbState.set_replyObject(0);
        self.replyTCB = 0;
        set_thread_state(tcb, ThreadState::ThreadStateInactive);
    }
    pub fn push(&mut self, tcb_caller: &mut tcb_t, tcb_callee: &mut tcb_t, canDonate: bool) {
        // TODO: MCS
    }
    pub fn pop(&mut self, tcb: &mut tcb_t) {
        // TODO: MCS
    }
    pub fn remove(&mut self, tcb: &mut tcb_t) {
        // TODO: MCS
    }
}
