//! The Mesh's paid work (ADR-096, ADR-097). The first tests need nothing running; `live` needs a Demiurge development
//! node, whose genesis makes a Mesh period ten blocks (a minute), and is ignored by default:
//!
//! ```text
//! chain/target/release/demiurge-node --dev --tmp
//! cargo test --manifest-path tools/qor-launcher/core/Cargo.toml --features desktop --lib mesh::paid::tests::live -- --ignored --nocapture
//! ```

use super::*;
use crate::mesh::Net;

struct Dir(std::path::PathBuf);

impl Dir {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("qor-mesh-paid-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn work(len: usize, seed: u8) -> Vec<u8> {
    (0..len)
        .map(|i| (i as u32).wrapping_mul(2_654_435_761).to_le_bytes()[1] ^ seed)
        .collect()
}

/// A proof read a piece at a time from the Mesh's store is the proof `mesh_proof` makes from the whole file, so the
/// chain, which checks with `mesh_proof`, accepts it. Sizes cross every boundary the tree has: one group, just over
/// one, a full tree, and more than one piece read per sibling.
#[tokio::test]
async fn a_proof_from_the_store_is_the_proof_the_chain_checks() {
    let dir = Dir::new();
    let store = FsStore::load(dir.0.join("store")).await.unwrap();
    for size in [1usize, 16_384, 16_385, 100_000, 1 << 20, (1 << 20) + 7] {
        let bytes = work(size, size as u8);
        let tag = store.blobs().add_slice(&bytes).await.unwrap();
        for index in 0..mesh_proof::groups(size as u64) {
            let (group, proof) = prove_from_store(&store, tag.hash, size as u64, index)
                .await
                .unwrap();
            assert_eq!(
                Some(proof.clone()),
                mesh_proof::prove(&bytes, index),
                "size {size}, group {index}"
            );
            assert!(
                mesh_proof::verify(tag.hash.as_bytes(), size as u64, index, &group, &proof),
                "size {size}, group {index}"
            );
        }
        assert!(
            prove_from_store(
                &store,
                tag.hash,
                size as u64,
                mesh_proof::groups(size as u64)
            )
            .await
            .is_err(),
            "a group the file does not have"
        );
    }
    store.shutdown().await.unwrap();
}

/// The Mesh grant signs with no dialog, so only the Mesh's own code may reach it: the module that encodes and checks
/// granted calls, and the one that builds receipts (ADR-097 decision 2). A new caller anywhere else in the core fails
/// here. Since the Mesh moved into the core with the vault (DIRECTION L13.2), signing under a grant and restoring one are
/// the core's alone, so nothing outside it can reach either, and the vault's own test holds the Windows launcher to that.
#[test]
fn only_the_mesh_signs_under_its_grant() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut callers = Vec::new();
    let mut restorers = Vec::new();
    let mut stack = vec![src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            // Tests sign under a grant to check what the Mesh refuses; they are not callers in the launcher.
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with("tests.rs"))
            {
                let text = std::fs::read_to_string(&path).unwrap();
                let name = path
                    .strip_prefix(&src)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                // Split, so this file is not a caller by containing the needle.
                if text.contains(concat!(".sign_", "granted(")) {
                    callers.push(name.clone());
                }
                if text.contains(concat!("MeshGrant::", "recorded")) {
                    restorers.push(name);
                }
            }
        }
    }
    callers.sort();
    restorers.sort();
    assert_eq!(callers, ["chain/mesh.rs", "mesh/paid.rs"]);
    assert_eq!(restorers, ["mesh/paid.rs"]);
}

/// Nobody is asked to allow paid work on a chain that cannot pay for it (here, no chain at all), and nothing is recorded.
#[tokio::test]
async fn no_grant_is_asked_for_where_the_chain_has_no_mesh() {
    let dir = Dir::new();
    let vault = Arc::new(crate::vault::testing::vault_in(dir.0.join("vault")));
    vault
        .create(&crate::vault::derive::generate_mnemonic().unwrap())
        .unwrap();
    let mesh = Mesh::new(&dir.0, Net::Local);
    mesh.connect(
        Arc::clone(&vault),
        Arc::new(ChainClient::new("ws://127.0.0.1:1").unwrap()),
    );
    let approving = crate::testing::Scripted::approving();
    assert!(matches!(
        mesh.allow_paid(&approving).await,
        Err(QorError::Mesh(_))
    ));
    assert_eq!(approving.times_asked(), 0, "nobody was asked");
    assert_eq!(mesh.grant(), None);
    assert_eq!(mesh.paid_status().account, None);
    // With no grant a round of paid work is nothing at all: it does not even reach for the chain.
    mesh.tick(0).await.unwrap();
}

