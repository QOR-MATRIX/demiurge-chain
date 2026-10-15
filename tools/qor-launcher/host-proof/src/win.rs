//! The Windows half: every unsafe call the product host makes, each wrapped so that what it returns is owned and safe.

use std::ffi::c_void;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};
use std::ptr::{null, null_mut};

use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use windows_sys::Win32::Foundation::{LocalFree, FILETIME, HANDLE, HLOCAL};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenUser, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Pipes::GetNamedPipeClientProcessId;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetProcessTimes, OpenProcess, OpenProcessToken,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

/// A handle Windows returned, owned so it is closed exactly once; null is the error it signals.
fn owned(handle: HANDLE) -> io::Result<OwnedHandle> {
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a non-null handle just returned to us, owned by nobody else.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle as RawHandle) })
}

/// A `BOOL` result as an `io::Result`.
fn check(ok: i32) -> io::Result<()> {
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// The user a process runs as, as a SID string (`S-1-5-21-…`).
fn user_sid_of(process: HANDLE) -> io::Result<String> {
    let mut token: HANDLE = null_mut();
    // SAFETY: `process` is a live process handle; `token` receives a handle we then own.
    check(unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) })?;
    let token = owned(token)?;

    let mut needed = 0u32;
    // SAFETY: asking for the size only, with no buffer; the call fails by design and sets `needed`.
    unsafe {
        GetTokenInformation(
            token.as_raw_handle() as HANDLE,
            TokenUser,
            null_mut(),
            0,
            &mut needed,
        )
    };
    if needed == 0 {
        return Err(io::Error::last_os_error());
    }
    // `u64`s, so the buffer is aligned for the pointer TOKEN_USER begins with.
    let mut buffer = vec![0u64; (needed as usize).div_ceil(8)];
    // SAFETY: the buffer holds at least `needed` bytes.
    check(unsafe {
        GetTokenInformation(
            token.as_raw_handle() as HANDLE,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    })?;
    // SAFETY: Windows wrote a TOKEN_USER at the start of the aligned buffer, which outlives this borrow.
    let user = unsafe { &*(buffer.as_ptr() as *const TOKEN_USER) };

    let mut text: *mut u16 = null_mut();
    // SAFETY: the SID points into `buffer`, still alive; `text` receives a LocalAlloc'd string we free below.
    check(unsafe { ConvertSidToStringSidW(user.User.Sid, &mut text) })?;
    // SAFETY: a NUL-terminated wide string from the call above.
    let sid = unsafe {
        let length = (0..).take_while(|&i| *text.add(i) != 0).count();
        String::from_utf16_lossy(std::slice::from_raw_parts(text, length))
    };
    // SAFETY: allocated by ConvertSidToStringSidW with LocalAlloc.
    unsafe { LocalFree(text as HLOCAL) };
    Ok(sid)
}

/// A process opened for querying only.
fn open_for_query(pid: u32) -> io::Result<OwnedHandle> {
    // SAFETY: plain call; the handle, if any, is owned by `owned`.
    owned(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) })
}

/// A named pipe only the current user can open: a protected DACL granting this user's SID and nobody else, the first
/// instance of `name` (so nobody who created it earlier is answering instead), and no clients from other machines
/// (ADR-046 decision 5's transport ACL).
pub fn create_user_pipe(name: &str) -> io::Result<NamedPipeServer> {
    // SAFETY: the current process's pseudo-handle needs no closing.
    let sid = user_sid_of(unsafe { GetCurrentProcess() })?;
    let sddl: Vec<u16> = format!("D:P(A;;GA;;;{sid})")
        .encode_utf16()
        .chain([0])
        .collect();
    let mut descriptor: *mut c_void = null_mut();
    // SAFETY: a NUL-terminated SDDL string; `descriptor` receives a LocalAlloc'd descriptor freed below.
    check(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            null_mut(),
        )
    })?;
    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    // SAFETY: `attributes` and the descriptor it points to are alive for the call, which copies what it needs.
    let pipe = unsafe {
        ServerOptions::new()
            .first_pipe_instance(true)
            .reject_remote_clients(true)
            .create_with_security_attributes_raw(
                name,
                (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
            )
    };
    // SAFETY: allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW with LocalAlloc.
    unsafe { LocalFree(descriptor as HLOCAL) };
    pipe
}

/// The process id of the client connected to `pipe`.
pub fn pipe_client_pid(pipe: &NamedPipeServer) -> io::Result<u32> {
    let mut pid = 0u32;
    // SAFETY: `pipe` is a live pipe handle for the duration of the call.
    check(unsafe { GetNamedPipeClientProcessId(pipe.as_raw_handle() as HANDLE, &mut pid) })?;
    Ok(pid)
}

