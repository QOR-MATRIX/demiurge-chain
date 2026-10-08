//! Qontrol's host tests.
//!
//! Two of these exist because the owner asked for them by name, and both were
//! proven to fail before they were trusted: a commit that bypassed the index
//! leaves a clean-looking repository that lies, and two ignore implementations
//! that disagree lose a file quietly. Everything else here is ordinary cover.

use super::*;

/// No setup is needed to find the helper: `sidecar::binary()` already falls
/// back to `../qontrol-git/target/{debug,release}/`, which is where a clean
/// checkout builds it. The crate forbids `unsafe`, so the env-var shortcut is
/// not available here either — and it turns out not to be wanted.
/// gitoxide reads committer identity from configuration and refuses to write a
/// commit without one — correct, and unhelpful in a test.
fn identity(path: &Path) {
    let config = path.join(".git").join("config");
    let existing = std::fs::read_to_string(&config).unwrap_or_default();
    std::fs::write(
        config,
        format!("{existing}\n[user]\n\tname = Qontrol Test\n\temail = test@invalid\n"),
    )
    .expect("write config");
}

fn project(kind: Scaffold) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    scaffold(dir.path(), kind).expect("scaffold");
    identity(dir.path());
    dir
}

/// Is the helper there? The tests that commit cannot run without it.
fn helper_present(test: &str) -> bool {
    sidecar::available()
        || skip_or_fail(
            test,
            "the qontrol-git helper is not built \
             (`cargo build --manifest-path ../qontrol-git/Cargo.toml`)",
        )
}

/// Is `git` itself there? One test asks it for a second opinion.
fn git_present(test: &str) -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
        || skip_or_fail(test, "git is not installed or not usable")
}

/// Normally a missing tool is a skip, said out loud, because a missing build
/// step is not a failing repository. Under the `qontrol-no-skips` feature it is
/// a failure: a skipped test reports `ok`, so a run without the tool would
/// otherwise read as evidence that the split path works. Returns `false` when
/// it returns at all.
fn skip_or_fail(test: &str, why: &str) -> bool {
    if cfg!(feature = "qontrol-no-skips") {
        panic!("{test}: {why}, and this run does not allow skips");
    }
    eprintln!("skipped {test}: {why}");
    false
}

#[test]
fn a_scaffold_lays_out_folders_and_ignores() {
    for (kind, expected) in [
        (Scaffold::Code, "src"),
        (Scaffold::Music, "stems"),
        (Scaffold::Game, "scenes"),
    ] {
        let dir = project(kind);
        assert!(
            dir.path().join(expected).is_dir(),
            "{expected} exists for {kind:?}"
        );
        assert!(dir.path().join(".gitignore").is_file(), "an ignore file");
        assert!(dir.path().join("README.md").is_file(), "a README");
        assert!(dir.path().join(".git").is_dir(), "it is a repository");
    }
}

/// The music scaffold's whole argument: a render can be made again, a stem
/// cannot. If this inverts, the layout has stopped meaning anything.
#[test]
fn the_music_scaffold_ignores_renders_and_keeps_stems() {
    let dir = project(Scaffold::Music);
    let ignore = std::fs::read_to_string(dir.path().join(".gitignore")).expect("read");
    assert!(ignore.contains("renders/"), "renders are ignored");
    assert!(!ignore.contains("stems/"), "stems are NEVER ignored");
}

#[test]
fn a_fresh_project_has_no_history_and_that_is_not_an_error() {
    let dir = project(Scaffold::Code);
    let project = read(dir.path()).expect("read");
    assert!(project.history.is_empty(), "no commits yet");
    assert!(
        !project.changes.is_empty(),
        "the scaffold's files are untracked"
    );
}

#[test]
fn a_folder_that_is_not_a_repository_is_refused() {
    let dir = tempfile::tempdir().expect("temp dir");
    assert!(matches!(
        read(dir.path()),
        Err(QontrolError::NotARepository(_))
    ));
}

#[test]
fn an_empty_message_is_refused_before_anything_is_staged() {
    let dir = project(Scaffold::Code);
    assert!(matches!(
        commit(dir.path(), "   "),
        Err(QontrolError::EmptyMessage)
    ));
}

/// THE TEST THE SPLIT EXISTS FOR.
///
/// libgit2 stages and writes the tree in the sidecar; gitoxide writes the commit
/// here and moves the ref. Get that order wrong — commit a tree built in memory,
/// bypassing the index — and the objects are still correct while `.git/index`
/// goes stale, so the next status reports files as changed that were just
/// committed. Nothing errors. This pins it.
#[test]
fn after_a_commit_the_working_tree_is_clean() {
    let dir = project(Scaffold::Code);
    if !helper_present("after_a_commit_the_working_tree_is_clean") {
        return;
    }

    let before = read(dir.path()).expect("read before");
    assert!(!before.changes.is_empty(), "there is something to commit");

    let id = commit(dir.path(), "Lay out the project").expect("commit");
    assert_eq!(id.len(), 40, "a commit id is 40 hex characters");

    let after = read(dir.path()).expect("read after");
    assert!(
        after.changes.is_empty(),
        "the tree must be clean after a commit; a stale index would report \
         these as still changed: {:?}",
        after.changes
    );
    assert_eq!(after.history.len(), 1, "one commit in the history");
    assert_eq!(after.history[0].summary, "Lay out the project");
}

