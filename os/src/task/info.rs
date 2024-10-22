//! Implementation of [`TaskContext`]

// use core::time;

use crate::{config::MAX_SYSCALL_NUM, timer::get_time_ms};
use super::TaskStatus;
// use crate::timer::TICKS_PER_SEC;

/// The task info of a task.
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// The task status in it's lifecycle
    pub status: TaskStatus,
    /// The times the syscall are called. Index represent the id of syscall
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// time gap between first scheduled and last syscall
    pub time: usize,
    /// first time scheduled
    pub first_time: Option<usize>,
}

impl TaskInfo {
    /// Create a new empty task context
    pub fn init() -> Self {
        Self {
            status: TaskStatus::Running,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0,
            first_time: None
        }
    }

    /// update the task info when syscall happened
    pub fn syscalled(&mut self, syscall_id: usize) {
        // println!("task: {}: syscall: {}", pid.0, syscall_id);
        self.syscall_times[syscall_id] += 1 ;
        // println!("task: {}: syscall[169]: {}", pid.0, self.syscall_times[169]);
        let now = get_time_ms();
        if let Some(first_time) = self.first_time {
            self.time = now - first_time;
        } else {
            self.first_time = Some(now);
            self.time = 0;
        }

        // println!("called time: {}" , self.time);
    }
}

/// just the wrap for the task_info to pass the test
pub struct TaskInfoToReturn {
    /// task status
    pub status: TaskStatus,
    /// syscall times
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// last call time
    pub time: usize,
}