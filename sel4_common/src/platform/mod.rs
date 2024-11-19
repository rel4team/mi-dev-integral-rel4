pub mod timer;
// 这里除了arch的区别，还有不同arch下的platform的区别，这是两回事，我们这边目前只涉及到关于platform的timer的部分

#[cfg(target_arch="aarch64")]
pub mod qemu_arm_virt;
#[cfg(target_arch="aarch64")]
pub use qemu_arm_virt::*;

pub mod ffi;
#[cfg(target_arch="aarch64")]
pub use ffi::*;

#[cfg(target_arch="riscv64")]
pub mod spike;
// pub mod qemu_riscv_virt;

#[cfg(target_arch="riscv64")]
pub use spike::*;