/// IGNORE AGREEMENT.
///
/// Two implementations answer the same question here: gitoxide's decides what
/// `status` shows, libgit2's decides what gets staged. If they disagree, a file
/// a creator believes is ignored gets committed, or one they expect to commit
/// never arrives — and neither failure announces itself.
#[test]
fn the_two_ignore_implementations_agree() {
    let dir = project(Scaffold::Code);
    if !helper_present("the_two_ignore_implementations_agree") {
        return;
    }

    std::fs::write(dir.path().join(".gitignore"), "secret.txt\nbuild/\n").expect("ignore");
    std::fs::write(dir.path().join("secret.txt"), "not this").expect("secret");
    std::fs::create_dir_all(dir.path().join("build")).expect("build dir");
    std::fs::write(dir.path().join("build").join("out.bin"), "nor this").expect("out");
    std::fs::write(dir.path().join("kept.txt"), "this one").expect("kept");

    // gitoxide's answer.
    let seen: Vec<String> = read(dir.path())
        .expect("read")
        .changes
        .into_iter()
        .map(|c| c.path)
        .collect();
    assert!(
        seen.iter().any(|p| p == "kept.txt"),
        "gitoxide sees the kept file, got {seen:?}"
    );
    assert!(
        !seen.iter().any(|p| p.contains("secret.txt")),
        "gitoxide must not surface an ignored file, got {seen:?}"
    );
    assert!(
        !seen.iter().any(|p| p.starts_with("build/")),
        "gitoxide must not surface an ignored directory, got {seen:?}"
    );

    // libgit2's answer: what actually lands. If it staged the ignored files,
    // they would be committed and the tree would still read clean — so the
    // disagreement is caught by what is left behind, not by the commit failing.
    commit(dir.path(), "Only what is not ignored").expect("commit");
    let after = read(dir.path()).expect("read after");
    assert!(
        after.changes.is_empty(),
        "leftovers mean the two disagreed: {:?}",
        after.changes
    );

    // And the ignored paths are absent from the committed tree.
    let repo = gix::open(dir.path()).expect("open");
    let head = repo.head_id().expect("head").object().expect("object");
    let tree = head.into_commit().tree().expect("tree");
    assert!(
        tree.lookup_entry_by_path("secret.txt")
            .ok()
            .flatten()
            .is_none(),
        "an ignored file must not be in the commit"
    );
    assert!(
        tree.lookup_entry_by_path("kept.txt")
            .ok()
            .flatten()
            .is_some(),
        "a tracked file must be in the commit"
    );
}

#[test]
fn committing_twice_keeps_both_commits_newest_first() {
    let dir = project(Scaffold::Code);
    if !helper_present("committing_twice_keeps_both_commits_newest_first") {
        return;
    }
    commit(dir.path(), "First").expect("first");
    std::fs::write(dir.path().join("src").join("main.txt"), "more").expect("write");
    commit(dir.path(), "Second").expect("second");

    let project = read(dir.path()).expect("read");
    assert_eq!(project.history.len(), 2);
    assert_eq!(project.history[0].summary, "Second", "newest first");
    assert_eq!(project.history[1].summary, "First");
}

#[test]
fn committing_nothing_is_refused() {
    let dir = project(Scaffold::Code);
    if !helper_present("committing_nothing_is_refused") {
        return;
    }
    commit(dir.path(), "Everything").expect("first");
    assert!(matches!(
        commit(dir.path(), "Again"),
        Err(QontrolError::NothingToCommit)
    ));
}

#[test]
fn a_repository_reports_its_branch() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_repository_reports_its_branch") {
        return;
    }
    commit(dir.path(), "First").expect("commit");
    let project = read(dir.path()).expect("read");
    assert!(project.branch.is_some(), "a branch is reported");
    assert!(
        project.branches.iter().any(|b| b.head),
        "one branch is marked as the one you are on"
    );
}

/// The same claim, asked of git itself.
///
/// The test above is gitoxide checking gitoxide's own work. This one asks the
/// implementation that actually matters to a creator: the `git` they already
/// have. If Qontrol's commit left the index stale, `git status --porcelain`
/// prints a line per file and this fails. Skipped where git is absent, because
/// a missing tool is not a failing repository — except under `qontrol-no-skips`.
#[test]
fn git_itself_agrees_the_tree_is_clean() {
    let dir = project(Scaffold::Code);
    let test = "git_itself_agrees_the_tree_is_clean";
    if !helper_present(test) || !git_present(test) {
        return;
    }

    commit(dir.path(), "Lay out the project").expect("commit");

    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir.path())
        .output()
        .expect("git status");
    let reported = String::from_utf8_lossy(&out.stdout);

    assert!(
        reported.trim().is_empty(),
        "git status must be clean after a Qontrol commit. It printed:\n{reported}"
    );
}

