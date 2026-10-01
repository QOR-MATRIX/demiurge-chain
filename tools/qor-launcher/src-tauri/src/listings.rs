//! The description of a listing, drafted by a creator for an asset they hold
//! (L4.6).
//!
//! # Nothing here is published, and the form says so
//!
//! **The listing a buyer can act on is not in this module.** It is a price on
//! chain, published and settled through `pallet-drc369-royalties`
//! ([`crate::chain::sales`], ADR-061). The chain holds that price and nothing
//! else: a title, a kind and a description have nowhere public to go until
//! there is an indexer to serve them and a storefront to show them (ADR-028,
//! M5.4, P5). What this module does is keep the creator's own description on
//! their own machine, so the work of describing an asset is not lost between
//! now and Market, and so the shape of a listing is decided by looking at a real
//! one rather than by imagining it.
//!
//! A draft is a file in the launcher's data directory. It is not sent anywhere,
//! not synchronised and not visible to anybody else. It carries the price that
//! was typed when it was saved, as a note; the price that counts is the chain's.
//!
//! # The vocabulary lives here, in one place
//!
//! The categories and the fields each one asks for are a table in this file, and
//! the interface renders whatever the table says — so "choosing Gaming reveals
//! the gaming questions" is data, not a second copy of the product's vocabulary
//! in TypeScript. **The table is a proposal**: the owner asked for Gaming,
//! Entertainment and "Physical Offline" and said "etc.", so the rest is filled
//! in here to be argued with, and naming is the owner's (AGENTS.md §8).
//!
//! It is deliberately NOT the same axis as Market's payload kinds — Game, Tool,
//! Editor plugin, QFX scene, QFX preset, Theme (`docs/blueprints/market.md`).
//! Those say what a buyer's machine does with the bytes, which is an install and
//! launch question. A category says what the thing IS, which is a browsing
//! question. One asset has both, and neither replaces the other.
//!
//! # Money
//!
//! A price is the creator's own number, in CGT, parsed exactly by
//! [`crate::cgt::parse_cgt`] — never a float, and excess precision is refused
//! rather than rounded (AGENTS.md §5). It invents no economic value: what a
//! platform takes from a sale is undecided (U-15) and appears nowhere here.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cgt;
use crate::error::{QorError, QorResult};

/// The owner's number, on 22 September 2026: a title is at most 30 characters.
pub const TITLE_LIMIT: usize = 30;
/// Characters, not bytes: a limit a person can count while typing.
pub const NOTES_LIMIT: usize = 600;
/// One answer to one question.
pub const DETAIL_LIMIT: usize = 120;

/// One question a category asks. No options means the answer is free text.
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub id: &'static str,
    pub label: &'static str,
    pub options: &'static [&'static str],
}

/// One kind of thing a listing can be about, and what it asks.
#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub id: &'static str,
    pub name: &'static str,
    /// Shown with the category when it needs to be said before anyone chooses
    /// it, rather than afterwards.
    pub note: Option<&'static str>,
    pub fields: &'static [Field],
}

const fn field(id: &'static str, label: &'static str, options: &'static [&'static str]) -> Field {
    Field { id, label, options }
}

/// **PROPOSED VOCABULARY (the owner's to confirm).** Ordered as it is offered.
pub const CATEGORIES: &[Category] = &[
    Category {
        id: "gaming",
        name: "Gaming",
        note: None,
        fields: &[
            field(
                "kind",
                "What it is",
                &["Game", "Tool", "Mod", "Editor plugin", "Asset pack"],
            ),
            field(
                "platform",
                "Runs on",
                &["Windows", "macOS", "Linux", "More than one"],
            ),
            field(
                "engine",
                "Built with",
                &[
                    "QOR Engine",
                    "Godot",
                    "Unity",
                    "Unreal",
                    "Something else",
                    "Not applicable",
                ],
            ),
            field(
                "players",
                "Players",
                &["One", "Local together", "Online together"],
            ),
        ],
    },
    Category {
        id: "entertainment",
        name: "Entertainment",
        note: None,
        fields: &[
            field(
                "kind",
                "What it is",
                &["Music", "Video", "Artwork", "Writing", "Photography"],
            ),
            field("length", "Length or size", &[]),
            field("explicit", "Explicit content", &["No", "Yes"]),
        ],
    },
    Category {
        id: "creative-tools",
        name: "Creative tools",
        note: None,
        fields: &[
            field(
                "kind",
                "What it is",
                &[
                    "Sample pack",
                    "Preset pack",
                    "Template",
                    "Font",
                    "3D model",
                    "Texture pack",
                ],
            ),
            field("works-with", "Works with", &[]),
        ],
    },
    Category {
        id: "software",
        name: "Software",
        note: None,
        fields: &[
            field("kind", "What it is", &["Application", "Library", "Script"]),
            field(
                "platform",
                "Runs on",
                &["Windows", "macOS", "Linux", "More than one"],
            ),
            field("source", "Source included", &["No", "Yes"]),
        ],
    },
    Category {
        id: "physical",
        name: "Physical (offline)",
        note: Some(
            "The chain moves the record, not the object. Whoever buys this owns the asset the \
             moment it settles; getting the physical thing to them is between the two of you, and \
             nothing here holds the money, checks that it arrived or settles an argument about it.",
        ),
        fields: &[
            field("arrives", "What actually arrives", &[]),
            field("ships-from", "Sent from", &[]),
            field(
                "who-ships",
                "Who sends it",
                &["You", "Someone else on your behalf"],
            ),
        ],
    },
    Category {
        id: "other",
        name: "Other",
        note: None,
        fields: &[field("what", "What it is", &[])],
    },
];

