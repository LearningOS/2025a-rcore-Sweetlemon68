### Questions

#### Question 1

The PTEs in SV39 page tables consists of PPN, RSW bits and flag bits. PPN is the physical page number that this PTE maps to, which is partitioned to three parts to support normal and super pages. RSW bits are reserved for software.

The meanings of flag bits are described as follows:
- V describes whether this PTE is valid.
- R/W/X describes whether the virtual page in this PTE is readable/writable/executable. If all three bits are zero, then this is not a leaf node of the multi-level page table.
- U describes whether this virtual page is accessible in U-mode.
- G marks this PTE as global (for all address spaces), which is used for TLB.
- A/D describes whether this virtual page is accessed/modified after that bit is cleared.

#### Question 2

Instruction Page Fault, Load Page Fault and Store Page Fault can be caused by page faults.

When a page fault occurs, `scause` is set to the concrete page fault exception (9, 10, 11 in our framework), and `stval` is set to the virtual address that results in page fault. `sepc` is set to the virtual address of instruction that causes page fault, and `sstatus` is set like usual exceptions (like SSP bit is set to U-mode).

For lazy page allocation in code loading, we may boot the program faster if the code size is large. Also, if the program contains some dead code that is never executed, we can save memory space by not loading them.

To process 10GB of memory, note that 10GB is 2621440 pages, and this requires about 2621440 PTEs in the third level page table and 5120 PTEs in the second level page table. Each PTE takes 8 bytes, so the total memory space required is about 20MB, which wastes a lot of memory if we only access a small portion of the memory.

To implement lazy page allocation, we may record the memory regions (like using binary search trees that allows to insert and search efficiently) that are allocated but not mapped in `MemorySet`. When a page fault occurs, we check whether the faulting address is in these regions. If so, we allocate a physical frame, map it to the faulting virtual page, copy data if it is a file-backed page (like text segment pages), and return. Otherwise, we treat it as an invalid access and kill the process.

When the page is swapped to disks, the valid bit of PTE is unset, so a page fault will occur when accessing this page. OS may record the position that the page is swapped to in the PTE (like in the PPN field or RSW bits), so when a page fault occurs, OS can read the page from disks and map it back to the virtual page.

#### Question 3

- When we only have a single page table, we only need to switch page tables at context switch. To switch a page table, note that the kernel memory mapping is the same for all process page tables, so the memory mapping is continuous during the switch. We only need to set `satp` to the new page table root and flush TLB.
- To prevent U-mode code to access kernel memory, we can set the U bit to 0 in kernel memory mappings in all process page tables. Thus, when U-mode code tries to access kernel memory, a page fault will occur.
- The advantages of single page table is that we do not need to switch page tables at traps, so it is faster (like for syscalls) and easier to implement. And the kernel can access user memory directly without walking user page tables in software.
- In the double page table implementation, we have to switch page tables whenever we enter or exit kernel mode or when we have a context switch. If we write an OS with single page table implementation, we only need to switch page tables at context switch, which is less frequent.

### Coding Labs

We implement `mmap` and `munmap` system calls, and fix `sys_get_time` and `sys_trace` in address spaces. Apart from changes in the previous lab, we add methods in `TaskManager` to allow us to operate on TCBs, and add a method in `MemorySet` to unmap a memory region.

- For read and write operations in `sys_trace`, we translate the virtual address, check that it is valid, readable (writable) and accessable in user mode, then perform the operation.
- For `sys_get_time`, we utilize `sys_trace` to write the result by bytes.
- For `mmap`, we check the parameters, convert permission flags to `MapPermission`, check conflict by translating the virtual pages in range one by one, and map the memory region by `MemorySet::insert_framed_area`.
- For `munmap`, we check the parameters, translate the virtual pages in range one by one to ensure they are mapped, and unmap the memory region by `MemorySet::unmap_area`.

### Honor Code

1. 在完成本次实验的过程（含此前学习的过程）中，我未曾交流对象做过交流。

2. 此外，除课程文档外，我未参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
