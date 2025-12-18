//! Process management syscalls
use core::mem::size_of;

use alloc::sync::Arc;

use crate::{
    loader::get_app_data_by_name,
    mm::{VirtAddr, translated_byte_buffer, translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    }, timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time",
        current_task().unwrap().pid.0
    );
    // get addr of sec and usec in TimeVal
    unsafe {
        let sec_addr = &mut (*ts).sec as *mut usize as *mut u8;
        let usec_addr = &mut (*ts).usec as *mut usize as *mut u8;
        let us = get_time_us();
        let sec = us / 1_000_000;
        let usec = us % 1_000_000;
        // write sec and usec to user space
        let sec_buf = &mut translated_byte_buffer(current_user_token(), sec_addr, size_of::<usize>())[0];
        let usec_buf = &mut translated_byte_buffer(current_user_token(), usec_addr, size_of::<usize>())[0];
        sec_buf.copy_from_slice(&sec.to_ne_bytes());
        usec_buf.copy_from_slice(&usec.to_ne_bytes());
    }
    0
}

/// mmap.
pub fn sys_mmap(start: usize, len: usize, prop: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap",
        current_task().unwrap().pid.0
    );
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
    let end_va = VirtAddr(start + len);
    let cur_task = crate::task::current_task().unwrap();
    let mut tcb = cur_task.inner_exclusive_access();
    for vpn in start_va.floor().0..end_va.ceil().0 {
        if let Some(pte) = tcb.memory_set.translate(vpn.into()) {
            if pte.is_valid() {
                return -1;
            }
        }
    }
    // Allocate and map pages
    tcb.memory_set.insert_framed_area(start_va, end_va, permission);
    0
}

/// munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap",
        current_task().unwrap().pid.0
    );
    let start_va = VirtAddr(start);
    // If address start is not aligned to page size, return -1
    if !start_va.aligned() {
        return -1;
    }
    let end_va = VirtAddr(start + len);
    let cur_task = crate::task::current_task().unwrap();
    let mut tcb = cur_task.inner_exclusive_access();
    for vpn in start_va.floor().0..end_va.ceil().0 {
        if let Some(pte) = tcb.memory_set.translate(vpn.into()) {
            if !pte.is_valid() {
                return -1;
            }
        } else {
            return -1;
        }
    }
    tcb.memory_set.remove_area_with_start_vpn(start_va.floor());
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// spawn.
pub fn sys_spawn(path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        let new_task = task.spawn(data);
        let new_pid = new_task.pid.0;
        // add new task to scheduler
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    }
}

// Set task priority.
pub fn sys_set_priority(prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority",
        current_task().unwrap().pid.0
    );
    if prio <= 1 {
        return -1;
    }
    let cur_task = current_task().unwrap();
    let mut inner = cur_task.inner_exclusive_access();
    inner.pass = crate::config::BIG_STRIDE / (prio as usize);
    prio
}