pub(crate) mod live {
    use super::*;
    use crate::chain::mesh::EscrowTerms;
    use crate::chain::{live::endpoint, live::fund_from_alice};
    use crate::mesh::Settings;
    use crate::testing::Scripted;
    use std::time::Instant;

    const CGT: u128 = crate::cgt::EMBERS_PER_CGT;
    const MIB: u64 = 1024 * 1024;

    struct Launcher {
        dir: Dir,
        vault: Arc<Vault>,
        chain: Arc<ChainClient>,
        mesh: Arc<Mesh>,
        account: [u8; 32],
        phrase: String,
    }

    impl Launcher {
        async fn new(seeding: bool) -> Self {
            let dir = Dir::new();
            let vault = Arc::new(crate::vault::testing::vault_in(dir.0.join("vault")));
            let phrase = crate::vault::derive::generate_mnemonic().unwrap();
            let accounts = vault.create(&phrase).unwrap();
            let account = crate::vault::normalise_address(&accounts[0].address).unwrap();
            fund_from_alice(&endpoint(), account, 1_000 * CGT).await;
            let chain = Arc::new(ChainClient::new(endpoint()).unwrap());
            let mesh = Mesh::new(&dir.0, Net::Local);
            mesh.connect(Arc::clone(&vault), Arc::clone(&chain));
            mesh.configure(Settings {
                seeding,
                disk_bytes: 64 * MIB,
                upload_bytes_per_second: if seeding { 64 * MIB } else { 0 },
            })
            .await
            .unwrap();
            let approving = Scripted::approving();
            mesh.allow_paid(&approving).await.unwrap();
            assert_eq!(approving.times_asked(), 1, "the grant is one dialog");
            // The test drives each round itself, so no two rounds sign with one nonce.
            mesh.stop_worker();
            Self {
                dir,
                vault,
                chain,
                mesh,
                account,
                phrase,
            }
        }

        fn address(&self) -> String {
            crate::vault::derive::address_of(&self.account)
        }

        async fn balance(&self) -> u128 {
            self.chain.balance(&self.address()).await.unwrap()
        }
    }