// ---------------------------------------------------------------------------
// Publishing a version (M4.1): what a mint pins
// ---------------------------------------------------------------------------

/// A mint pins the commit HEAD points at, by its id, and fingerprints every
/// file in that commit's tree — not the folder, and not a branch.
#[test]
fn a_snapshot_pins_the_head_commit_and_fingerprints_its_files() {
    let dir = project(Scaffold::Music);
    if !helper_present("a_snapshot_pins_the_head_commit_and_fingerprints_its_files") {
        return;
    }
    std::fs::write(dir.path().join("stems").join("vox.wav"), b"RIFF take one").expect("stem");
    let id = commit(dir.path(), "First take").expect("commit");

    let snapshot = publish::snapshot(dir.path()).expect("snapshot");
    assert_eq!(
        snapshot.commit.hex(),
        id,
        "the commit HEAD points at, by id"
    );
    assert_eq!(snapshot.commit.kind(), "SHA-1");
    assert!(
        snapshot.branch.is_some(),
        "the branch is recorded for a reader"
    );

    let paths: Vec<&str> = snapshot
        .manifest
        .entries
        .iter()
        .map(|e| e.path.as_str())
        .collect();
    assert!(paths.contains(&"stems/vox.wav"), "{paths:?}");
    assert!(paths.contains(&"README.md"), "{paths:?}");
    assert!(paths.contains(&".gitignore"), "{paths:?}");
    for entry in &snapshot.manifest.entries {
        let on_disk = std::fs::read(dir.path().join(&entry.path)).expect("the file");
        assert_eq!(
            entry.content,
            crate::content::ContentRef::of(&on_disk),
            "{} is fingerprinted from its committed bytes",
            entry.path
        );
    }
    assert_eq!(snapshot.reference, snapshot.manifest.reference());

    // The same commit makes the same asset, however often it is asked.
    let again = publish::snapshot(dir.path()).expect("again");
    assert_eq!(again.reference, snapshot.reference);

    // Once approved, every file and the manifest land in the temporary store.
    let data = tempfile::tempdir().expect("data dir");
    let store = crate::content::TemporaryStore::in_data_dir(data.path());
    snapshot.write_to(dir.path(), &store).expect("store");
    assert_eq!(
        std::fs::read(store.path_of(&snapshot.reference)).expect("the manifest"),
        snapshot.manifest.bytes()
    );
    let vox = snapshot
        .manifest
        .entries
        .iter()
        .find(|e| e.path == "stems/vox.wav")
        .expect("the stem");
    assert_eq!(
        std::fs::read(store.path_of(&vox.content)).expect("the stem's bytes"),
        b"RIFF take one"
    );
}

/// Uncommitted work is refused rather than silently left out of the asset.
#[test]
fn a_snapshot_refuses_uncommitted_changes() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_snapshot_refuses_uncommitted_changes") {
        return;
    }
    commit(dir.path(), "Lay out the project").expect("commit");
    std::fs::write(dir.path().join("README.md"), "changed after the commit").expect("edit");

    let refused = publish::snapshot(dir.path()).unwrap_err().to_string();
    assert!(refused.contains("commit first"), "{refused}");
}

/// A repository with nothing committed has nothing to pin.
#[test]
fn a_snapshot_of_a_repository_with_no_commit_is_refused() {
    let dir = tempfile::tempdir().expect("temp dir");
    open_or_init(dir.path()).expect("init");
    let refused = publish::snapshot(dir.path()).unwrap_err().to_string();
    assert!(refused.contains("no commit to mint"), "{refused}");
}

// ---------------------------------------------------------------------------
// Diffs (P1.1): what changed inside a file, as git would say it
// ---------------------------------------------------------------------------

/// Append a setting to the repository's own configuration. The machine's
/// global configuration is not something a test gets to assume: Git for
/// Windows ships `core.autocrlf = true` system-wide, which would make a test
/// about attributes pass for the wrong reason.
fn configure(path: &Path, section: &str, key: &str, value: &str) {
    let config = path.join(".git").join("config");
    let existing = std::fs::read_to_string(&config).unwrap_or_default();
    std::fs::write(
        config,
        format!("{existing}\n[{section}]\n\t{key} = {value}\n"),
    )
    .expect("write config");
}

fn one(dir: &Path, file: &str) -> FileDiff {
    let mut diffs = diff(dir, Some(file)).expect("diff");
    assert_eq!(diffs.len(), 1, "one diff for {file}: {diffs:?}");
    diffs.remove(0)
}

fn lines(body: &DiffBody) -> Vec<(&'static str, String)> {
    match body {
        DiffBody::Text { lines, .. } => lines
            .iter()
            .filter(|l| l.kind != "hunk")
            .map(|l| (l.kind, l.text.clone()))
            .collect(),
        other => panic!("expected a text diff, got {other:?}"),
    }
}

fn changed_only(body: &DiffBody) -> Vec<(&'static str, String)> {
    lines(body)
        .into_iter()
        .filter(|(k, _)| *k != "context")
        .collect()
}

