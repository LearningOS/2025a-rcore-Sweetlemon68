### Questions

#### Question 1

In the real case, as the stride is stored using 8-bit unsigned integers, after executing p2, the stride of p2 will overflow and become 4, so it would execute again.

When there is no overflow and all priority is larger than 2, i.e., all passes are not larger than BigStride / 2, by contradiction, suppose there exists a moment when STRIDE_MAX - STRIDE_MIN > BigStride / 2. As there is no overflow, STRIDE_MAX must be larger than zero, and at the last time when STRIDE_MAX increases, the task corresponding to STRIDE_MIN must have stride not larger than STRIDE_MIN, and the task corresponding to STRIDE_MAX must have stride equal to STRIDE_MAX - pass, which is at least STRIDE_MAX - BigStride / 2, larger than STRIDE_MIN, which contradicts with the stride algorithm (because we should exectute the task with minimum stride). Therrefore, STRIDE_MAX - STRIDE_MIN <= BigStride / 2.

If we set BigStride smaller than the maximum value M of the stride type, then we can compute the difference of two strides a and b as min((a - b + M) % M, (b - a + M) % M). Then according to which difference is used, we can determine which stride is larger. Therefore, the above proof still holds, and we can still guarantee that no overflow will occur.

The code is listed below:

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let diff1 = self.0.wrapping_sub(other.0);
        let diff2 = other.0.wrapping_sub(self.0);
        if diff1 < diff2 {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

### Coding Labs

First we migrate the previous labs to the new framework, especially using the `inner_exclusive_access` method.

To implement the `spawn` system call, we get path and ELF, and create memory set as in `exec`, then create the task control block as in fork.

To implement the stride scheduling algorithm, we define a `Stride` class as described in the question above, and add `stride` and `pass` fields to the TCB. In the task manager, we use a vector instead of deque to maintain the ready tasks, and when fetching the next task, we iterate through the vector to find the task with the minimum `pass` value, and update its `pass` value accordingly.

### Honor Code

1. 在完成本次实验的过程（含此前学习的过程）中，我未曾交流对象做过交流。

2. 此外，除课程文档外，我未参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
