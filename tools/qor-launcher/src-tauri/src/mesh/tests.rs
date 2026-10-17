//! The Mesh, L10.1 (ADR-095): two launchers on this PC, each with its own data folder and a node on the loopback address
//! with no relay. Every file here is made by the test.

use std::time::{Duration, Instant};

use super::*;
use crate::content::{ContentRef, TemporaryStore};

struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qor-mesh-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const MIB: u64 = 1024 * 1024;

fn on(disk: u64, upload: u64) -> Settings {
    Settings {
        seeding: true,
        disk_bytes: disk,
        upload_bytes_per_second: upload,
    }
}

/// Published work in a launcher's temporary content store, as minting leaves it.
fn publish(data: &Path, bytes: &[u8]) -> ContentRef {
    let reference = ContentRef::of(bytes);
    TemporaryStore::in_data_dir(data)
        .put(&reference, bytes)
        .unwrap();
    reference
}

fn work(len: usize, seed: u8) -> Vec<u8> {
    (0..len)
        .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
        .collect()
}

#[tokio::test]
async fn seeding_is_off_until_the_person_turns_it_on_with_both_limits() {
    let a = Dir::new();
    let mesh = Mesh::new(&a.0, Net::Local);
    let status = mesh.status().await;
    assert!(!status.settings.seeding);
    assert_eq!(status.node, None, "nothing runs while seeding is off");
    for half in [on(0, MIB), on(MIB, 0)] {
        assert!(matches!(mesh.configure(half).await, Err(QorError::Mesh(_))));
    }
    assert!(!mesh.settings().seeding, "a refused choice is not saved");
    mesh.resume().await.unwrap();
    assert_eq!(
        mesh.status().await.node,
        None,
        "resume starts nothing when seeding is off"
    );
}

#[tokio::test]
async fn own_published_work_is_seeded_and_another_launcher_fetches_it_verified() {
    let (a, b) = (Dir::new(), Dir::new());
    let bytes = work(300 * 1024, 7);
    let reference = publish(&a.0, &bytes);

    let seeder = Mesh::new(&a.0, Net::Local);
    let status = seeder.configure(on(64 * MIB, 64 * MIB)).await.unwrap();
    assert!(status.node.is_some());
    assert_eq!((status.held, status.used_bytes), (1, bytes.len() as u64));
    let ticket = seeder.ticket(&reference.root_hex()).await.unwrap();

    let fetcher = Mesh::new(&b.0, Net::Local);
    fetcher
        .configure(Settings {
            seeding: false,
            disk_bytes: 64 * MIB,
            upload_bytes_per_second: 0,
        })
        .await
        .unwrap();
    let fetched = fetcher.fetch(&ticket).await.unwrap();
    assert_eq!(fetched.root, reference.root_hex());
    assert_eq!(
        seeder.held().await,
        vec![Held {
            root: reference.root_hex(),
            size: bytes.len() as u64,
            kind: "own"
        }]
    );
    assert_eq!(
        fetcher.held().await,
        vec![Held {
            root: reference.root_hex(),
            size: bytes.len() as u64,
            kind: "fetched"
        }]
    );
    assert_eq!(fetched.size, bytes.len() as u64);
    {
        let node = fetcher.node.lock().await;
        let got = node
            .as_ref()
            .unwrap()
            .store
            .blobs()
            .get_bytes(Hash::from_bytes(reference.root))
            .await
            .unwrap();
        assert_eq!(
            &got[..],
            &bytes[..],
            "the bytes that arrived are the published ones"
        );
    }
    assert!(
        seeder.status().await.served_bytes >= bytes.len() as u64,
        "what was served is counted"
    );
    // Fetching what is already held asks no one.
    seeder.stop().await;
    assert_eq!(
        fetcher.fetch(&ticket).await.unwrap().size,
        bytes.len() as u64
    );
    fetcher.stop().await;
}

#[tokio::test]
async fn a_seeder_whose_bytes_changed_is_refused_and_nothing_unverified_is_kept() {
    let (a, b) = (Dir::new(), Dir::new());
    let bytes = work(200 * 1024, 3);
    let reference = publish(&a.0, &bytes);
    let seeder = Mesh::new(&a.0, Net::Local);
    seeder.configure(on(64 * MIB, 64 * MIB)).await.unwrap();
    let ticket = seeder.ticket(&reference.root_hex()).await.unwrap();

    // The file the seeder serves from, changed under it after it was taken in: same length, other bytes.
    let mut tampered = bytes.clone();
    tampered[100 * 1024] ^= 0xff;
    std::fs::write(
        TemporaryStore::in_data_dir(&a.0).path_of(&reference),
        &tampered,
    )
    .unwrap();

    let fetcher = Mesh::new(&b.0, Net::Local);
    fetcher
        .configure(Settings {
            seeding: false,
            disk_bytes: 64 * MIB,
            upload_bytes_per_second: 0,
        })
        .await
        .unwrap();
    assert!(matches!(
        fetcher.fetch(&ticket).await,
        Err(QorError::Mesh(_))
    ));
    let node = fetcher.node.lock().await;
    let status = node
        .as_ref()
        .unwrap()
        .store
        .blobs()
        .status(Hash::from_bytes(reference.root))
        .await
        .unwrap();
    assert!(
        !matches!(status, BlobStatus::Complete { .. }),
        "a file that did not verify is not kept whole: {status:?}"
    );
}

