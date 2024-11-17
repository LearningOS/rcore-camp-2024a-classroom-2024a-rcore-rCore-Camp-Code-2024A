## 实现功能

实现了spawn和stride调度两个功能，添加了`sys_spawn`,`sys_setprio` 两个系统调用。

## 问答作业

1. 并不是。因为 `p2.stride`在add一个pass后，会产生溢出，最后的值为`5`，小于`p1.stride`，因此程序会调度`p2`

2.  当`priority`大于2时`PASS_MAX` = `BIG_STRIDE/2`，显然有， `STRIDE_MAX – STRIDE_MIN <= BigStride / 2`

3. ```
   impl PartialOrd for Stride {
       fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
           if self.0 < other.0 {
           	if other.0 - self.0 > BigStride / 2 {
           		Some(Ordering::Greater)
           	} else {
           		Some(Ordering::Less)
           	}
           } else if self.0 > other.0 {
           	if self.0 - other.0 > BigStride / 2 {
           		Some(Ordering::Less)
           	} else {
           		Some(Ordering::Greater)
           	}
           } else {
           	Some(Ordering::Equal)
           }
       }
   }
   ```

   

