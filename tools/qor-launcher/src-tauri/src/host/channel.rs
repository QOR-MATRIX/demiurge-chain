//! The channel one spawn opens, on whichever platform this is (ADR-046 decisions 3 and 5): a named pipe on Windows
//! that only this user can open, a Unix socket in a folder only this user can enter elsewhere. It takes exactly one
//! connection, and only from the process the launcher spawned, as the operating system reports it.

use std::io;

use super::session::TOKEN_LIFETIME;
use super::{random_hex, HostError};

/// Who must be on the other end: the spawned process, and when it started (where the platform says), so a process
/// that later reuses its id does not pass.
#[derive(Debug, Clone, Copy)]
pub struct Expected {
    pub pid: u32,
    pub started: Option<u64>,
}

impl Expected {
    fn check(&self, pid: Option<u32>) -> Result<(), HostError> {
        let Some(pid) = pid else {
            return Err(HostError::NotSpawned(
                "the operating system did not say who connected".into(),
            ));
        };
        if pid != self.pid {
            return Err(HostError::NotSpawned(format!(
                "process {pid} connected, where the launcher started process {}",
                self.pid
            )));
        }
        if let Some(started) = self.started {
            if qor_host_proof::process_start_time(pid).ok() != Some(started) {
                return Err(HostError::NotSpawned(format!(
                    "process {pid} is not the process the launcher started under that id"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
pub type Stream = tokio::net::windows::named_pipe::NamedPipeServer;
#[cfg(unix)]
pub type Stream = tokio::net::UnixStream;

pub struct Channel {
    name: String,
    #[cfg(windows)]
    pipe: Stream,
    #[cfg(unix)]
    listener: tokio::net::UnixListener,
    #[cfg(unix)]
    dir: std::path::PathBuf,
}

impl Channel {
    /// A new channel under a name nobody can guess, created before the product starts so it is there to connect to.
    pub fn open() -> io::Result<Self> {
        #[cfg(windows)]
        {
            let name = format!(r"\\.\pipe\qor-host-{}", random_hex(16));
            let pipe = qor_host_proof::create_user_pipe(&name)?;
            Ok(Self { name, pipe })
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
            let dir = std::env::temp_dir().join(format!("qor-host-{}", random_hex(8)));
            std::fs::DirBuilder::new().mode(0o700).create(&dir)?;
            let path = dir.join("channel");
            let listener = tokio::net::UnixListener::bind(&path)?;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
            Ok(Self {
                name: path.to_string_lossy().into_owned(),
                listener,
                dir,
            })
        }
    }

    /// What the product is told to connect to. A name, not a secret: the token is the secret.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The one connection, within the token's lifetime, from the expected process; nothing else is accepted
    /// afterwards, since the channel is consumed.
    pub async fn accept(self, expected: Expected) -> Result<Stream, HostError> {
        #[cfg(windows)]
        {
            tokio::time::timeout(TOKEN_LIFETIME, self.pipe.connect())
                .await
                .map_err(|_| HostError::Expired)?
                .map_err(HostError::Io)?;
            expected.check(qor_host_proof::pipe_client_pid(&self.pipe).ok())?;
            let ours =
                qor_host_proof::process_is_this_users(expected.pid).map_err(HostError::Io)?;
            if !ours {
                return Err(HostError::NotSpawned(
                    "the process runs as another user".into(),
                ));
            }
            Ok(self.pipe)
        }
        #[cfg(unix)]
        {
            let accepted = tokio::time::timeout(TOKEN_LIFETIME, self.listener.accept()).await;
            // The socket is not wanted again whatever happened: one spawn, one connection.
            let _ = std::fs::remove_dir_all(&self.dir);
            let (stream, _) = accepted
                .map_err(|_| HostError::Expired)?
                .map_err(HostError::Io)?;
            let pid = stream
                .peer_cred()
                .ok()
                .and_then(|cred| cred.pid())
                .and_then(|pid| u32::try_from(pid).ok());
            expected.check(pid)?;
            Ok(stream)
        }
    }
}

#[cfg(unix)]
impl Drop for Channel {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
