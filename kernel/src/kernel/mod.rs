pub mod boot;
pub mod fastpath;
pub mod fault;
core::arch::global_asm!(include_str!("fastpath_restore.S"));