/// Nothing listed, or listed as `Same`: both mean "nothing changed", and which
/// one depends on whether status looked at the content or only the timestamp.
fn nothing_changed(dir: &Path, file: &str) -> bool {
    let diffs = diff(dir, Some(file)).expect("diff");
    diffs.iter().all(|d| d.body == DiffBody::Same)
}

const POEM: &str = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\n";

#[test]
fn a_changed_line_is_shown_as_one_removal_and_one_addition() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_changed_line_is_shown_as_one_removal_and_one_addition") {
        return;
    }
    std::fs::write(dir.path().join("poem.txt"), POEM).expect("write");
    commit(dir.path(), "A poem").expect("commit");

    std::fs::write(dir.path().join("poem.txt"), POEM.replace("five", "FIVE")).expect("edit");
    let d = one(dir.path(), "poem.txt");
    assert_eq!(d.state, "modified");
    assert_eq!(
        changed_only(&d.body),
        vec![("remove", "five".into()), ("add", "FIVE".into())]
    );

    // Line numbers are the file's own: line five, on both sides.
    let DiffBody::Text { lines, .. } = &d.body else {
        unreachable!()
    };
    let removed = lines.iter().find(|l| l.kind == "remove").expect("removal");
    let added = lines.iter().find(|l| l.kind == "add").expect("addition");
    assert_eq!((removed.old, removed.new), (Some(5), None));
    assert_eq!((added.old, added.new), (None, Some(5)));
    assert!(
        lines
            .iter()
            .any(|l| l.kind == "context" && l.text == "four"),
        "context around the change"
    );
}

/// THE REASON THE DIFF WAS NOT WRITTEN IN THE FIRST SLICE.
///
/// With `core.autocrlf = true`, git stores LF and the file on disk has CRLF.
/// Compare those bytes directly and every line differs: a creator on Windows
/// would see their whole file rewritten when they changed one word. Git reads
/// the working-tree side through its filters first, and so must this.
#[test]
fn core_autocrlf_is_honoured_as_git_honours_it() {
    let dir = project(Scaffold::Code);
    let test = "core_autocrlf_is_honoured_as_git_honours_it";
    if !helper_present(test) {
        return;
    }
    configure(dir.path(), "core", "autocrlf", "true");
    std::fs::write(dir.path().join("poem.txt"), POEM).expect("write");
    commit(dir.path(), "A poem").expect("commit");

    // The same text with CRLF endings, which is what checking it out under
    // this configuration writes. No change in content.
    let crlf = POEM.replace('\n', "\r\n");
    std::fs::write(dir.path().join("poem.txt"), &crlf).expect("rewrite");
    assert!(
        nothing_changed(dir.path(), "poem.txt"),
        "line endings the configuration converts are not a change: {:?}",
        diff(dir.path(), Some("poem.txt"))
    );

    // One word changed, still CRLF: one line out, one line in. Not eight.
    std::fs::write(dir.path().join("poem.txt"), crlf.replace("five", "FIVE")).expect("edit");
    assert_eq!(
        changed_only(&one(dir.path(), "poem.txt").body),
        vec![("remove", "five".into()), ("add", "FIVE".into())],
        "only the changed line"
    );

    // And git, asked the same question, agrees on the count.
    if git_present(test) {
        let out = std::process::Command::new("git")
            .args(["diff", "--numstat", "--", "poem.txt"])
            .current_dir(dir.path())
            .output()
            .expect("git diff");
        let numstat = String::from_utf8_lossy(&out.stdout);
        assert!(
            numstat.starts_with("1\t1\t"),
            "git counts one line in and one out; it printed {numstat:?}"
        );
    }
}

/// The same, with the rule in `.gitattributes` rather than configuration, and
/// configuration explicitly off so the attribute is the only thing that can
/// make it pass.
#[test]
fn a_gitattributes_text_rule_is_honoured() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_gitattributes_text_rule_is_honoured") {
        return;
    }
    configure(dir.path(), "core", "autocrlf", "false");
    std::fs::write(dir.path().join(".gitattributes"), "*.txt text\n").expect("attributes");
    std::fs::write(dir.path().join("poem.txt"), POEM).expect("write");
    commit(dir.path(), "A poem").expect("commit");

    std::fs::write(dir.path().join("poem.txt"), POEM.replace('\n', "\r\n")).expect("rewrite");
    assert!(
        nothing_changed(dir.path(), "poem.txt"),
        "`text` normalises line endings on the way in, so CRLF on disk is not a change: {:?}",
        diff(dir.path(), Some("poem.txt"))
    );
}

/// `-diff` in `.gitattributes` says "never show this as lines", which is what
/// a music project wants on audio. Honoured even when the bytes are text.
#[test]
fn a_gitattributes_no_diff_rule_makes_a_file_binary() {
    let dir = project(Scaffold::Music);
    if !helper_present("a_gitattributes_no_diff_rule_makes_a_file_binary") {
        return;
    }
    std::fs::write(dir.path().join(".gitattributes"), "*.dat -diff\n").expect("attributes");
    std::fs::write(dir.path().join("take.dat"), "plain text\n").expect("write");
    commit(dir.path(), "A take").expect("commit");

    std::fs::write(dir.path().join("take.dat"), "other text, longer\n").expect("edit");
    assert_eq!(
        one(dir.path(), "take.dat").body,
        DiffBody::Binary {
            old_size: Some(11),
            new_size: Some(19)
        }
    );
}

