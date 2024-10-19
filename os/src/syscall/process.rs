//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{copy_to_user, MapPermission},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_curr_task_status, mmap, munmap, suspend_current_and_run_next, TaskStatus
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub(crate) status: TaskStatus,
    /// The numbers of syscall called by task
    pub(crate) syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub(crate) time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    // let ts = translated_ptr(current_user_token(), _ts as usize) as *mut TimeVal;
    // unsafe {
    //     *ts = TimeVal {
    //         sec: us / 1_000_000,
    //         usec: us % 1_000_000,
    //     };
    // }
    let ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    copy_to_user(current_user_token(), _ts as *mut u8, &ts as *const TimeVal as *const u8, core::mem::size_of::<TimeVal>());
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let ti = get_curr_task_status();
    copy_to_user(current_user_token(), _ti as *mut u8, &ti as *const TaskInfo as *const u8, core::mem::size_of::<TaskInfo>());
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if _start & 0xfff != 0 {
        return -1;
    }
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    }
    let mut map_perm = MapPermission::U;
    if _port & 0x1 != 0 {
        map_perm |= MapPermission::R;
    }
    if _port & 0x2 != 0 {
        map_perm |= MapPermission::W;
    }
    if _port & 0x4 != 0 {
        map_perm |= MapPermission::X;
    }
    mmap(_start, _start + _len, map_perm)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start & 0xfff != 0 {
        return -1;
    }
    munmap(_start, _start + _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}