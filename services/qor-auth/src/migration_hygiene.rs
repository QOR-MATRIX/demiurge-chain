//! The check that every migration is the same bytes on every platform.
//!
//! `sqlx::migrate!` embeds each file in `migrations/` at build time, and the first run against a
//! database records the SHA-384 of those bytes in `_sqlx_migrations`. Every later start compares the
//! two, and refuses to run if they differ: "migration N was previously applied but has been modified".
//!
//! A line ending is part of those bytes. With `core.autocrlf=true`, git on Windows checked these files
//! out with CRLF while Linux read LF, so a database migrated by a Windows build was refused by the
//! Linux build on Railway, and the reverse. Nothing in either build said why.
//!
//! `.gitattributes` now pins `services/qor-auth/migrations/*.sql` to LF on every platform. This reads
//! the files, and the copies compiled into this binary, and fails on a carriage return, so the rule
//! cannot be dropped, or an editor cannot convert a file, without a test saying so.
//!
//! A database that already holds CRLF checksums is corrected, without re-running anything, by
//! `scripts/correct-migration-checksums.ps1`.

use std::fs;
use std::path::Path;

/// What is wrong with one migration's bytes, if anything.
fn carriage_returns(name: &str, bytes: &[u8]) -> Option<String> {
    let count = bytes.iter().filter(|byte| **byte == b'\r').count();
    let line = bytes
        .split(|byte| *byte == b'\n')
        .position(|line| line.contains(&b'\r'))?;
    Some(format!(
        "{name}: {count} carriage return(s), the first on line {}",
        line + 1
    ))
}

const WHY: &str = "\
A migration holds a carriage return. sqlx records the SHA-384 of each migration's bytes in \
_sqlx_migrations, so a file with CRLF line endings has a different checksum from the same file \
with LF, and a database migrated by one build is refused by the other (\"was previously applied \
but has been modified\"). Migrations are LF on every platform: check that .gitattributes still has \
`services/qor-auth/migrations/*.sql text eol=lf`, then restore the file's line endings \
(delete it and `git checkout -- services/qor-auth/migrations`). Do not edit a checksum to match.";

#[test]
fn no_migration_file_holds_a_carriage_return() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let mut seen = 0;
    let mut problems = Vec::new();
    for entry in fs::read_dir(&dir).expect("the migrations directory") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_none_or(|extension| extension != "sql") {
            continue;
        }
        seen += 1;
        let name = path
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        problems.extend(carriage_returns(
            &name,
            &fs::read(&path).expect("a migration"),
        ));
    }

    // A clean result means nothing if no migration was read.
    assert!(
        seen >= 18,
        "the check should read the service's migrations; it read {seen}"
    );
    problems.sort();
    assert!(problems.is_empty(), "{WHY}\n{}", problems.join("\n"));
}

/// The files on disk can be right while the binary still holds what it was built from. This reads
/// what `main` will actually run and checksum.
#[test]
fn no_migration_compiled_into_the_binary_holds_a_carriage_return() {
    let migrator = sqlx::migrate!("./migrations");
    let mut seen = 0;
    let mut problems = Vec::new();
    for migration in migrator.iter() {
        seen += 1;
        problems.extend(carriage_returns(
            &format!("{} ({})", migration.version, migration.description),
            migration.sql.as_bytes(),
        ));
    }
    assert!(
        seen >= 18,
        "the check should read the embedded migrations; it read {seen}"
    );
    assert!(problems.is_empty(), "{WHY}\n{}", problems.join("\n"));
}

#[test]
fn a_carriage_return_is_found_and_a_file_without_one_passes() {
    for (bytes, expected) in [
        (
            &b"CREATE TABLE a (id INT);\r\nCREATE TABLE b (id INT);\r\n"[..],
            "2 carriage return(s), the first on line 1",
        ),
        (
            &b"-- a comment\nCREATE TABLE a (id INT);\r\n"[..],
            "1 carriage return(s), the first on line 2",
        ),
        // A bare carriage return changes the checksum as surely as CRLF does.
        (&b"SELECT 1;\n-- old Mac\rSELECT 2;\n"[..], "line 2"),
    ] {
        let found = carriage_returns("planted.sql", bytes);
        assert!(
            found
                .as_deref()
                .is_some_and(|problem| problem.contains(expected)),
            "{bytes:?} should be found as {expected:?}; found {found:?}"
        );
    }
    assert_eq!(
        carriage_returns("clean.sql", b"CREATE TABLE a (id INT);\n-- done\n"),
        None
    );
}