    /// Run rounds of `who`'s paid work until `done` says so, or fail after `limit`.
    async fn until(
        who: &Launcher,
        limit: Duration,
        what: &str,
        done: impl Fn(&PaidStatus) -> bool,
    ) {
        let started = Instant::now();
        let mut n = 1;
        while !done(&who.mesh.paid_status()) {
            assert!(
                started.elapsed() < limit,
                "{what} did not happen within {limit:?}"
            );
            if let Err(e) = who.mesh.tick(n).await {
                eprintln!("round {n}: {e}");
            }
            n += 1;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    /// The Mesh's whole paid path against a real chain: an uploader's escrow, a seeder that takes it on and is paid for
    /// proving it holds the file, and a receiver who finds the file by its fingerprint alone, fetches it from that
    /// seeder, and signs the receipt the seeder is paid by. Every signature is the vault's, under the grant; every
    /// check is the chain's.
    #[tokio::test]
    #[ignore = "needs a running Demiurge development node; see the module docs"]
    async fn a_seeder_is_paid_for_holding_and_for_serving() {
        let captured = crate::testing::captured_log();
        let from = captured.0.lock().unwrap().len();
        // Declined, the grant records nothing, and the person was asked once.
        {
            let dir = Dir::new();
            let vault = Arc::new(crate::vault::testing::vault_in(dir.0.join("vault")));
            vault
                .create(&crate::vault::derive::generate_mnemonic().unwrap())
                .unwrap();
            let mesh = Mesh::new(&dir.0, Net::Local);
            mesh.connect(vault, Arc::new(ChainClient::new(endpoint()).unwrap()));
            let declining = Scripted::declining();
            assert!(matches!(
                mesh.allow_paid(&declining).await,
                Err(QorError::Declined)
            ));
            assert_eq!(declining.times_asked(), 1);
            assert_eq!(mesh.grant(), None);
        }

        let uploader = Launcher::new(true).await;
        let seeder = Launcher::new(true).await;
        let receiver = Launcher::new(false).await;
        for (a, b) in [
            (&uploader, &seeder),
            (&seeder, &uploader),
            (&receiver, &seeder),
            (&receiver, &uploader),
        ] {
            a.mesh.learn(b.mesh.addr().await.unwrap());
        }

        // The receipt bytes the launcher signs are the bytes the chain checks.
        let sample = Receipt {
            escrow: 9,
            period: 4,
            seeder: seeder.account,
            receiver: receiver.account,
            bytes: 12_345,
        };
        assert_eq!(
            receipt_message(&seeder.chain.mesh_genesis().await.unwrap(), &sample),
            seeder
                .chain
                .mesh_receipt_message_on_chain(&sample)
                .await
                .unwrap()
        );

        // 1. The uploader publishes, binds its node, and opens an escrow at its own prices, in a dialog.
        let bytes = work(300 * 1024, 3);
        let reference = crate::content::ContentRef::of(&bytes);
        crate::content::TemporaryStore::in_data_dir(&uploader.dir.0)
            .put(&reference, &bytes)
            .unwrap();
        assert_eq!(uploader.mesh.take_in_own().await.unwrap(), 1);
        uploader.mesh.tick(1).await.unwrap();
        let node = uploader.mesh.node_id().await.unwrap();
        assert_eq!(
            uploader.chain.mesh_node(uploader.account).await.unwrap(),
            Some(node)
        );

        // Placeholders for the test, not prices anyone is offered (AGENTS.md §5): every price is an uploader's own.
        let storage_price = CGT;
        let serving_per_mib = 4 * CGT;
        let terms = EscrowTerms {
            root: reference.root_hex(),
            size: bytes.len() as u64,
            copies: 1,
            storage_price_embers: storage_price.to_string(),
            serving_price_per_mib_embers: serving_per_mib.to_string(),
            serving_cap_per_period: bytes.len() as u64,
            deposit_embers: (200 * CGT).to_string(),
        };
        let approving = Scripted::approving();
        let escrow = uploader
            .chain
            .mesh_open_escrow(&uploader.vault, &approving, &uploader.address(), &terms)
            .await
            .unwrap();
        assert_eq!(
            approving.times_asked(),
            1,
            "an escrow moves CGT, so it has its dialog"
        );

        // 2. The seeder finds it, fetches the file from the uploader, verified, and takes the copy.
        let before = seeder.balance().await;
        seeder.mesh.tick(0).await.unwrap();
        let held = seeder.mesh.paid_status().holdings;
        assert_eq!(held.len(), 1, "the seeder took the escrow on");
        assert_eq!(
            (held[0].escrow, held[0].root.as_str()),
            (escrow, reference.root_hex().as_str())
        );
        assert!(seeder
            .chain
            .mesh_assignment(escrow, seeder.account)
            .await
            .unwrap()
            .is_some());

        // 3. The next period's challenge is answered with a proof from the store, and the chain pays the storage price.
        until(&seeder, Duration::from_secs(240), "a storage answer", |s| {
            s.holdings.first().is_some_and(|h| h.answered.is_some())
        })
        .await;
        assert_eq!(
            seeder.mesh.paid_status().paid_embers,
            storage_price.to_string()
        );
        assert_eq!(seeder.balance().await, before + storage_price);

        // 4. The receiver finds the file by its fingerprint alone: the chain names the seeder, who serves it.
        let fetched = receiver
            .mesh
            .fetch_root(&reference.root_hex())
            .await
            .unwrap();
        assert_eq!(
            (fetched.root.as_str(), fetched.size),
            (reference.root_hex().as_str(), bytes.len() as u64)
        );
        assert_eq!(
            seeder.mesh.paid_status().pending_receipts,
            1,
            "the receipt reached the seeder"
        );

        // What the seeder refuses to keep: a signature that is not the receiver's, a receipt naming another seeder, and
        // one for a file it does not hold. The chain would refuse each; the seeder does not wait to be told.
        let genesis = seeder.chain.mesh_genesis().await.unwrap();
        let period = seeder.chain.mesh_period().await.unwrap();
        let honest = Receipt {
            escrow,
            period,
            seeder: seeder.account,
            receiver: receiver.account,
            bytes: 2 * bytes.len() as u64,
        };
        let forged = uploader
            .vault
            .sign_granted(
                &uploader.mesh.grant().unwrap(),
                &receipt_message(&genesis, &honest),
            )
            .unwrap();
        assert!(
            !seeder.mesh.take_receipt(honest.clone(), forged).await,
            "signed by someone else"
        );
        let sign = |r: &Receipt| {
            receiver
                .vault
                .sign_granted(
                    &receiver.mesh.grant().unwrap(),
                    &receipt_message(&genesis, r),
                )
                .unwrap()
        };
        let elsewhere = Receipt {
            seeder: uploader.account,
            ..honest.clone()
        };
        assert!(
            !seeder
                .mesh
                .take_receipt(elsewhere.clone(), sign(&elsewhere))
                .await,
            "for another seeder"
        );
        let not_held = Receipt {
            escrow: escrow + 1,
            ..honest.clone()
        };
        assert!(
            !seeder
                .mesh
                .take_receipt(not_held.clone(), sign(&not_held))
                .await,
            "for a file not held"
        );
        assert_eq!(
            seeder.mesh.paid_status().pending_receipts,
            1,
            "none of them was kept"
        );

        // 5. The seeder claims it, and the chain pays for the bytes served at the uploader's price per MiB: 300 KiB at
        //    4 CGT a MiB is 1200/1024 of a CGT, exactly 1.171875 CGT, so nothing is rounded away.
        const SERVED: u128 = 1_171_875_000_000_000_000;
        let served = SERVED;
        let before = seeder.balance().await;
        until(&seeder, Duration::from_secs(60), "a serving claim", |s| {
            s.pending_receipts == 0
        })
        .await;
        assert_eq!(seeder.balance().await, before + served);
        assert_eq!(
            seeder.mesh.paid_status().paid_embers,
            (storage_price + served).to_string()
        );
        for l in [&uploader, &seeder, &receiver] {
            l.mesh.stop().await;
        }

        // Every level of the log was on throughout, and no account's recovery phrase reached it, as text or as bytes.
        // Receipts and signatures are not secrets: the chain publishes them when they are claimed.
        let log = String::from_utf8_lossy(&captured.0.lock().unwrap()[from..]).into_owned();
        assert!(
            log.contains("iroh") || log.contains("jsonrpsee"),
            "the run reached the log at all"
        );
        for l in [&uploader, &seeder, &receiver] {
            let bytes = format!("{:?}", &l.phrase.as_bytes()[..16]);
            let bytes = bytes.trim_start_matches('[').trim_end_matches(']');
            assert!(
                !log.contains(&l.phrase) && !log.contains(bytes),
                "a recovery phrase reached the log"
            );
        }
    }

    /// A file published from the Library (L10.2): minted as an asset whose manifest names it alone, with no commit, after
    /// a dialog that says it becomes public; once minted the Mesh holds it as one's own work, by the file's own
    /// fingerprint. Declined, nothing is written.
    #[tokio::test]
    #[ignore = "needs a running Demiurge development node; see the module docs"]
    async fn a_file_published_from_the_library_is_minted_and_seeded() {
        let me = Launcher::new(true).await;
        // An account's first mint also creates its collection, and holds just over 1,000 CGT in deposits for it.
        fund_from_alice(&endpoint(), me.account, 2_000 * CGT).await;
        let bytes = work(200 * 1024, 9);
        let source = me.dir.0.join("Holiday.jpg");
        std::fs::write(&source, &bytes).unwrap();
        let file = crate::content::ContentRef::of_file(&source).unwrap();
        let (name, manifest) =
            crate::library::publication("Holiday.jpg", 1_700_000_000, file).unwrap();
        let reference = manifest.reference();
        let store = crate::content::TemporaryStore::in_data_dir(&me.dir.0);
        let publish = |confirm: Scripted| {
            let (store, source, manifest, name) = (&store, &source, &manifest, &name);
            let (vault, chain, address) = (&me.vault, &me.chain, me.address());
            async move {
                let result = chain
                    .publish_file(vault, &confirm, &address, name, &reference, &file, || {
                        store.put_file(&file, source)?;
                        store.put(&reference, &manifest.bytes())?;
                        Ok(())
                    })
                    .await;
                (result, confirm)
            }
        };

        let (declined, asked) = publish(Scripted::declining()).await;
        assert!(matches!(declined, Err(QorError::Declined)), "{declined:?}");
        let body = asked.asked.lock()[0].body.clone();
        assert!(
            body.contains("becomes public") && body.contains("yours to share"),
            "the dialog says so: {body}"
        );
        assert!(
            !store.path_of(&file).exists(),
            "declined, nothing is written"
        );

        let (receipt, _) = publish(Scripted::approving()).await;
        let receipt = receipt.unwrap();
        assert_eq!(receipt.name, "Holiday.jpg");
        assert_eq!(receipt.file.root, file.root_hex());
        let asset = me
            .chain
            .assets_of(&me.address())
            .await
            .unwrap()
            .into_iter()
            .find(|a| (a.collection, a.item) == (receipt.collection, receipt.item))
            .expect("the asset is on the chain");
        assert_eq!(
            asset.current.root,
            reference.root_hex(),
            "the asset is its manifest"
        );
        assert_eq!(asset.commit, None, "a Library file has no commit");

        me.mesh.take_in_own().await.unwrap();
        let held = me.mesh.held().await;
        assert!(
            held.iter().any(|h| h.root == file.root_hex()
                && h.kind == "own"
                && h.size == bytes.len() as u64),
            "the Mesh holds the file as one's own work: {held:?}"
        );
        me.mesh.stop().await;
    }
}
