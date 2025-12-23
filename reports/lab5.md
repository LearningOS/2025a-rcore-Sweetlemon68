### Questions

#### Question 1

When the main thread exits, we need to move all child processes to init process, recycle the resources of all tasks (tid, trap context, user stack, kernel stack), and then recycle the memory set, the fd table and the task vector.

The `TaskControlBlock` of other threads can be referenced in the `TimerCondVar`, the wait queues of mutexes, semaphores, and conditional variables, the PCB task vector of this process, and the ready queue of task manager. To remove these references, we clear the reference in `TimerCondVar` and ready queue of task manager manually, and the references in PCB and wait queues will be removed after we clear the PCB task vector and remove all references of the PCB.

#### Question 2

In the first implementation of mutex, when a thread releases the lock, it will wake up the first thread in the wait queue, while releases the lock to let all threads in the ready queue to compete for the lock. And in the second implementation, the lock is transferred to the first thread in the wait queue directly.

The first implementation may cause starvation, because when a thread releases the lock, it wakes up the first thread in the wait queue, but does not guarantee that this thread will acquire the lock successfully. Compared with this, the second implementation guarantees FIFO order, thus is fairer.

On the other hand, if we have starvation in task scheduling in the second implementation, it may cause "deadlock" (deadlock-like behavior). If the lock is transferred to a thread which is not scheduled to run for a long time, other threads waiting for the lock will never acquire it, thus causing "deadlock".

### Coding Labs

We implement the deadlock detection algorithm as specified in the document. To enable calculation of `available` vector, `need` matrix and `allocation` matrix, we add a `is_locked` method to the `Mutex` trait, and add two record vectors -- waiting records and holding records -- to every mutex and semaphore. When a process tries to acquire a mutex or semaphore, it is first added to the waiting records of the corresponding resource, and when it successfully acquires the resource, it is removed from the waiting records and added to the holding records; and when it releases the resource, it is removed from the holding records.

When a thread is to acquire a mutex or semaphore, we calculate all required vector and matrices according to the records, and run the deadlock detection algorithm. If a deadlock is detected, the `-0xDEAD` error code is returned.

### Honor Code

1. 在完成本次实验的过程（含此前学习的过程）中，我未曾交流对象做过交流。

2. 此外，除课程文档外，我未参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
