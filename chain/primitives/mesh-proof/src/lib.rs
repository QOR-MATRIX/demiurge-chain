//! The Mesh's storage proof (ADR-095 decision 5, ADR-096; `docs/architecture/MESH_PROOFS.md` §2).
//!
//! A file is named by its BLAKE3-256 hash (ADR-047). BLAKE3 is a binary tree over 1 KiB chunks, so one **group** of the
//! file — 16 KiB, the granularity ADR-047 and iroh-blobs share — can be shown to belong to the root by the chaining
//! values of the subtrees beside its path: one 32-byte value per level, about 1 KiB of proof for a 1 GiB file.
//!
//! - [`prove`], on a seeder's PC, from the whole file: the siblings, leaf to root.
//! - [`siblings`] and [`Subtree`], the same proof made a piece at a time, for a file too large to hold in memory: read
//!   each sibling's range and hash it.
//! - [`verify`], on chain, from the root, the size, the group's index, the group's bytes and the siblings.
//!
//! The tree is split exactly as BLAKE3 splits it ([`blake3::hazmat::left_subtree_len`]): a node's left subtree is the
//! largest power-of-two run of chunks that leaves something on the right. Above 16 KiB that is always a whole number of
//! groups, so every group is one node of the tree. Nothing here can panic on what a caller sends: lengths and offsets are
//! checked before BLAKE3's own assertions could see them.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use blake3::hazmat::{
    left_subtree_len, merge_subtrees_non_root, merge_subtrees_root, ChainingValue, HasherExt, Mode,
};

/// One group: 16 KiB.
pub const GROUP: u64 = 16 * 1024;

/// The deepest path a file can have: 2^64 bytes over 16 KiB groups is 50 levels; 64 is room to spare.
pub const MAX_DEPTH: usize = 64;

/// How many groups a file of `size` bytes has. An empty file has one, empty.
pub fn groups(size: u64) -> u64 {
    if size == 0 {
        1
    } else {
        size.div_ceil(GROUP)
    }
}

/// Where group `index` starts and how long it is, or `None` if the file has no such group.
pub fn group_range(size: u64, index: u64) -> Option<(u64, u64)> {
    if index >= groups(size) {
        return None;
    }
    let start = index * GROUP;
    Some((start, (size - start).min(GROUP)))
}

/// One level of the path from the root down to a group: the sibling subtree's start and length, and whether the group's
/// side is the left one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Level {
    sibling_start: u64,
    sibling_len: u64,
    group_on_left: bool,
}

/// The path from the root down to group `index`, top first. Empty when the file is one group.
fn path(size: u64, index: u64) -> Vec<Level> {
    let target = index * GROUP;
    let (mut start, mut len) = (0u64, size);
    let mut levels = Vec::new();
    while len > GROUP {
        let left = left_subtree_len(len);
        if target < start + left {
            levels.push(Level {
                sibling_start: start + left,
                sibling_len: len - left,
                group_on_left: true,
            });
            len = left;
        } else {
            levels.push(Level {
                sibling_start: start,
                sibling_len: left,
                group_on_left: false,
            });
            start += left;
            len -= left;
        }
    }
    levels
}

/// The chaining value of the subtree of `bytes` that starts at `offset` in the file.
fn subtree(bytes: &[u8], offset: u64) -> ChainingValue {
    let mut hasher = Subtree::at(offset);
    hasher.update(bytes);
    hasher.finish()
}

/// One sibling's chaining value, hashed a piece at a time: start it at the sibling's offset, feed it the sibling's bytes
/// in order, and finish it.
pub struct Subtree(blake3::Hasher);

