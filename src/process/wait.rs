use alloc::sync::Arc;

use crate::prelude::*;

use super::{Pid, Process};

pub fn wait_child(wait_pid: i32, process: Arc<Process>) -> Result<(Pid, u32)> {
    if wait_pid == -1 {
        // Try first wait
        let res = process.wait_remove_one_nonblock();
        match res {
            Ok((pid, exit_code)) => return Ok((pid as Pid, exit_code)),
            Err(err) if err.error() == Errno::EAGAIN => {}
            Err(err) => return Err(err),
        }

        let wait_queue = process.wait_children_queue();
        Ok(wait_queue.wait_until(|| process.wait_remove_one_nonblock().ok()))
    } else {
        if wait_pid < -1 {
            warn!("We use pgid as pid since we don't support pgid");
        }

        let pid = wait_pid.abs();

        // Try first wait
        let res = process.wait_with_pid_nonblock(pid as u32);
        match res {
            Ok(exit_code) => return Ok((pid as Pid, exit_code)),
            Err(err) if err.error() == Errno::EAGAIN => {}
            Err(err) => return Err(err),
        }

        let wait_queue = process.wait_children_queue();
        let exit_code = wait_queue.wait_until(|| process.wait_with_pid_nonblock(pid as Pid).ok());

        Ok((pid as Pid, exit_code))
    }
}
