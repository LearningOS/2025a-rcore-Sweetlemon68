use super::suspend_current_and_run_next;
use crate::mm::{translated_refmut, translated_byte_buffer, UserBuffer};
use crate::task::current_task;
use alloc::sync::Arc;

/// Submission Queue Entry
#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SQEntry {
    /// Operation code
    pub opcode: u8,
    /// Additional flags
    pub flags: u8,
    /// I/O priority
    pub ioprio: u16,
    /// File descriptor
    pub fd: i32,
    /// Buffer address
    pub addr: u64,
    /// Buffer length
    pub len: u32,
    /// Read/Write flags
    pub rw_flags: i32,
    /// User data
    pub user_data: u64,
}

/// Completion Queue Entry
#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CQEntry {
    /// User data
    pub user_data: u64,
    /// Result of the operation
    pub res: i32,
    /// Additional flags
    pub flags: u32,
}

const ENTRIES: usize = 4096;

/// Ring Buffer structure containing Submission and Completion Queues
#[allow(unused)]
#[repr(C)]
pub struct RingBuffer {
    /// Submission Queue head index
    pub sq_head: u32,
    /// Submission Queue tail index
    pub sq_tail: u32,
    /// Completion Queue head index
    pub cq_head: u32,
    /// Completion Queue tail index
    pub cq_tail: u32,
    /// Submission Queue entries
    pub sq_entries: [SQEntry; ENTRIES],
    /// Completion Queue entries
    pub cq_entries: [CQEntry; ENTRIES],
}

/// The entry point for the kernel worker thread.
/// `args` is the virtual address of the RingBuffer in User Space.
pub fn io_uring_worker(args: usize) {
    let task = current_task().unwrap();
    let process_weak = task.process.clone();
    drop(task);
    
    let ring_va = args;
    
    loop {
        let mut handled = false;
        
        if let Some(process) = process_weak.upgrade() {
            if process.inner_exclusive_access().is_zombie {
                break;
            }

            // Scope to hold the token and ring reference
            {
                let inner = process.inner_exclusive_access();
                let token = inner.get_user_token();
                drop(inner); // Release lock before potentially long operations (though translated_refmut is fast)

                // Translate RingBuffer pointer
                let ring = translated_refmut::<RingBuffer>(token, ring_va as *mut RingBuffer);
                
                let head = ring.sq_head;
                let tail = ring.sq_tail;
                
                // 处理所有挂起的请求（Batch processing）
                // 我们在一次唤醒中尽可能多处理一些，减少上下文切换频率
                let mut loop_count = 0;
                let mut current_sq_head = head;
                
                while current_sq_head != tail && loop_count < ENTRIES {
                    let idx = (current_sq_head as usize) % ENTRIES;
                    let sqe = ring.sq_entries[idx];
                    
                    // Do IO
                    let start_time = crate::timer::get_time_us();
                    let ret = handle_io(token, &sqe, &process);
                    let end_time = crate::timer::get_time_us();
                    
                    // 重新获取 ring 引用 (虽然在单核且未 yield 情况下可能不需要，但为了安全)
                    // Re-acquire ring to write Completion Queue (CQ)
                    // Note: translated_refmut returns a temporary mutable reference. 
                    // We re-translate to ensure we point to the correct physical memory even if we yielded (though we didn't here).
                    // let ring = translated_refmut::<RingBuffer>(token, ring_va as *mut RingBuffer);
                    
                    let cq_tail = ring.cq_tail;
                    let cq_idx = (cq_tail as usize) % ENTRIES;
                    
                    ring.cq_entries[cq_idx] = CQEntry {
                        user_data: sqe.user_data,
                        res: ret,
                        flags: (end_time - start_time) as u32, // Store latency in flags for perf purpose
                    };
                    
                    // Commit changes
                    ring.cq_tail = cq_tail.wrapping_add(1);
                    current_sq_head = current_sq_head.wrapping_add(1);
                    loop_count += 1;
                    handled = true;
                }
                
                // 更新 SQ head
                if handled {
                     let ring = translated_refmut::<RingBuffer>(token, ring_va as *mut RingBuffer);
                     ring.sq_head = current_sq_head;
                }
            }
        } else {
            break;
        }
        
        // If no work was found, yield the CPU to let user process run.
        if !handled {
             suspend_current_and_run_next();
        }
    }
}

fn handle_io(token: usize, sqe: &SQEntry, process: &Arc<crate::task::ProcessControlBlock>) -> i32 {
    let fd = sqe.fd as usize;
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() || inner.fd_table[fd].is_none() {
        return -9; // EBADF
    }
    let file = inner.fd_table[fd].as_ref().unwrap().clone();
    drop(inner);
    
    match sqe.opcode {
        0 => { // READ
            if !file.readable() { return -22; } // EINVAL
            // Translate user buffer address to kernel slices
            let buffers = translated_byte_buffer(token, sqe.addr as *const u8, sqe.len as usize);
            let user_buffer = UserBuffer::new(buffers);
            file.read(user_buffer) as i32
        }
        1 => { // WRITE
            if !file.writable() { return -22; }
            let buffers = translated_byte_buffer(token, sqe.addr as *const u8, sqe.len as usize);
            let user_buffer = UserBuffer::new(buffers);
            file.write(user_buffer) as i32
        }
        _ => -22 // EINVAL
    }
}