impl Subtree {
    pub fn at(offset: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        if offset > 0 {
            hasher.set_input_offset(offset);
        }
        Self(hasher)
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    pub fn finish(self) -> [u8; 32] {
        self.0.finalize_non_root()
    }
}

/// The ranges of the file, as (start, length), whose chaining values prove group `index` of a file of `size` bytes, leaf
/// to root: the order [`verify`] takes them in. `None` if the file has no such group; empty if it is one group.
pub fn siblings(size: u64, index: u64) -> Option<Vec<(u64, u64)>> {
    group_range(size, index)?;
    Some(
        path(size, index)
            .iter()
            .rev()
            .map(|l| (l.sibling_start, l.sibling_len))
            .collect(),
    )
}

/// The siblings that prove group `index` of `file` belongs to `blake3::hash(file)`, leaf to root. `None` if the file has
/// no such group. Hashes every sibling subtree, so it reads the whole file once.
pub fn prove(file: &[u8], index: u64) -> Option<Vec<[u8; 32]>> {
    Some(
        siblings(file.len() as u64, index)?
            .into_iter()
            .map(|(start, len)| subtree(&file[start as usize..(start + len) as usize], start))
            .collect(),
    )
}

/// Whether `group` is group `index` of the file of `size` bytes whose BLAKE3 hash is `root`, by the `proof` [`prove`]
/// made. The proof binds the group to the root; `size` only places the group in the tree, so it must be a value the
/// verifier already holds (on chain, the escrow's), never one the prover chooses.
pub fn verify(root: &[u8; 32], size: u64, index: u64, group: &[u8], proof: &[[u8; 32]]) -> bool {
    let Some((start, len)) = group_range(size, index) else {
        return false;
    };
    if group.len() as u64 != len || proof.len() > MAX_DEPTH {
        return false;
    }
    let levels = path(size, index);
    if proof.len() != levels.len() {
        return false;
    }
    if levels.is_empty() {
        // The whole file is this one group.
        return blake3::hash(group).as_bytes() == root;
    }
    // Every group of a file larger than one group is a non-empty node of the tree.
    let mut cv = subtree(group, start);
    let top = levels.len() - 1;
    for (depth, (level, sibling)) in levels.iter().rev().zip(proof).enumerate() {
        let (left, right) = if level.group_on_left {
            (&cv, sibling)
        } else {
            (sibling, &cv)
        };
        if depth == top {
            return merge_subtrees_root(left, right, Mode::Hash).as_bytes() == root;
        }
        cv = merge_subtrees_non_root(left, right, Mode::Hash);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn file(len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| (i as u32).wrapping_mul(2_654_435_761).to_le_bytes()[1])
            .collect()
    }

    const SIZES: [usize; 10] = [
        0,
        1,
        1024,
        16_384,
        16_385,
        32_768,
        49_153,
        100_000,
        1 << 20,
        (1 << 20) + 7,
    ];

    #[test]
    fn every_group_of_every_size_proves_to_its_root() {
        for size in SIZES {
            let f = file(size);
            let root = *blake3::hash(&f).as_bytes();
            for index in 0..groups(size as u64) {
                let (start, len) = group_range(size as u64, index).unwrap();
                let group = &f[start as usize..(start + len) as usize];
                let proof = prove(&f, index).unwrap();
                assert!(
                    verify(&root, size as u64, index, group, &proof),
                    "size {size}, group {index}"
                );
            }
        }
    }

    #[test]
    fn a_changed_byte_a_wrong_index_a_wrong_size_or_a_cut_proof_is_refused() {
        let size = 100_000usize;
        let f = file(size);
        let root = *blake3::hash(&f).as_bytes();
        let index = 3;
        let (start, len) = group_range(size as u64, index).unwrap();
        let group = f[start as usize..(start + len) as usize].to_vec();
        let proof = prove(&f, index).unwrap();
        assert!(verify(&root, size as u64, index, &group, &proof));

        let mut changed = group.clone();
        changed[100] ^= 1;
        assert!(
            !verify(&root, size as u64, index, &changed, &proof),
            "a changed byte"
        );
        assert!(
            !verify(&root, size as u64, 2, &group, &proof),
            "another group's index"
        );
        // The proof binds the group to the root, not to the size: a size that leaves the group's place in the tree
        // where it was still verifies, so the size must come from the chain, never from the seeder.
        assert!(
            !verify(&root, 2 * size as u64, index, &group, &proof),
            "a size that moves the group in the tree"
        );
        let last = groups(size as u64) - 1;
        let (ls, ll) = group_range(size as u64, last).unwrap();
        let tail = &f[ls as usize..(ls + ll) as usize];
        assert!(
            !verify(
                &root,
                size as u64 + 1,
                last,
                tail,
                &prove(&f, last).unwrap()
            ),
            "the last group of another size"
        );
        assert!(
            !verify(&root, size as u64, index, &group, &proof[1..]),
            "a proof cut short"
        );
        let mut longer = proof.clone();
        longer.push([0; 32]);
        assert!(
            !verify(&root, size as u64, index, &group, &longer),
            "a proof made longer"
        );
        let mut wrong = proof.clone();
        wrong[0][0] ^= 1;
        assert!(
            !verify(&root, size as u64, index, &group, &wrong),
            "a changed sibling"
        );
        let mut other_root = root;
        other_root[31] ^= 1;
        assert!(
            !verify(&other_root, size as u64, index, &group, &proof),
            "another root"
        );
    }

    #[test]
    fn what_cannot_be_a_group_is_refused_without_panicking() {
        let root = [7u8; 32];
        assert!(!verify(&root, 100_000, 99, &[0; 10], &[]), "no such group");
        assert!(
            !verify(&root, 100_000, 0, &[], &[[0; 32]; 3]),
            "an empty group of a larger file"
        );
        assert!(
            !verify(&root, 100_000, 0, &[0; 20_000], &[[0; 32]; 3]),
            "a group longer than 16 KiB"
        );
        assert!(
            !verify(
                &root,
                u64::MAX,
                u64::MAX / GROUP,
                &[0; 16_384],
                &[[0; 32]; 64]
            ),
            "the largest file"
        );
        assert!(
            !verify(
                &root,
                1 << 30,
                5,
                &[0; 16_384],
                &vec![[0; 32]; MAX_DEPTH + 1]
            ),
            "too deep"
        );
        assert_eq!(prove(&[], 1), None);
    }

    /// What a seeder does with a file too large to hold: each sibling's range read in pieces and hashed as it arrives.
    #[test]
    fn a_proof_made_a_piece_at_a_time_is_the_same_proof() {
        for size in SIZES {
            let f = file(size);
            for index in 0..groups(size as u64) {
                let streamed: Vec<[u8; 32]> = siblings(size as u64, index)
                    .unwrap()
                    .into_iter()
                    .map(|(start, len)| {
                        let mut hasher = Subtree::at(start);
                        for piece in f[start as usize..(start + len) as usize].chunks(1000) {
                            hasher.update(piece);
                        }
                        hasher.finish()
                    })
                    .collect();
                assert_eq!(
                    Some(streamed),
                    prove(&f, index),
                    "size {size}, group {index}"
                );
            }
        }
        assert_eq!(siblings(100_000, 7), None);
    }

    #[test]
    fn a_proof_is_one_sibling_per_level() {
        assert_eq!(prove(&file(16_384), 0).unwrap().len(), 0);
        assert_eq!(prove(&file(16_385), 0).unwrap().len(), 1);
        // 64 groups: a complete tree six levels deep above the groups.
        assert_eq!(prove(&file(1 << 20), 17).unwrap().len(), 6);
    }
}
