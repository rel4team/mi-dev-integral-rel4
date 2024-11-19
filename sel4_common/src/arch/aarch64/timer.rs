#[cfg(feature = "KERNEL_MCS")]
use crate::{
    platform::time_def::{
        ticks_t, time_t, KHZ_IN_MHZ, TIMER_CLOCK_HZ, TIMER_CLOCK_KHZ, TIMER_CLOCK_MHZ, USE_KHZ,
    },
    sel4_config::UINT64_MAX,
};
#[cfg(feature = "KERNEL_MCS")]
pub fn getMaxTicksToUs() -> ticks_t {
    if USE_KHZ {
        return UINT64_MAX / TIMER_CLOCK_KHZ;
    } else {
        return UINT64_MAX;
    }
}
#[cfg(feature = "KERNEL_MCS")]
pub fn ticksToUs(ticks: ticks_t) -> time_t {
    if USE_KHZ {
        return (ticks * KHZ_IN_MHZ) / TIMER_CLOCK_KHZ;
    } else {
        return ticks / TIMER_CLOCK_MHZ;
    }
}
