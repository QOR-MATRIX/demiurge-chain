//! What tests share, here and in the Windows launcher's tests: the scripted confirmation, and the one logger a test
//! process has. Behind the `testing` feature, which only test builds turn on.

// The whole test process's log (moved from the Windows launcher with Social and the Mesh, whose checks share it).
#[derive(Clone, Default)]
pub struct Captured(pub std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
    type Writer = Captured;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// The whole test process's log, at every level, from the first time it is asked for. One logger for the process,
/// not one for a test: `tracing` caches across threads whether each log line is wanted, and a logger set for one test's
/// thread loses lines that other tests, running without one, first decided nobody wanted. With this one installed
/// everything is wanted, always.
pub fn captured_log() -> &'static Captured {
    static CAPTURED: std::sync::OnceLock<Captured> = std::sync::OnceLock::new();
    CAPTURED.get_or_init(|| {
        let captured = Captured::default();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .with_writer(captured.clone())
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("the only process-wide logger in these tests");
        captured
    })
}

// The scripted confirmation, moved with the vault that its tests confirm through (ADR-104).
pub use crate::confirm::testing::Scripted;