/// Content git sniffs as binary, a NUL byte, is binary without any rule.
#[test]
fn content_with_a_nul_byte_is_binary() {
    let dir = project(Scaffold::Code);
    std::fs::write(dir.path().join("blob.bin"), b"a\0b").expect("write");
    assert_eq!(
        one(dir.path(), "blob.bin").body,
        DiffBody::Binary {
            old_size: None,
            new_size: Some(3)
        }
    );
}

#[test]
fn an_untracked_file_is_all_additions_numbered_from_one() {
    let dir = project(Scaffold::Code);
    std::fs::write(dir.path().join("new.txt"), "alpha\nbeta\n").expect("write");
    let d = one(dir.path(), "new.txt");
    assert_eq!(d.state, "untracked");
    let DiffBody::Text { lines, truncated } = &d.body else {
        panic!("text: {:?}", d.body)
    };
    assert!(!truncated);
    let added: Vec<_> = lines
        .iter()
        .filter(|l| l.kind == "add")
        .map(|l| (l.new, l.text.as_str()))
        .collect();
    assert_eq!(added, vec![(Some(1), "alpha"), (Some(2), "beta")]);
    assert!(lines.iter().all(|l| l.kind != "remove"));
}

#[test]
fn a_removed_file_is_all_removals() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_removed_file_is_all_removals") {
        return;
    }
    std::fs::write(dir.path().join("gone.txt"), "alpha\nbeta\n").expect("write");
    commit(dir.path(), "Soon gone").expect("commit");
    std::fs::remove_file(dir.path().join("gone.txt")).expect("remove");

    let d = one(dir.path(), "gone.txt");
    assert_eq!(d.state, "removed");
    assert_eq!(
        lines(&d.body),
        vec![("remove", "alpha".into()), ("remove", "beta".into())]
    );
}

#[test]
fn a_new_empty_file_is_empty_not_unchanged() {
    let dir = project(Scaffold::Code);
    std::fs::write(dir.path().join("empty.txt"), "").expect("write");
    assert_eq!(one(dir.path(), "empty.txt").body, DiffBody::Empty);
}

/// A reformat can change every line of a large file. The diff stops, and says
/// that it stopped, rather than ending where the reader cannot tell.
#[test]
fn a_huge_diff_is_cut_short_and_says_so() {
    let dir = project(Scaffold::Code);
    let text: String = (0..diff::MAX_LINES + 50)
        .map(|i| format!("{i}\n"))
        .collect();
    std::fs::write(dir.path().join("big.txt"), text).expect("write");
    let DiffBody::Text { lines, truncated } = one(dir.path(), "big.txt").body else {
        panic!("text")
    };
    assert!(truncated, "it says it was cut short");
    assert!(lines.len() <= diff::MAX_LINES);
}

/// Without a path, every change gets a diff, in the order the list shows them.
#[test]
fn without_a_path_every_change_has_a_diff() {
    let dir = project(Scaffold::Code);
    let project = read(dir.path()).expect("read");
    let diffs = diff(dir.path(), None).expect("diff");
    let listed: Vec<_> = project.changes.iter().map(|c| c.path.clone()).collect();
    let diffed: Vec<_> = diffs.iter().map(|d| d.path.clone()).collect();
    assert_eq!(listed, diffed);
}

// ---------------------------------------------------------------------------
// P1.2: the guard before staging, per-file commit, branch, switch, discard.
// ---------------------------------------------------------------------------

/// A private key, split so this source file is not itself credential-shaped.
fn pem_key() -> String {
    format!(
        "-----BEGIN {}PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASC\n-----END {}PRIVATE KEY-----\n",
        "RSA ", "RSA "
    )
}

fn changed_paths(dir: &Path) -> Vec<String> {
    read(dir)
        .expect("read")
        .changes
        .into_iter()
        .map(|c| c.path)
        .collect()
}

fn committed(dir: &Path, path: &str) -> bool {
    let repo = gix::open(dir).expect("open");
    let head = repo.head_id().expect("head").object().expect("object");
    let tree = head.into_commit().tree().expect("tree");
    tree.lookup_entry_by_path(path).ok().flatten().is_some()
}

