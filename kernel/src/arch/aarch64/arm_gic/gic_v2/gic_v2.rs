use super::consts::*;
use super::{Gic_Cpu_Iface_Map, Gic_Dist_Map};
use aarch64_cpu::registers::Readable;
use tock_registers::interfaces::Writeable;

static GIC_DIST: Gic_Dist_Map = Gic_Dist_Map::new(GIC_V2_DISTRIBUTOR_PPTR as *mut u8);
static GIC_CPUIFACE: Gic_Cpu_Iface_Map = Gic_Cpu_Iface_Map::new(GIC_V2_CONTROLLER_PPTR as *mut u8);

// This is for aarch64 only
pub fn cpu_iface_init() {
    GIC_DIST.regs().enable_clr[0].set(IRQ_SET_ALL);
    GIC_DIST.regs().pending_clr[0].set(IRQ_SET_ALL);
    GIC_DIST.regs().security[0].set(0);
    GIC_DIST.regs().priority[0].set(0x0);

    let mut i = 0;
    while i < 16 {
        GIC_DIST.regs().sgi_pending_clr[i >> 2].set(IRQ_SET_ALL);
        i += 4;
    }

    GIC_CPUIFACE.regs().icontrol.set(0);
    GIC_CPUIFACE.regs().pri_msk_c.set(0x000000f0);
    GIC_CPUIFACE.regs().pb_c.set(0x00000003);
    let mut i = GIC_CPUIFACE.regs().int_ack.get();
    while (i & IRQ_MASK) != IRQ_NONE {
        GIC_CPUIFACE.regs().eoi.set(i);
        i = GIC_CPUIFACE.regs().int_ack.get();
    }
    GIC_CPUIFACE.regs().icontrol.set(1);
}

pub fn cpu_initLocalIRQController() {
    cpu_iface_init();
}

/// Enable the IRQ controller
pub fn irq_enable(irq: usize) {
    let word = irq >> 5;
    let bits = (irq & 0x1f) as u32;
    GIC_DIST.regs().enable_set[word].set(1 << bits);
}

/// Get the current interrupt number
pub fn gic_int_ack() -> usize {
    GIC_CPUIFACE.regs().int_ack.get() as usize
}

/// Acknowledge the interrupt
pub fn ack_irq(irq: usize) {
    GIC_CPUIFACE.regs().eoi.set(irq as _);
}

pub fn dist_init() {
    let nirqs = 32 * ((GIC_DIST.regs().ic_type.get() & 0x1f) + 1) as usize;

    GIC_DIST.regs().enable.set(0);

    for i in (0..nirqs).step_by(32) {
        GIC_DIST.regs().enable_clr[i >> 5].set(IRQ_SET_ALL);
        GIC_DIST.regs().pending_clr[i >> 5].set(IRQ_SET_ALL);
    }

    for i in (32..nirqs).step_by(4) {
        GIC_DIST.regs().priority[i >> 2].set(0);
    }

    let target = infer_cpu_gic_id(nirqs);

    for i in (0..nirqs).step_by(4) {
        GIC_DIST.regs().targets[i >> 2].set(TARGET_CPU_ALLINT(target));
    }

    for i in (64..nirqs).step_by(32) {
        GIC_DIST.regs().config[i >> 5].set(0x55555555);
    }

    for i in (0..nirqs).step_by(32) {
        GIC_DIST.regs().security[i >> 5].set(0);
    }

    GIC_DIST.regs().enable.set(1);
}

// BOOT_CODE static void dist_init(void)
// {
//     word_t i;
//     int nirqs = 32 * ((gic_dist->ic_type & 0x1f) + 1);
//     gic_dist->enable = 0;

//     for (i = 0; i < nirqs; i += 32) {
//         /* disable */
//         gic_dist->enable_clr[i >> 5] = IRQ_SET_ALL;
//         /* clear pending */
//         gic_dist->pending_clr[i >> 5] = IRQ_SET_ALL;
//     }

//     /* reset interrupts priority */
//     for (i = 32; i < nirqs; i += 4) {
//         if (config_set(CONFIG_ARM_HYPERVISOR_SUPPORT)) {
//             gic_dist->priority[i >> 2] = 0x80808080;
//         } else {
//             gic_dist->priority[i >> 2] = 0;
//         }
//     }

//     /*
//      * reset int target to current cpu
//      * We query which id that the GIC uses for us and use that.
//      */
//     uint8_t target = infer_cpu_gic_id(nirqs);
//     for (i = 0; i < nirqs; i += 4) {
//         gic_dist->targets[i >> 2] = TARGET_CPU_ALLINT(target);
//     }

//     /* level-triggered, 1-N */
//     for (i = 64; i < nirqs; i += 32) {
//         gic_dist->config[i >> 5] = 0x55555555;
//     }

//     /* group 0 for secure; group 1 for non-secure */
//     for (i = 0; i < nirqs; i += 32) {
//         if (config_set(CONFIG_ARM_HYPERVISOR_SUPPORT) && !config_set(CONFIG_PLAT_QEMU_ARM_VIRT)) {
//             gic_dist->security[i >> 5] = 0xffffffff;
//         } else {
//             gic_dist->security[i >> 5] = 0;
//         }
//     }
//     /* enable the int controller */
//     gic_dist->enable = 1;
// }

// BOOT_CODE static uint8_t infer_cpu_gic_id(int nirqs)
// {
//     word_t i;
//     uint32_t target = 0;
//     for (i = 0; i < nirqs; i += 4) {
//         target = gic_dist->targets[i >> 2];
//         target |= target >> 16;
//         target |= target >> 8;
//         if (target) {
//             break;
//         }
//     }
//     if (!target) {
//         printf("Warning: Could not infer GIC interrupt target ID, assuming 0.\n");
//         target = BIT(0);
//     }
//     // return target & 0xff;
// }

fn infer_cpu_gic_id(nirqs: usize) -> u8 {
    let mut target = 0;
    for i in (0..nirqs).step_by(4) {
        target = GIC_DIST.regs().targets[i >> 2].get();
        target |= target >> 16;
        target |= target >> 8;
        if target != 0 {
            break;
        }
    }
    if target == 0 {
        log::warn!("Warning: Could not infer GIC interrupt target ID, assuming 0.");
        target = 1;
    }
    (target & 0xff) as u8
}

fn TARGET_CPU_ALLINT(CPU: u8) -> u32 {
    ((CPU as u32) << 0) | ((CPU as u32) << 8) | ((CPU as u32) << 16) | ((CPU as u32) << 24)
}
