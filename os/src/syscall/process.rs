//! Process management syscalls
use crate::mm::VirtAddr;
use crate::task::{
    change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
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
    // get addr of sec and usec in TimeVal
    unsafe {
        let sec_addr = &mut (*ts).sec as *mut usize as *mut u8;
        let usec_addr = &mut (*ts).usec as *mut usize as *mut u8;
        let us = get_time_us();
        let sec = us / 1_000_000;
        let usec = us % 1_000_000;
        let sec_bytes = sec.to_ne_bytes();
        let usec_bytes = usec.to_ne_bytes();
        // write sec and usec to user space
        for i in 0..core::mem::size_of::<usize>() {
            sys_trace(1, sec_addr.add(i) as usize, sec_bytes[i] as usize);
            sys_trace(1, usec_addr.add(i) as usize, usec_bytes[i] as usize);
        }
    }
    0
}

/// trace syscall for debugging and statistics
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    match trace_request {
        0 => {
            // id should be regarded as *const u8, and return one byte data
            let va = VirtAddr(id);
            crate::task::TASK_MANAGER.compute_current_tcb(|tcb| {
                if let Some(pte) = tcb.memory_set.translate(va.floor()) {
                    if pte.is_valid() && pte.user_accessible() && pte.readable() {
                        pte.ppn().get_bytes_array()[va.page_offset()] as isize
                    } else {
                        -1
                    }
                } else {
                    -1
                }
            })
        }
        1 => {
            // id should be regarded as *mut u8, and data is one byte data
            let va = VirtAddr(id);
            crate::task::TASK_MANAGER.compute_current_tcb_mut(|tcb| {
                if let Some(pte) = tcb.memory_set.translate(va.floor()) {
                    if pte.is_valid() && pte.user_accessible() && pte.writable() {
                        let bytes = pte.ppn().get_bytes_array();
                        bytes[va.page_offset()] = data as u8;
                        0
                    } else {
                        -1
                    }
                } else {
                    -1
                }
            })
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

// mmap.
pub fn sys_mmap(start: usize, unaligned_len: usize, prop: usize) -> isize {
    trace!("kernel: sys_mmap");
    let start_va = VirtAddr(start);
    // If address start is not aligned to page size, return -1
    if !start_va.aligned() {
        return -1;
    }
    // Check prop: 1-R, 2-W, 4-X; other bits must be set to 0
    if (prop & !0x7) != 0 || prop & 0x7 == 0 {
        return -1;
    }
    // Convert prop to MapPermission
    use crate::mm::MapPermission;
    let mut permission = MapPermission::U;
    if (prop & 0x1) != 0 {
        permission |= MapPermission::R;
    }
    if (prop & 0x2) != 0 {
        permission |= MapPermission::W;
    }
    if (prop & 0x4) != 0 {
        permission |= MapPermission::X;
    }
    let end_va = VirtAddr(start + unaligned_len);
    let has_conflict = crate::task::TASK_MANAGER.compute_current_tcb(|tcb| {
        for vpn in start_va.floor().0..end_va.ceil().0 {
            if let Some(pte) = tcb.memory_set.translate(vpn.into()) {
                if pte.is_valid() {
                    return true;
                }
            }
        }
        false
    });
    if has_conflict {
        return -1;
    }
    // Allocate and map pages
    crate::task::TASK_MANAGER.compute_current_tcb_mut(|tcb| {
        tcb.memory_set
            .insert_framed_area(start_va, end_va, permission);
    });
    0
}

// munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start_va = VirtAddr(start);
    // If address start is not aligned to page size, return -1
    if !start_va.aligned() {
        return -1;
    }
    let end_va = VirtAddr(start + len);
    let has_unmapped = crate::task::TASK_MANAGER.compute_current_tcb(|tcb| {
        for vpn in start_va.floor().0..end_va.ceil().0 {
            if let Some(pte) = tcb.memory_set.translate(vpn.into()) {
                if !pte.is_valid() {
                    return true;
                }
            } else {
                return true;
            }
        }
        false
    });
    if has_unmapped {
        return -1;
    }
    if crate::task::TASK_MANAGER.compute_current_tcb_mut(|tcb| tcb.memory_set.unmap_area(start_va))
    {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
