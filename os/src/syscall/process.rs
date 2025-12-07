//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            // id should be regarded as *const u8, and return one byte data
            let addr = id as *const u8;
            unsafe {
                let data = *addr;
                data as isize
            }
        }
        1 => {
            // id should be regarded as *mut u8, and data is one byte data
            let addr = id as *mut u8;
            let data = data as u8;
            unsafe {
                *addr = data;
            }
            0
        }
        2 => {
            // id should be regarded as syscall id, and return count of this syscall
            let syscall_idx = super::get_syscall_count_index(id).unwrap();
            crate::task::get_current_thread_syscall_count(syscall_idx) as isize
        }
        _ => {
            panic!("Unsupported trace_request: {}", trace_request);
        }
    }
}
