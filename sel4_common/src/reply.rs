#[repr(C)]
#[derive(Debug, Clone)]
// TODO: MCS
pub struct reply {
    /// TCB pointed to by this reply object
    replyTCB: usize,
}
