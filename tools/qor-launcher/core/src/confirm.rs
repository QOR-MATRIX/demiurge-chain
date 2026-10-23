//! Host-side confirmation (roadmap L1.4): what the person is asked to approve, and who asks them.
//!
//! Moved here from the Windows launcher's `lib.rs` with the vault, which asks through it (DIRECTION L13.2). Each shell
//! draws the question itself: the Windows launcher's `HostDialog` is a native dialog the webview can neither draw,
//! dismiss nor answer.

use std::future::Future;
use std::pin::Pin;

/// What the person at the keyboard is asked to approve.
///
/// The host builds it from the values it is about to act on, never from text the
/// webview supplies, so the prompt describes what will actually happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub title: String,
    pub body: String,
    /// Label of the approving button. The other button is always "Cancel".
    pub approve: String,
}

/// The answer to a [`Prompt`]: `true` only if the person approved.
pub type Answer<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// Asks the person at the keyboard to approve an action (roadmap L1.4).
///
/// Every signature and every endpoint change goes through one of these. The
/// Windows launcher's implementation is `HostDialog`, a native dialog the host draws
/// and the webview can neither draw, dismiss nor answer. A compromised webview
/// can therefore ask for a signature, but cannot obtain one unseen.
///
/// The one approval that is not a dialog is the Windows launcher's `IdentityConsent`
/// under an open `ArrivalGrant`: straight after the vault opens, it approves the QOR ID
/// sign-in for the signed-in profile's wallet account (ADR-016, ADR-056, ADR-099).
pub trait Confirm: Send + Sync {
    fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a>;
}

/// Test support: a [`Confirm`] with a fixed answer that records every prompt. The Windows launcher's tests use it too,
/// through the core's `testing` feature.
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use super::{Answer, Confirm, Prompt};

    pub struct Scripted {
        answer: bool,
        pub asked: parking_lot::Mutex<Vec<Prompt>>,
    }

    impl Scripted {
        pub fn approving() -> Self {
            Self {
                answer: true,
                asked: parking_lot::Mutex::new(Vec::new()),
            }
        }

        pub fn declining() -> Self {
            Self {
                answer: false,
                asked: parking_lot::Mutex::new(Vec::new()),
            }
        }

        pub fn times_asked(&self) -> usize {
            self.asked.lock().len()
        }
    }

    impl Confirm for Scripted {
        fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a> {
            // Every prompt a test reaches reads as written: a lost line continuation in a string literal leaves a run of
            // spaces mid-sentence, which the Mesh grant's dialog showed from 0.1.10 to 0.1.14.
            for text in [&prompt.title, &prompt.body, &prompt.approve] {
                assert!(
                    !text.contains("  "),
                    "a prompt has a run of spaces: {text:?}"
                );
            }
            self.asked.lock().push(prompt.clone());
            let answer = self.answer;
            Box::pin(async move { answer })
        }
    }
}