/// Stage-on-commit is how a secret enters history, so a file named for where
/// secrets live stops the commit until the person says yes to that file, and
/// the refusal names the file and never what is in it.
#[test]
fn a_credential_file_stops_a_commit_until_it_is_accepted() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_credential_file_stops_a_commit_until_it_is_accepted") {
        return;
    }
    // The Code scaffold ignores `.env`; a creator who removed that line is the
    // case this exists for.
    std::fs::write(dir.path().join(".gitignore"), "target/\n").expect("ignore");
    std::fs::write(dir.path().join(".env"), "API_TOKEN=do-not-print-me\n").expect("env");

    let warnings = guard::check(dir.path(), None).expect("check");
    assert_eq!(warnings.len(), 1, "one warning: {warnings:?}");
    assert_eq!(warnings[0].path, ".env");
    assert_eq!(warnings[0].concern, guard::Concern::Credential);

    let refused = commit(dir.path(), "Everything").expect_err("refused without a yes");
    let said = refused.to_string();
    assert!(said.contains(".env"), "the refusal names the file: {said}");
    assert!(
        !said.contains("do-not-print-me"),
        "the refusal must never carry the secret: {said}"
    );
    assert!(
        read(dir.path()).expect("read").history.is_empty(),
        "nothing was committed"
    );

    let accepted = [guard::Accepted {
        path: ".env".into(),
        concern: guard::Concern::Credential,
    }];
    commit_selected(dir.path(), "Everything, knowingly", None, &accepted).expect("accepted");
    assert!(committed(dir.path(), ".env"));
}

/// A key is found by what it is, whatever the file is called.
#[test]
fn a_private_key_is_caught_by_its_content_whatever_it_is_called() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_private_key_is_caught_by_its_content_whatever_it_is_called") {
        return;
    }
    std::fs::write(dir.path().join("docs").join("notes.txt"), pem_key()).expect("write");

    let warnings = guard::check(dir.path(), None).expect("check");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(warnings[0].path, "docs/notes.txt");
    assert_eq!(warnings[0].reason, "looks like a private key");
}

/// The guard reads the helper's own list, so a secret the repository ignores
/// is not warned about: it was never going to be committed.
#[test]
fn an_ignored_secret_is_not_warned_about() {
    let dir = project(Scaffold::Code);
    if !helper_present("an_ignored_secret_is_not_warned_about") {
        return;
    }
    std::fs::write(dir.path().join(".env"), "API_TOKEN=x\n").expect("env");
    let warnings = guard::check(dir.path(), None).expect("check");
    assert!(
        warnings.is_empty(),
        "the scaffold ignores .env: {warnings:?}"
    );
    commit(dir.path(), "Everything").expect("commit");
    assert!(!committed(dir.path(), ".env"));
}

/// A Keynote deck is a `.key`. A creator's slides are not a credential.
#[test]
fn a_keynote_deck_is_not_a_credential() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_keynote_deck_is_not_a_credential") {
        return;
    }
    std::fs::write(dir.path().join("docs").join("talk.key"), "PK slides").expect("write");
    assert!(guard::check(dir.path(), None).expect("check").is_empty());
}

#[test]
fn a_large_file_is_held_back_until_it_is_accepted() {
    let dir = project(Scaffold::Music);
    if !helper_present("a_large_file_is_held_back_until_it_is_accepted") {
        return;
    }
    let take = dir.path().join("stems").join("take.wav");
    let file = std::fs::File::create(&take).expect("create");
    file.set_len(guard::LARGE + 1).expect("size");
    drop(file);

    let warnings = guard::check(dir.path(), None).expect("check");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(warnings[0].path, "stems/take.wav");
    assert_eq!(warnings[0].concern, guard::Concern::Large);

    // A yes to a different concern is not a yes to this one.
    let wrong = [guard::Accepted {
        path: "stems/take.wav".into(),
        concern: guard::Concern::Credential,
    }];
    assert!(matches!(
        commit_selected(dir.path(), "The take", None, &wrong),
        Err(QontrolError::Unconfirmed(_))
    ));
}

/// A per-file commit takes what was chosen and leaves the rest as it was.
#[test]
fn a_per_file_commit_leaves_the_other_changes_alone() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_per_file_commit_leaves_the_other_changes_alone") {
        return;
    }
    commit(dir.path(), "Lay out the project").expect("first");
    std::fs::write(dir.path().join("README.md"), "changed").expect("write");
    std::fs::write(dir.path().join("src").join("new.txt"), "new").expect("write");

    let files = vec!["src/new.txt".to_string()];
    commit_selected(dir.path(), "Only the new file", Some(&files), &[]).expect("commit");

    assert!(committed(dir.path(), "src/new.txt"));
    assert_eq!(
        changed_paths(dir.path()),
        vec!["README.md".to_string()],
        "the other change is still there, uncommitted"
    );
}

/// The interface can send any string. A commit or a discard acts on the
/// changes the repository reports, and nothing else.
#[test]
fn a_file_that_is_not_a_change_is_refused() {
    let dir = project(Scaffold::Code);
    let files = vec!["../../outside.txt".to_string()];
    assert!(matches!(
        commit_selected(dir.path(), "Sneaky", Some(&files), &[]),
        Err(QontrolError::Refused(_))
    ));
    assert!(matches!(
        discard(dir.path(), &files),
        Err(QontrolError::Refused(_))
    ));
}