/// When process `pid` was created, in 100-nanosecond intervals since 1601. Two processes with one id have different
/// creation times.
pub fn process_start_time(pid: u32) -> io::Result<u64> {
    let process = open_for_query(pid)?;
    let zero = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
    // SAFETY: a live process handle and four FILETIMEs to fill.
    check(unsafe {
        GetProcessTimes(
            process.as_raw_handle() as HANDLE,
            &mut created,
            &mut exited,
            &mut kernel,
            &mut user,
        )
    })?;
    Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

/// Whether process `pid` runs as the same user as this one.
pub fn process_is_this_users(pid: u32) -> io::Result<bool> {
    let process = open_for_query(pid)?;
    // SAFETY: the current process's pseudo-handle needs no closing.
    let ours = user_sid_of(unsafe { GetCurrentProcess() })?;
    Ok(user_sid_of(process.as_raw_handle() as HANDLE)? == ours)
}

/// A job object that ends every process in it when the last handle to it closes: when the launcher exits, however it
/// exits, the products it started end with it (ADR-046 decision 2).
pub struct KillOnCloseJob(OwnedHandle);

impl KillOnCloseJob {
    pub fn new() -> io::Result<Self> {
        // SAFETY: an anonymous job with default security; the handle is owned by `owned`.
        let job = owned(unsafe { CreateJobObjectW(null(), null()) })?;
        // SAFETY: an all-zero JOBOBJECT_EXTENDED_LIMIT_INFORMATION is valid: no limits set.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: a live job handle and a structure of the size given.
        check(unsafe {
            SetInformationJobObject(
                job.as_raw_handle() as HANDLE,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        })?;
        Ok(Self(job))
    }

    /// Put a process (a `std::process::Child`, say) in the job.
    pub fn assign(&self, process: &impl AsRawHandle) -> io::Result<()> {
        // SAFETY: both handles are alive for the call.
        check(unsafe {
            AssignProcessToJobObject(
                self.0.as_raw_handle() as HANDLE,
                process.as_raw_handle() as HANDLE,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::windows::named_pipe::ClientOptions;

    fn unique(name: &str) -> String {
        format!(
            r"\\.\pipe\qor-host-proof-test-{name}-{}-{:?}",
            std::process::id(),
            std::time::Instant::now()
        )
    }

    #[tokio::test]
    async fn a_client_on_the_pipe_is_named_by_its_process_id() {
        let name = unique("pid");
        let server = create_user_pipe(&name).unwrap();
        let _client = ClientOptions::new().open(&name).unwrap();
        server.connect().await.unwrap();
        assert_eq!(pipe_client_pid(&server).unwrap(), std::process::id());
    }

    #[tokio::test]
    async fn a_second_pipe_of_the_same_name_is_refused() {
        let name = unique("first");
        let _first = create_user_pipe(&name).unwrap();
        assert!(
            create_user_pipe(&name).is_err(),
            "someone else could answer in our place"
        );
    }

    #[test]
    fn this_process_has_a_start_time_that_does_not_change_and_is_this_users() {
        let first = process_start_time(std::process::id()).unwrap();
        assert!(first > 0);
        assert_eq!(process_start_time(std::process::id()).unwrap(), first);
        assert!(process_is_this_users(std::process::id()).unwrap());
    }

    #[test]
    fn a_process_that_has_ended_has_no_start_time() {
        let mut child = std::process::Command::new("cmd")
            .args(["/c", "exit 0"])
            .spawn()
            .unwrap();
        let pid = child.id();
        child.wait().unwrap();
        drop(child);
        // Its id may be reused at once, but not by a process created at the same instant; either way, not this one.
        if let Ok(started) = process_start_time(pid) {
            assert_ne!(started, process_start_time(std::process::id()).unwrap());
        }
    }

    #[test]
    fn closing_the_job_ends_the_process_in_it() {
        let mut child = std::process::Command::new("cmd")
            .args(["/c", "ping -n 30 127.0.0.1 >nul"])
            .spawn()
            .unwrap();
        let job = KillOnCloseJob::new().unwrap();
        job.assign(&child).unwrap();
        assert!(
            child.try_wait().unwrap().is_none(),
            "the process should still be running"
        );
        drop(job);
        let ended = (0..50).any(|_| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            child.try_wait().unwrap().is_some()
        });
        if !ended {
            let _ = child.kill();
        }
        assert!(ended, "the process outlived its job");
    }
}
