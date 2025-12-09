### Questions

#### Question 1

The bad test cases are `ch2b_bad_instructions.rs` and `ch2b_bad_register.rs`. The bad test cases are run when we invoke `make run` command.

The output of both test cases is as follows:

```plain
[kernel] IllegalInstruction in application, kernel killed it.
```

When we access register `sstatus` of S-privilege mode or invoke S-privileged instruction `sret` in U-privilege mode, an `IllegalInstruction` exception will be raised. The kernel will catch this exception and kill the application.

Our RustSBI version is RustSBI-QEMU Version 0.2.0-alpha.2.

#### Question 2

1. When we just entering `__restore`, `sp` points to the top of the kernel stack, which points to the TrapContext of current user task.
Two common use cases of `__restore` are entering U-mode for the first time, and returning from a trap to U-mode.
2. In L43-L48, `sstatus`, `sepc` and `sscratch` are handled specially by reading them from `TrapContext` to `t0`, `t1` and `t2`. Register `sstatus` stores the previous privilege mode (U) and interrupt enable bit, which are essential for returning to U-mode correctly. Register `sepc` stores the return address of the user program (which instruction to run after returning to U-mode). Register `sscratch` is used to store the `sp` of the user stack, which is then swapped with the kernel stack `sp` in L60.
3. `x2` is `sp`, which is still used to restore later registers, so we cannot restore it here; it must be handled specially in L60. `x4` is `tp`, which is not used in U-mode and not saved in `__alltraps`, so we skip restoring it here.
4. After L60, `sp` now points to the user stack, and `sscratch` now points to the kernel stack (to restore in the next trap).
5. After `sret`, the CPU will switch to U-mode. This is because when executing `sret`, the CPU reads the previous privilege mode from `sstatus` (which is U-mode) and switches to that mode.
6. After L13, `sp` points to the top of the kernel stack (which is saved in `sscratch` when entering U-mode at the last time). And `sscratch` now points to the top of user stack.
7. To enter S-mode from U-mode, the user program can use `ecall` instruction. Also, any instructions that cause exceptions (like page fault, illegal instruction, etc.) will also transfer to S-mode.

### Coding Labs

Add a syscall count array to TCB, and increment the count in `os::syscall::syscall` function whenever a syscall is invoked. Implement the `trace` syscall to read/write memory of current process, or return the count of a specific syscall invoked by the current process, by reading the array in TCB.

### Honor Code

1. 在完成本次实验的过程（含此前学习的过程）中，我未曾交流对象做过交流。

2. 此外，除课程文档外，我未参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
