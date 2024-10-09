use crate::arch::resetTimer;
use crate::config::{irqInvalid, maxIRQ};
use crate::interrupt::*;
use core::intrinsics::unlikely;
use log::debug;
use sel4_common::structures::exception_t;
use sel4_common::structures_gen::{cap_Splayed, cap_tag};
use sel4_ipc::notification_t;
use sel4_task::{activateThread, schedule, timerTick};

#[no_mangle]
pub fn handleInterruptEntry() -> exception_t {
    let irq = getActiveIRQ();

    if irq != irqInvalid {
        handleInterrupt(irq);
    }

    schedule();
    activateThread();
    exception_t::EXCEPTION_NONE
}

#[no_mangle]
pub fn handleInterrupt(irq: usize) {
    if unlikely(irq > maxIRQ) {
        debug!(
            "Received IRQ {}, which is above the platforms maxIRQ of {}\n",
            irq, maxIRQ
        );
        mask_interrupt(true, irq);
        ackInterrupt(irq);
        return;
    }
    match get_irq_state(irq) {
        IRQState::IRQInactive => {
            debug!("IRQInactive");
            mask_interrupt(true, irq);
            debug!("Received disabled IRQ: {}\n", irq);
        }
        IRQState::IRQSignal => {
            debug!("IRQSignal");
            let handler_slot = get_irq_handler_slot(irq);
            let handler_cap = &handler_slot.capability;
			match handler_cap.splay() {
				cap_Splayed::notification_cap(data)=>{
					if data.get_capNtfnCanSend() !=0{
						let nf = convert_to_mut_type_ref::<notification_t>(data.get_capNtfnPtr() as usize);
						nf.send_signal(data.get_capNtfnBadge() as usize);
					}
				}
				_=>{}
			}
        }
        IRQState::IRQTimer => {
            timerTick();
            resetTimer();
        }
        #[cfg(feature = "ENABLE_SMP")]
        IRQState::IRQIPI => {
            unsafe { crate::ffi::handleIPI(irq, true) };
        }
        IRQState::IRQReserved => {
            debug!("Received unhandled reserved IRQ: {}\n", irq);
        }
    }
    ackInterrupt(irq);
}