#[test]
fn a_branch_is_made_at_head_and_not_switched_to() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_branch_is_made_at_head_and_not_switched_to") {
        return;
    }
    commit(dir.path(), "First").expect("commit");
    let before = read(dir.path()).expect("read");

    create_branch(dir.path(), "sketch").expect("branch");

    let after = read(dir.path()).expect("read");
    assert_eq!(after.branch, before.branch, "still on the same branch");
    assert!(after.branches.iter().any(|b| b.name == "sketch" && !b.head));

    let repo = gix::open(dir.path()).expect("open");
    let sketch = repo
        .find_reference("refs/heads/sketch")
        .expect("ref")
        .into_fully_peeled_id()
        .expect("id");
    assert_eq!(sketch.to_string(), before.history[0].id, "made at HEAD");
}

#[test]
fn a_branch_that_exists_or_has_a_bad_name_is_refused() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_branch_that_exists_or_has_a_bad_name_is_refused") {
        return;
    }
    commit(dir.path(), "First").expect("commit");
    create_branch(dir.path(), "sketch").expect("branch");

    let again = create_branch(dir.path(), "sketch").expect_err("exists");
    assert!(again.to_string().contains("already exists"), "{again}");
    for bad in ["", "-f", "a..b", "with space", "x~1"] {
        assert!(
            matches!(
                create_branch(dir.path(), bad),
                Err(QontrolError::Refused(_))
            ),
            "{bad:?} must be refused"
        );
    }
}

#[test]
fn a_branch_before_any_commit_is_refused() {
    let dir = project(Scaffold::Code);
    let refused = create_branch(dir.path(), "sketch").expect_err("no commit yet");
    assert!(refused.to_string().contains("first commit"), "{refused}");
}

#[test]
fn switching_shows_that_branchs_history_and_carries_a_new_file() {
    let dir = project(Scaffold::Code);
    if !helper_present("switching_shows_that_branchs_history_and_carries_a_new_file") {
        return;
    }
    commit(dir.path(), "First").expect("commit");
    let home = read(dir.path()).expect("read").branch.expect("a branch");
    create_branch(dir.path(), "sketch").expect("branch");
    switch(dir.path(), "sketch").expect("switch");
    std::fs::write(dir.path().join("src").join("idea.txt"), "idea").expect("write");
    commit(dir.path(), "On sketch").expect("commit");

    switch(dir.path(), &home).expect("back");
    let there = read(dir.path()).expect("read");
    assert_eq!(there.branch.as_deref(), Some(home.as_str()));
    assert_eq!(
        there.history.len(),
        1,
        "home does not have the sketch commit"
    );
    assert!(!dir.path().join("src").join("idea.txt").exists());

    std::fs::write(dir.path().join("docs").join("draft.txt"), "draft").expect("write");
    switch(dir.path(), "sketch").expect("switch with a new file");
    let sketch = read(dir.path()).expect("read");
    assert_eq!(sketch.history[0].summary, "On sketch");
    assert!(
        sketch.changes.iter().any(|c| c.path == "docs/draft.txt"),
        "the uncommitted file came along"
    );
}

/// The switch that would lose work. Refused, in plain words, with the edit and
/// the branch both left exactly as they were.
#[test]
fn a_switch_that_would_overwrite_an_edit_is_refused_and_touches_nothing() {
    let dir = project(Scaffold::Code);
    if !helper_present("a_switch_that_would_overwrite_an_edit_is_refused_and_touches_nothing") {
        return;
    }
    let readme = dir.path().join("README.md");
    commit(dir.path(), "First").expect("commit");
    let home = read(dir.path()).expect("read").branch.expect("a branch");
    create_branch(dir.path(), "sketch").expect("branch");
    switch(dir.path(), "sketch").expect("switch");
    std::fs::write(&readme, "the sketch readme").expect("write");
    commit(dir.path(), "On sketch").expect("commit");
    switch(dir.path(), &home).expect("back");

    std::fs::write(&readme, "an edit not yet committed").expect("write");
    let refused = switch(dir.path(), "sketch").expect_err("must refuse");

    assert!(
        refused.to_string().contains("would overwrite"),
        "said plainly: {refused}"
    );
    assert_eq!(
        read(dir.path()).expect("read").branch.as_deref(),
        Some(home.as_str())
    );
    assert_eq!(
        std::fs::read_to_string(&readme).expect("read"),
        "an edit not yet committed"
    );
}

#[test]
fn discarding_a_change_restores_it_and_leaves_the_rest() {
    let dir = project(Scaffold::Code);
    if !helper_present("discarding_a_change_restores_it_and_leaves_the_rest") {
        return;
    }
    commit(dir.path(), "First").expect("commit");
    let original = std::fs::read_to_string(dir.path().join("README.md")).expect("read");
    std::fs::write(dir.path().join("README.md"), "a mistake").expect("write");
    std::fs::remove_file(dir.path().join("src").join(".gitkeep")).expect("remove");
    std::fs::write(dir.path().join("docs").join(".gitkeep"), "kept edit").expect("write");

    let files = vec!["README.md".to_string(), "src/.gitkeep".to_string()];
    discard(dir.path(), &files).expect("discard");

    // Compared with line endings set aside: under `core.autocrlf` the restored
    // file is written with CRLF, exactly as `git restore` writes it, and git
    // reports it unchanged (the assertion on `changed_paths` below).
    assert_eq!(
        std::fs::read_to_string(dir.path().join("README.md"))
            .expect("read")
            .replace("\r\n", "\n"),
        original
    );
    assert!(dir.path().join("src").join(".gitkeep").exists(), "restored");
    assert_eq!(
        changed_paths(dir.path()),
        vec!["docs/.gitkeep".to_string()],
        "the change not named is untouched"
    );
}

