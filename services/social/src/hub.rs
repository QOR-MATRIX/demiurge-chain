//! Who is connected, and what reaches them (docs/blueprints/social.md sections 1 and 2).
//!
//! One process fans out (release 1; several need a shared bus, designed when needed). Each socket has a bounded
//! queue: a socket that falls this far behind is dropped rather than allowed to hold the others up, and its client
//! reconnects and catches up from history.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use serde_json::json;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// How many frames wait for one socket before it is dropped.
const QUEUE: usize = 512;

struct Connection {
    id: u64,
    frames: mpsc::Sender<String>,
}

/// What a person shows others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Presence {
    /// `online`, `away` or `offline`. Invisible is shown as offline.
    pub state: String,
    /// What a product the launcher started says they are playing.
    pub playing: Option<String>,
}

#[derive(Clone)]
struct Chosen {
    state: String,
    playing: Option<String>,
}

#[derive(Default)]
pub struct Hub {
    connections: RwLock<HashMap<Uuid, Vec<Connection>>>,
    chosen: RwLock<HashMap<Uuid, Chosen>>,
    next: AtomicU64,
}

/// A server event, framed.
pub fn event(name: &str, data: impl Serialize) -> String {
    json!({ "type": "event", "event": name, "data": data }).to_string()
}

impl Hub {
    /// A socket for `sub`, and the frames it is to send.
    pub async fn connect(&self, sub: Uuid) -> (u64, mpsc::Receiver<String>) {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (frames, receiver) = mpsc::channel(QUEUE);
        self.connections
            .write()
            .await
            .entry(sub)
            .or_default()
            .push(Connection { id, frames });
        (id, receiver)
    }

    /// Forget a socket. True when it was the person's last, so they have gone.
    pub async fn disconnect(&self, sub: Uuid, id: u64) -> bool {
        let mut connections = self.connections.write().await;
        let gone = match connections.get_mut(&sub) {
            Some(list) => {
                list.retain(|c| c.id != id);
                list.is_empty()
            }
            None => true,
        };
        if gone {
            connections.remove(&sub);
            self.chosen.write().await.remove(&sub);
        }
        gone
    }

    /// Send one frame to every socket of each of `subs`.
    pub async fn send(&self, subs: &[Uuid], frame: &str) {
        let mut lagging = Vec::new();
        {
            let connections = self.connections.read().await;
            for sub in subs {
                for c in connections.get(sub).into_iter().flatten() {
                    if c.frames.try_send(frame.to_string()).is_err() {
                        lagging.push((*sub, c.id));
                    }
                }
            }
        }
        if !lagging.is_empty() {
            let mut connections = self.connections.write().await;
            for (sub, id) in lagging {
                if let Some(list) = connections.get_mut(&sub) {
                    // Dropping its sender ends the socket; the client reconnects and catches up.
                    list.retain(|c| c.id != id);
                }
            }
        }
    }

    pub async fn set(&self, sub: Uuid, state: &str, playing: Option<String>) {
        self.chosen.write().await.insert(
            sub,
            Chosen {
                state: state.to_string(),
                playing,
            },
        );
    }

    /// What `sub` shows others now.
    pub async fn presence(&self, sub: Uuid) -> Presence {
        let online = self.connections.read().await.contains_key(&sub);
        let chosen = self.chosen.read().await.get(&sub).cloned();
        match (online, chosen) {
            (false, _) => Presence {
                state: "offline".into(),
                playing: None,
            },
            (true, Some(c)) if c.state == "invisible" => Presence {
                state: "offline".into(),
                playing: None,
            },
            (true, Some(c)) => Presence {
                state: c.state,
                playing: c.playing,
            },
            (true, None) => Presence {
                state: "online".into(),
                playing: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_frame_reaches_every_socket_of_each_person_and_a_lagging_one_is_dropped() {
        let hub = Hub::default();
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let (_, mut a1) = hub.connect(a).await;
        let (_, mut a2) = hub.connect(a).await;
        let (_, mut b1) = hub.connect(b).await;
        hub.send(&[a], "one").await;
        assert_eq!(a1.recv().await.as_deref(), Some("one"));
        assert_eq!(a2.recv().await.as_deref(), Some("one"));
        assert!(b1.try_recv().is_err(), "only the people named");

        for _ in 0..QUEUE {
            hub.send(&[b], "x").await;
        }
        hub.send(&[b], "one too many").await;
        let mut got = 0;
        while b1.recv().await.is_some() {
            got += 1;
        }
        assert_eq!(got, QUEUE, "the queue drained, then the socket ended");
    }

    #[tokio::test]
    async fn presence_is_what_a_person_chose_while_they_are_here() {
        let hub = Hub::default();
        let a = Uuid::new_v4();
        assert_eq!(hub.presence(a).await.state, "offline");
        let (one, _r1) = hub.connect(a).await;
        let (two, _r2) = hub.connect(a).await;
        assert_eq!(hub.presence(a).await.state, "online");
        hub.set(a, "away", Some("Rift Reversi".into())).await;
        assert_eq!(
            hub.presence(a).await,
            Presence {
                state: "away".into(),
                playing: Some("Rift Reversi".into())
            }
        );
        hub.set(a, "invisible", Some("hidden".into())).await;
        assert_eq!(
            hub.presence(a).await,
            Presence {
                state: "offline".into(),
                playing: None
            }
        );
        assert!(!hub.disconnect(a, one).await, "one socket left");
        assert!(hub.disconnect(a, two).await, "gone");
        assert_eq!(hub.presence(a).await.state, "offline");
    }
}
