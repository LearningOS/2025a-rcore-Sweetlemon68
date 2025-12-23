use crate::task::{current_task, add_task, TaskControlBlock, io_uring_worker};
use alloc::sync::Arc;

pub fn sys_io_uring_setup(buffer: usize, _len: usize) -> isize {
    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    
    // Inherit ustack base from current task for ID allocation purposes.
    // The kernel thread won't actually use this user stack.
    let ustack_base = task.inner_exclusive_access().res.as_ref().unwrap().ustack_base;
    
    // Create the kernel worker thread
    let new_task = Arc::new(TaskControlBlock::new_kernel(
        Arc::clone(&process),
        io_uring_worker as usize, // Entry point
        buffer,                   // Argument (Ring Buffer Address)
        ustack_base,
    ));
    
    // Register the new thread into the process structure
    let mut process_inner = process.inner_exclusive_access();
    let new_tid = new_task.inner_exclusive_access().res.as_ref().unwrap().tid;
    while process_inner.tasks.len() < new_tid + 1 {
        process_inner.tasks.push(None);
    }
    process_inner.tasks[new_tid] = Some(Arc::clone(&new_task));
    drop(process_inner);
    
    // Add to scheduler
    add_task(new_task);
    
    0 // Success
}
