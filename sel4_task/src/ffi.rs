use sel4_common::structures_gen::{endpoint, notification};

use crate::{prio_t, tcb_queue_t, tcb_t};

//TODO: MCS
#[no_mangle]
pub extern "C" fn sendIPC() {
    unimplemented!("MCS");
}
#[no_mangle]
pub extern "C" fn installTCBCap() {
    unimplemented!("MCS");
}
#[no_mangle]
pub extern "C" fn tcbSchedDequeue(tcb: &mut tcb_t) {
    (*tcb).sched_dequeue();
}
#[no_mangle]
pub extern "C" fn possibleSwitchTo() {
    unimplemented!("MCS");
}
#[no_mangle]
pub extern "C" fn tcbSchedAppend(tcb: &mut tcb_t) {
    (*tcb).sched_append();
}
#[no_mangle]
pub extern "C" fn invokeTCB_ThreadControlCaps() {
    unimplemented!("MCS");
}
#[no_mangle]
pub extern "C" fn setMCPriority(tcb: &mut tcb_t, prio: prio_t) {
    tcb.set_mc_priority(prio);
}
#[no_mangle]
pub extern "C" fn setPriority(tcb: &mut tcb_t, prio: prio_t) {
    tcb.set_priority(prio);
}

extern "C" {
    // reorder ep and reorder ntfn is a circular reference problem
    #[cfg(feature = "KERNEL_MCS")]
    pub fn reorder_EP(ep: &mut endpoint, thread: &mut tcb_t);
    #[cfg(feature = "KERNEL_MCS")]
    pub fn reorder_NTFN(ntfn: &mut notification, thread: &mut tcb_t);
}
