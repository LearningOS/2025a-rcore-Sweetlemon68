//! Implementation of [`TaskContext`]
use crate::trap::trap_return;
use core::arch::global_asm;

#[repr(C)]
/// task context structure containing some registers
pub struct TaskContext {
    /// Ret position after task switching
    ra: usize,
    /// Stack pointer
    sp: usize,
    /// s0-11 register, callee saved
    s: [usize; 12],
}

impl TaskContext {
    /// Create a new empty task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    /// Create a new task context with a trap return addr and a kernel stack pointer
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
    /// Create a new task context for a kernel thread
    pub fn goto_kernel_task(entry: usize, kstack_ptr: usize, arg: usize) -> Self {
        let mut s = [0; 12];
        s[0] = entry; // s0 = entry point
        s[1] = arg;   // s1 = argument
        Self {
            ra: kernel_thread_entry as usize,
            sp: kstack_ptr,
            s,
        }
    }
}

extern "C" {
    fn kernel_thread_entry();
}

global_asm!(
    ".section .text",
    ".globl kernel_thread_entry",
    "kernel_thread_entry:",
    "mv a0, s1", // Move arg (s1) to a0
    "jr s0",     // Jump to entry (s0)
);
