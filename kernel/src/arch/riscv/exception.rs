use super::{read_stval, read_time};
use crate::compatibility::lookupIPCBuffer;
use crate::config::*;
use crate::halt;
use crate::kernel::boot::current_fault;
use crate::object::lookupCapAndSlot;
use crate::strnlen;
use crate::syscall::handle_fault;
use crate::syscall::{
    SysDebugCapIdentify, SysDebugDumpScheduler, SysDebugHalt, SysDebugNameThread, SysDebugPutChar,
    SysDebugSnapshot, SysGetClock,
};
use log::debug;
use sel4_common::arch::ArchReg::*;
use sel4_common::fault::seL4_Fault_t;
use sel4_common::sel4_config::seL4_MsgMaxLength;
use sel4_common::structures::exception_t;
use sel4_cspace::arch::CapTag;
use sel4_common::print;
use sel4_task::{activateThread, get_currenct_thread, schedule};

#[no_mangle]
pub fn handleUnknownSyscall(w: isize) -> exception_t {
	let thread = get_currenct_thread();
    if w == SysDebugPutChar {
		print!("{}",thread.tcbArch.get_register(Cap) as u8 as char);
        return exception_t::EXCEPTION_NONE;
    }
    if w == SysDebugDumpScheduler {
		// unimplement debug
        // println!("debug dump scheduler");
        return exception_t::EXCEPTION_NONE;
    }
    if w == SysDebugHalt {
		// unimplement debug
        // println!("debug halt");
        return exception_t::EXCEPTION_NONE;
    }
    if w == SysDebugSnapshot {
		// unimplement debug
        // println!("debug snap shot");
        return exception_t::EXCEPTION_NONE;
    }
    if w == SysDebugCapIdentify {
        let cptr = thread.tcbArch.get_register(Cap);
        let lu_ret = lookupCapAndSlot(thread, cptr);
        let cap_type = lu_ret.cap.get_cap_type();
        thread.tcbArch.set_register(Cap, cap_type as usize);
        return exception_t::EXCEPTION_NONE;
    }
    if w == SysDebugNameThread {
        let cptr = thread.tcbArch.get_register(Cap);
        let lu_ret = lookupCapAndSlot(thread, cptr);
        let cap_type = lu_ret.cap.get_cap_type();

        if cap_type != CapTag::CapThreadCap {
            debug!("SysDebugNameThread: cap is not a TCB, halting");
            halt();
        }
        let name = lookupIPCBuffer(true, thread) + 1;
        if name == 0 {
            debug!("SysDebugNameThread: Failed to lookup IPC buffer, halting");
            halt();
        }

        let len = strnlen(name as *const u8, seL4_MsgMaxLength * 8);
        if len == seL4_MsgMaxLength * 8 {
            debug!("SysDebugNameThread: Name too long, halting");
            halt();
        }
		
        // setThreadName(TCB_PTR(cap_thread_cap_get_capTCBPtr(lu_ret.cap)), name);
        return exception_t::EXCEPTION_NONE;
    }
    if w == SysGetClock {
        let current = read_time();
        thread.tcbArch.set_register(Cap, current);
        return exception_t::EXCEPTION_NONE;
    }
    unsafe {
        current_fault = seL4_Fault_t::new_unknown_syscall_fault(w as usize);
        handle_fault(get_currenct_thread());
    }
    schedule();
    activateThread();
    exception_t::EXCEPTION_NONE
}

#[no_mangle]
pub fn handleUserLevelFault(w_a: usize, w_b: usize) -> exception_t {
    unsafe {
        current_fault = seL4_Fault_t::new_user_exeception(w_a, w_b);
        handle_fault(get_currenct_thread());
    }
    schedule();
    activateThread();
    exception_t::EXCEPTION_NONE
}

#[no_mangle]
pub fn handleVMFaultEvent(vm_faultType: usize) -> exception_t {
    let status = handle_vm_fault(vm_faultType);
    if status != exception_t::EXCEPTION_NONE {
        handle_fault(get_currenct_thread());
    }
    schedule();
    activateThread();
    exception_t::EXCEPTION_NONE
}

pub fn handle_vm_fault(type_: usize) -> exception_t {
    let addr = read_stval();
    match type_ {
        RISCVLoadPageFault | RISCVLoadAccessFault => {
            unsafe {
                current_fault = seL4_Fault_t::new_vm_fault(addr, RISCVLoadAccessFault, 0);
            }
            exception_t::EXCEPTION_FAULT
        }
        RISCVStorePageFault | RISCVStoreAccessFault => {
            unsafe {
                current_fault = seL4_Fault_t::new_vm_fault(addr, RISCVStoreAccessFault, 0);
            }
            exception_t::EXCEPTION_FAULT
        }
        RISCVInstructionAccessFault | RISCVInstructionPageFault => {
            unsafe {
                current_fault = seL4_Fault_t::new_vm_fault(addr, RISCVInstructionAccessFault, 1);
            }
            exception_t::EXCEPTION_FAULT
        }
        _ => panic!("Invalid VM fault type:{}", type_),
    }
}
