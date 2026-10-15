//! The operating-system checks the launcher's product host needs (ADR-046, ADR-088).
//!
//! A product (QQ Studio, first) is a process the launcher spawns. The launcher talks to it over a local channel and
//! trusts only the process it spawned, proved two ways at once: a one-time token (the host's business) and the
//! operating system's word about who is on the other end (this crate's). On Windows that is:
//!
//! - [`create_user_pipe`]: a named pipe whose DACL grants only the current user, the first instance of its name, and
//!   no remote clients;
//! - [`pipe_client_pid`]: the process id of whoever connected;
//! - [`process_start_time`] and [`process_is_this_users`]: when a process started, so a recycled process id cannot pass
//!   for the product, and whether it runs as this user;
//! - [`KillOnCloseJob`]: a job object that ends the product when the launcher ends (ADR-046 decision 2).
//!
//! On Linux [`process_start_time`] reads `/proc`; elsewhere it is unsupported and the host relies on the process id and
//! the socket's permissions alone. Nothing outside the Windows module is unsafe.

#![cfg_attr(not(windows), forbid(unsafe_code))]

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{
    create_user_pipe, pipe_client_pid, process_is_this_users, process_start_time, KillOnCloseJob,
};

/// When process `pid` started, in clock ticks since the machine booted (Linux, `/proc/<pid>/stat` field 22). Two
/// processes with one id have different start times.
#[cfg(target_os = "linux")]
pub fn process_start_time(pid: u32) -> std::io::Result<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    // The command name, field 2, is in parentheses and may hold spaces or parentheses itself: count from after the last
    // closing one, where field 3 begins.
    let rest = stat
        .rfind(')')
        .map(|at| &stat[at + 1..])
        .ok_or_else(|| std::io::Error::other("an unreadable /proc stat line"))?;
    rest.split_whitespace()
        .nth(22 - 3)
        .and_then(|field| field.parse().ok())
        .ok_or_else(|| std::io::Error::other("no start time in the /proc stat line"))
}

/// Unsupported here: the host then relies on the process id and the socket's permissions.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn process_start_time(_pid: u32) -> std::io::Result<u64> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "no process start time on this platform",
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn this_process_has_a_start_time_that_does_not_change() {
        let first = super::process_start_time(std::process::id()).unwrap();
        assert!(first > 0);
        assert_eq!(
            super::process_start_time(std::process::id()).unwrap(),
            first
        );
    }

    #[test]
    fn a_process_that_is_not_there_has_none() {
        assert!(super::process_start_time(u32::MAX - 1).is_err());
    }
}