#[tokio::test]
async fn what_would_pass_the_disk_limit_is_refused_before_it_is_fetched() {
    let (a, b) = (Dir::new(), Dir::new());
    let bytes = work(2 * MIB as usize, 9);
    let reference = publish(&a.0, &bytes);
    let seeder = Mesh::new(&a.0, Net::Local);
    seeder.configure(on(64 * MIB, 64 * MIB)).await.unwrap();
    let ticket = seeder.ticket(&reference.root_hex()).await.unwrap();

    let fetcher = Mesh::new(&b.0, Net::Local);
    fetcher
        .configure(Settings {
            seeding: false,
            disk_bytes: MIB,
            upload_bytes_per_second: 0,
        })
        .await
        .unwrap();
    assert!(matches!(
        fetcher.fetch(&ticket).await,
        Err(QorError::Mesh(_))
    ));
    assert_eq!(fetcher.status().await.used_bytes, 0, "nothing was written");

    // And a launcher that never chose a disk limit fetches nothing.
    let c = Dir::new();
    let unset = Mesh::new(&c.0, Net::Local);
    assert!(matches!(unset.fetch(&ticket).await, Err(QorError::Mesh(_))));
    assert_eq!(
        unset.status().await.node,
        None,
        "and reaches out to no peer to find that out"
    );
}

#[tokio::test]
async fn own_work_beyond_the_disk_limit_is_not_taken_in() {
    let a = Dir::new();
    let small = publish(&a.0, &work(100 * 1024, 1));
    publish(&a.0, &work(2 * MIB as usize, 2));
    let seeder = Mesh::new(&a.0, Net::Local);
    let status = seeder.configure(on(MIB, MIB)).await.unwrap();
    assert_eq!(status.held, 1, "only what fits");
    assert!(status.used_bytes <= MIB);
    assert!(seeder.ticket(&small.root_hex()).await.is_ok());
}

#[tokio::test]
async fn turning_seeding_off_stops_serving() {
    let (a, b) = (Dir::new(), Dir::new());
    let reference = publish(&a.0, &work(64 * 1024, 5));
    let seeder = Mesh::new(&a.0, Net::Local);
    seeder.configure(on(64 * MIB, 64 * MIB)).await.unwrap();
    let ticket = seeder.ticket(&reference.root_hex()).await.unwrap();
    let off = seeder
        .configure(Settings {
            seeding: false,
            ..seeder.settings()
        })
        .await
        .unwrap();
    assert_eq!(off.node, None);
    assert!(matches!(
        seeder.ticket(&reference.root_hex()).await,
        Err(QorError::Mesh(_))
    ));

    let fetcher = Mesh::new(&b.0, Net::Local);
    fetcher
        .configure(Settings {
            seeding: false,
            disk_bytes: 64 * MIB,
            upload_bytes_per_second: 0,
        })
        .await
        .unwrap();
    let fetched = tokio::time::timeout(Duration::from_secs(20), fetcher.fetch(&ticket)).await;
    assert!(
        matches!(fetched, Ok(Err(QorError::Mesh(_))) | Err(_)),
        "a launcher that stopped seeding serves nothing"
    );
}

#[tokio::test]
async fn serving_keeps_to_the_upload_limit() {
    let (a, b) = (Dir::new(), Dir::new());
    // 192 KiB at 64 KiB a second, with at most one second's worth at once: at least two seconds.
    let bytes = work(192 * 1024, 11);
    let reference = publish(&a.0, &bytes);
    let seeder = Mesh::new(&a.0, Net::Local);
    seeder.configure(on(64 * MIB, 64 * 1024)).await.unwrap();
    let ticket = seeder.ticket(&reference.root_hex()).await.unwrap();
    let fetcher = Mesh::new(&b.0, Net::Local);
    fetcher
        .configure(Settings {
            seeding: false,
            disk_bytes: 64 * MIB,
            upload_bytes_per_second: 0,
        })
        .await
        .unwrap();
    let started = Instant::now();
    fetcher.fetch(&ticket).await.unwrap();
    let took = started.elapsed();
    assert!(
        took >= Duration::from_millis(1800),
        "served too fast for the limit: {took:?}"
    );
}

#[test]
fn the_bucket_lets_a_seconds_worth_through_then_paces_the_rest() {
    let mut bucket = Bucket::new();
    bucket.tokens = 64.0 * 1024.0;
    assert_eq!(bucket.take(64 * 1024, 64 * 1024), Duration::ZERO);
    let wait = bucket.take(32 * 1024, 64 * 1024);
    assert!(
        wait >= Duration::from_millis(450) && wait <= Duration::from_millis(550),
        "{wait:?}"
    );
}

#[tokio::test]
async fn the_choice_and_the_node_survive_a_restart() {
    let a = Dir::new();
    let first = Mesh::new(&a.0, Net::Local);
    let node = first.configure(on(8 * MIB, MIB)).await.unwrap().node;
    first.stop().await;
    drop(first);
    let again = Mesh::new(&a.0, Net::Local);
    assert_eq!(again.settings(), on(8 * MIB, MIB));
    again.resume().await.unwrap();
    assert_eq!(
        again.status().await.node,
        node,
        "the same node, so peers find it again"
    );
    again.stop().await;
}

#[tokio::test]
async fn a_file_filed_under_a_root_it_does_not_hash_to_is_not_seeded() {
    let a = Dir::new();
    let real = publish(&a.0, &work(32 * 1024, 4));
    // A stray file named as another root.
    let stray = ContentRef::of(b"something else");
    let store = TemporaryStore::in_data_dir(&a.0);
    std::fs::write(store.path_of(&stray), work(32 * 1024, 6)).unwrap();
    let seeder = Mesh::new(&a.0, Net::Local);
    let status = seeder.configure(on(64 * MIB, MIB)).await.unwrap();
    assert_eq!(status.held, 1);
    assert!(seeder.ticket(&real.root_hex()).await.is_ok());
    assert!(seeder.ticket(&stray.root_hex()).await.is_err());
}