/// One answer, kept in the order the category asks its questions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detail {
    pub field: String,
    pub value: String,
}

/// A drafted description: what this creator would say about the asset, if there
/// were anywhere to show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    pub collection: u32,
    pub item: u32,
    pub title: String,
    pub category: String,
    pub details: Vec<Detail>,
    /// Exactly what the creator typed, so the form shows it back unchanged.
    pub price_cgt: String,
    /// The same number in Sparks, as a decimal string because JSON has no
    /// 128-bit integer. Parsed exactly; no float touches it.
    pub price_sparks: String,
    pub notes: String,
    pub created: u64,
    pub updated: u64,
}

/// What the interface sends when a creator saves.
#[derive(Debug, Clone, Deserialize)]
pub struct Draft {
    pub collection: u32,
    pub item: u32,
    pub title: String,
    pub category: String,
    pub details: Vec<Detail>,
    pub price_cgt: String,
    pub notes: String,
}

fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join("listing-drafts")
}

fn path(data_dir: &Path, collection: u32, item: u32) -> PathBuf {
    dir(data_dir).join(format!("{collection}-{item}.json"))
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default()
}

/// Save a draft, replacing whatever was there for that asset.
///
/// Everything a person can get wrong is refused here, before anything is
/// written: a title over the owner's thirty characters, a category this build
/// does not offer, an answer to a question that category never asks, and a price
/// that is not an exact amount of CGT.
pub fn save(data_dir: &Path, draft: &Draft) -> QorResult<Listing> {
    let title = draft.title.trim();
    if title.is_empty() {
        return Err(QorError::Qontrol("a listing needs a title".into()));
    }
    if title.chars().count() > TITLE_LIMIT {
        return Err(QorError::Qontrol(format!(
            "a title is at most {TITLE_LIMIT} characters, and this one is {}",
            title.chars().count()
        )));
    }

    let category = CATEGORIES
        .iter()
        .find(|known| known.id == draft.category)
        .ok_or_else(|| {
            QorError::Qontrol(format!("\"{}\" is not a kind of listing", draft.category))
        })?;

    let mut details = Vec::new();
    for detail in &draft.details {
        let value = detail.value.trim();
        if value.is_empty() {
            continue;
        }
        let asked = category
            .fields
            .iter()
            .find(|field| field.id == detail.field)
            .ok_or_else(|| {
                QorError::Qontrol(format!(
                    "\"{}\" is not something a {} listing asks",
                    detail.field, category.name
                ))
            })?;
        if value.chars().count() > DETAIL_LIMIT {
            return Err(QorError::Qontrol(format!(
                "\"{}\" is at most {DETAIL_LIMIT} characters",
                asked.label
            )));
        }
        if !asked.options.is_empty() && !asked.options.contains(&value) {
            return Err(QorError::Qontrol(format!(
                "\"{value}\" is not one of the answers to \"{}\"",
                asked.label
            )));
        }
        details.push(Detail {
            field: asked.id.to_string(),
            value: value.to_string(),
        });
    }

    let notes = draft.notes.trim();
    if notes.chars().count() > NOTES_LIMIT {
        return Err(QorError::Qontrol(format!(
            "the description is at most {NOTES_LIMIT} characters, and this one is {}",
            notes.chars().count()
        )));
    }

    // Exact, or refused. `parse_cgt` rejects excess precision rather than
    // truncating it, and no float is involved (ADR-035, AGENTS.md §5).
    let sparks = cgt::parse_cgt(&draft.price_cgt)?;
    if sparks == 0 {
        return Err(QorError::Qontrol(
            "a listing needs a price. Nothing here decides what it should be.".into(),
        ));
    }

    let created = load(data_dir, draft.collection, draft.item)
        .map(|was| was.created)
        .unwrap_or_else(now);
    let listing = Listing {
        collection: draft.collection,
        item: draft.item,
        title: title.to_string(),
        category: category.id.to_string(),
        details,
        price_cgt: cgt::format_cgt(sparks),
        price_sparks: sparks.to_string(),
        notes: notes.to_string(),
        created,
        updated: now(),
    };

    std::fs::create_dir_all(dir(data_dir))?;
    let json = serde_json::to_vec_pretty(&listing)
        .map_err(|e| QorError::Internal(format!("cannot write the listing: {e}")))?;
    std::fs::write(path(data_dir, listing.collection, listing.item), json)?;
    Ok(listing)
}

