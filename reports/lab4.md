### Questions

#### Question 1

The root inode serves as the inode of the root directory `/`, which records the size and block locations of the root directory. When we try to access one file or list files in the root directory, the system will need the root inode to find the corresponding data blocks of the root directory, which records the names and inodes and numbers under the root directory.

If the root inode is corrupted, normally the OS cannot access the root directory and thus cannot access any files in the disk. However, we can manually try to find other inodes and recover content of all files, and we can compare with the freemaps to know the data blocks corresponding to the root inode, then try to reconstruct the root directory.

#### Question 1 (Chapter 7)

In Linux terminal, we may want to redirect the output of one command to the input of another command. For example, if we want to output all processes containing "python" in their command, we may want to redirect the output of `ps aux` to the input of `grep python`. Then we will use pipe `ps aux | grep python`.

#### Question 2 (Chapter 7)

If wwe want to communicate among multiple processes but do not want to build pipes between every two processes, we can use message queues. Message queues are like shared queues. Processes can send messages to the queue and read messages from the queue. Thus, we only need to build one message queue for multiple processes to communicate.

### Coding Labs

We migrate codes of previous labs, and implement new syscalls.

For `linkat`, we find the old inode by name, and create a new directory entry pointing to the old inode.

For `unlinkat`, we find the directory entry and remove it. If the inode has reference count of 0 (by iterating all inodes to count this reference count) after unlink, we deallocate the data blocks and inode. Note that we cannot call the `clear` method of the data inode (as it will retry getting the `fs` lock, leading to deadlock).

For `fstat`, we extend the file descriptor table to also include the inode index, and we iterate through all inodes to get the reference count. Note that we must translate the `st` pointer to user buffer and copy the kernel `Stat` data byte by byte to the user buffer.

### Honor Code

1. 在完成本次实验的过程（含此前学习的过程）中，我未曾交流对象做过交流。

2. 此外，除课程文档外，我未参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