/// Discarding a new file would mean deleting it. Refused, and it survives.
#[test]
fn discarding_a_new_file_is_refused_and_it_survives() {
    let dir = project(Scaffold::Code);
    if !helper_present("discarding_a_new_file_is_refused_and_it_survives") {
        return;
    }
    commit(dir.path(), "First").expect("commit");
    let new = dir.path().join("src").join("only-copy.txt");
    std::fs::write(&new, "the only copy").expect("write");

    let refused = discard(dir.path(), &["src/only-copy.txt".to_string()]).expect_err("refused");
    assert!(
        refused.to_string().contains("never been committed"),
        "{refused}"
    );
    assert_eq!(
        std::fs::read_to_string(&new).expect("read"),
        "the only copy"
    );
}

#[test]
fn credential_shapes_are_recognised_at_a_word_boundary_only() {
    let github = format!("token = {}{}", "ghp_", "a".repeat(36));
    let aws = format!("key={}{}", "AKIA", "ABCDEFGHIJKLMNOP");
    assert_eq!(
        guard::credential_shape(github.as_bytes()),
        Some("looks like a GitHub token")
    );
    assert_eq!(
        guard::credential_shape(aws.as_bytes()),
        Some("looks like an AWS access key")
    );
    assert_eq!(
        guard::credential_shape(pem_key().as_bytes()),
        Some("looks like a private key")
    );

    // Too short, inside a word, or a public key: not credentials.
    let short = format!("{}{}", "ghp_", "a".repeat(10));
    let inside = format!("TAKIA{}", "ABCDEFGHIJKLMNOP");
    let public = "-----BEGIN PUBLIC KEY-----\nMIIB\n-----END PUBLIC KEY-----\n";
    for text in [
        short.as_str(),
        inside.as_str(),
        public,
        "an ordinary sentence",
    ] {
        assert_eq!(guard::credential_shape(text.as_bytes()), None, "{text}");
    }
}

/// QQ's promise for P3.1 (ADR-082): a scene saved into a game project commits like any other file, and changing one
/// value in the editor is one changed line in the diff Projects shows, not a rewritten file.
#[test]
fn a_qq_scene_commits_and_one_changed_value_is_one_changed_line() {
    let dir = project(Scaffold::Game);
    if !helper_present("a_qq_scene_commits_and_one_changed_value_is_one_changed_line") {
        return;
    }
    // Written as QQ's editor writes it: one component per line (src/qq/runtime/scene.ts).
    let scene = |glow: &str| {
        format!(
            "{{\n  \"qq\": 1,\n  \"name\": \"level\",\n  \"size\": {{ \"w\": 960, \"h\": 540 }},\n  \"background\": \"#07080c\",\n  \"entities\": [\n    {{\n      \"id\": \"e1\",\n      \"name\": \"Core\",\n      \"transform\": {{ \"x\": 0, \"y\": 0, \"rotation\": 0, \"scale\": 1 }},\n      \"shape\": {{ \"kind\": \"circle\", \"w\": 84, \"h\": 84, \"colour\": \"#ff6a00\", \"glow\": {glow} }}\n    }}\n  ]\n}}\n"
        )
    };

    crate::qq::save(dir.path(), "level", &scene("0.7")).expect("saved");
    commit(dir.path(), "First light").expect("commit");
    assert!(
        read(dir.path()).expect("read").changes.is_empty(),
        "committed clean"
    );

    crate::qq::save(dir.path(), "level", &scene("0.25")).expect("saved again");
    let changed = read(dir.path()).expect("read");
    assert_eq!(
        changed
            .changes
            .iter()
            .map(|c| (c.path.as_str(), c.state.as_str()))
            .collect::<Vec<_>>(),
        vec![("scenes/level.qq.json", "modified")]
    );

    let diffs = diff(dir.path(), Some("scenes/level.qq.json")).expect("diff");
    let DiffBody::Text { lines, .. } = &diffs[0].body else {
        panic!("a scene is text, not binary: {:?}", diffs[0].body);
    };
    let removed: Vec<&str> = lines
        .iter()
        .filter(|l| l.kind == "remove")
        .map(|l| l.text.as_str())
        .collect();
    let added: Vec<&str> = lines
        .iter()
        .filter(|l| l.kind == "add")
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(removed.len(), 1, "one line out: {removed:?}");
    assert_eq!(added.len(), 1, "one line in: {added:?}");
    assert!(
        removed[0].ends_with("\"glow\": 0.7 }") && added[0].ends_with("\"glow\": 0.25 }"),
        "{removed:?} -> {added:?}"
    );
}