/// One asset's draft, if it has one.
pub fn load(data_dir: &Path, collection: u32, item: u32) -> Option<Listing> {
    let bytes = std::fs::read(path(data_dir, collection, item)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Every draft on this machine, most recently changed first.
pub fn list(data_dir: &Path) -> Vec<Listing> {
    let Ok(entries) = std::fs::read_dir(dir(data_dir)) else {
        return Vec::new();
    };
    let mut listings: Vec<Listing> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| std::fs::read(entry.path()).ok())
        .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
        .collect();
    listings.sort_by_key(|one| std::cmp::Reverse(one.updated));
    listings
}

/// Throw a draft away.
pub fn discard(data_dir: &Path, collection: u32, item: u32) -> QorResult<()> {
    match std::fs::remove_file(path(data_dir, collection, item)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_for(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qor-listings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a directory to test in");
        dir
    }

    fn draft() -> Draft {
        Draft {
            collection: 1,
            item: 0,
            title: "A song about rain".into(),
            category: "entertainment".into(),
            details: vec![
                Detail {
                    field: "kind".into(),
                    value: "Music".into(),
                },
                Detail {
                    field: "length".into(),
                    value: "3:41".into(),
                },
            ],
            price_cgt: "250.5".into(),
            notes: "Recorded in one take.".into(),
        }
    }

    #[test]
    fn a_draft_round_trips_and_its_price_is_exact() {
        let dir = dir_for("round-trip");
        let saved = save(&dir, &draft()).unwrap();
        assert_eq!(
            saved.price_sparks,
            (2505 * cgt::SPARKS_PER_CGT / 10).to_string()
        );
        assert_eq!(saved.price_cgt, "250.50");
        assert_eq!(load(&dir, 1, 0).unwrap(), saved);
        assert_eq!(list(&dir).len(), 1);

        discard(&dir, 1, 0).unwrap();
        assert!(load(&dir, 1, 0).is_none());
        assert!(
            discard(&dir, 1, 0).is_ok(),
            "discarding twice is not an error"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn what_a_person_can_get_wrong_is_refused_before_anything_is_written() {
        let dir = dir_for("refusals");
        let cases: Vec<(&str, Draft)> = vec![
            (
                "at most 30 characters",
                Draft {
                    title: "x".repeat(TITLE_LIMIT + 1),
                    ..draft()
                },
            ),
            (
                "needs a title",
                Draft {
                    title: "   ".into(),
                    ..draft()
                },
            ),
            (
                "is not a kind of listing",
                Draft {
                    category: "invented".into(),
                    ..draft()
                },
            ),
            (
                "is not something a",
                Draft {
                    details: vec![Detail {
                        field: "platform".into(),
                        value: "Windows".into(),
                    }],
                    ..draft()
                },
            ),
            (
                "is not one of the answers",
                Draft {
                    details: vec![Detail {
                        field: "kind".into(),
                        value: "Interpretive dance".into(),
                    }],
                    ..draft()
                },
            ),
            (
                "needs a price",
                Draft {
                    price_cgt: "0".into(),
                    ..draft()
                },
            ),
        ];
        for (expected, bad) in cases {
            let error = save(&dir, &bad).unwrap_err();
            assert!(
                error.to_string().contains(expected),
                "expected {expected:?}, got {error}"
            );
        }
        assert!(list(&dir).is_empty(), "nothing was written");

        // Excess precision is refused, not rounded: eighteen decimals is the
        // most CGT has.
        let too_precise = Draft {
            price_cgt: format!("1.{}", "1".repeat(19)),
            ..draft()
        };
        assert!(save(&dir, &too_precise).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_category_asks_something_and_names_itself() {
        for category in CATEGORIES {
            assert!(!category.id.is_empty() && !category.name.is_empty());
            assert!(!category.fields.is_empty(), "{} asks nothing", category.id);
            for field in category.fields {
                assert!(!field.id.is_empty() && !field.label.is_empty());
            }
        }
        // The one that needs saying before it is chosen, says it.
        let physical = CATEGORIES
            .iter()
            .find(|category| category.id == "physical")
            .expect("the physical category");
        let note = physical.note.expect("physical says what it cannot do");
        assert!(note.contains("not the object"));
    }
}
