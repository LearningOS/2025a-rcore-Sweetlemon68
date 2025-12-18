### Questions

#### Question 1

The root inode serves as the inode of the root directory `/`, which records the size and block locations of the root directory. When we try to access one file or list files in the root directory, the system will need the root inode to find the corresponding data blocks of the root directory, which records the names and inodes and numbers under the root directory.

If the root inode is corrupted, normally the OS cannot access the root directory and thus cannot access any files in the disk. However, we can manually try to find other inodes and recover content of all files, and we can compare with the freemaps to know the data blocks corresponding to the root inode, then try to reconstruct the root directory.

### Coding Labs

We implement 

### Honor Code

1. 在完成本次实验的过程（含此前学习的过程）中，我未曾交流对象做过交流。

2. 此外，除课程文档外，我未参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
