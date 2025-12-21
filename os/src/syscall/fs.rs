//! File and filesystem-related syscalls
use crate::fs::{OpenFlags, Stat, get_inode_id, link_file, open_file, stat_file, unlink_file};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some((file, _inode_id)) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some((file, _inode_id)) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        let inode_id = match get_inode_id(path.as_str()) {
            Some(id) => id,
            None => return -1,
        };
        inner.fd_table[fd] = Some((inode, inode_id));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!("kernel:pid[{}] sys_fstat", current_task().unwrap().pid.0);
    let token = current_user_token();
    // translate st pointer
    let stat_buf = UserBuffer::new(translated_byte_buffer(
        token,
        st as *const u8,
        core::mem::size_of::<Stat>(),
    ));
    let kernel_stat = match stat_file(fd) {
        Some(stat) => stat,
        None => return -1,
    };
    // copy kernel_stat to user space byte by byte
    let stat_bytes: &[u8; core::mem::size_of::<Stat>()] =
        unsafe { core::mem::transmute(&kernel_stat) };
    let mut remaining_bytes = stat_bytes.as_slice();
    for buf in stat_buf.buffers.into_iter() {
        let copy_size = core::cmp::min(buf.len(), remaining_bytes.len());
        buf[..copy_size].copy_from_slice(&remaining_bytes[..copy_size]);
        remaining_bytes = &remaining_bytes[copy_size..];
        if remaining_bytes.is_empty() {
            break;
        }
    }
    0
}

/// linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_linkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let old_name = translated_str(token, old_name);
    let new_name = translated_str(token, new_name);
    if old_name == new_name {
        return -1;
    }
    if link_file(old_name.as_str(), new_name.as_str()) {
        0
    } else {
        -1
    }
}

/// unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_unlinkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let name = translated_str(token, name);
    if unlink_file(name.as_str()) {
        0
    } else {
        -1
    }
}
