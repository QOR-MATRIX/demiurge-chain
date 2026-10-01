//! The development dashboard: release-gate progress from `docs/GATES.toml`
//! (roadmap L2.2).
//!
//! # What this does, and nothing more
//!
//! `docs/GATES.toml` defines every criterion, the one signal that decides it and
//! exactly how progress is counted. This module applies those rules:
//!
//! - every unit is `met`, `not_met` or `unmeasurable`;
//! - roadmap, ADR, inventory and open-question criteria count once per listed
//!   item; every other criterion is one unit;
//! - a gate reports raw counts, and a bar only when none of its units is
//!   unmeasurable;
//! - a `check` is met only with evidence, and is labelled asserted;
//! - a gate is passed only when every unit is met.
//!
//! # Signals that cannot be read
//!
//! A unit is `unmeasurable` only where `GATES.toml` says so for its kind. When
//! any other signal cannot be read (`gh` is missing, a document is gone, a
//! criterion lists nothing), the unit is `not_met` and says why. Reading an
//! unreadable signal as unmeasurable would loosen the evidence rules, and that is
//! the owner's decision, not this module's.
//!
//! # Shape
//!
//! Gathering signals does the IO: documents, recorded test runs, `gh`, HTTP and
//! `cargo metadata`. [`evaluate`] is a pure function of the file, the documents
//! and the gathered signals, so the counting rules are tested without any IO.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::error::{QorError, QorResult};
use crate::{Confirm, Prompt};

/// Where the gates file lives, relative to the repository root.
pub const GATES_FILE: &str = "docs/GATES.toml";

/// Recorded test runs and downloaded reports, inside the launcher data directory.
const RUNS_DIR: &str = "gate-runs";

/// The branch whose CI run carries the coverage report.
const COVERAGE_BRANCH: &str = "main";

/// How long a CI reading is reused while the view polls.
const CI_TTL: Duration = Duration::from_secs(60);

/// How long a service reading is reused while the view polls.
const HTTP_TTL: Duration = Duration::from_secs(30);

// ─────────────────────────────── the file ─────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GatesFile {
    pub status: String,
    #[serde(default)]
    pub accepted: Option<String>,
    #[serde(default)]
    pub suites: HashMap<String, Suite>,
    #[serde(default)]
    pub ci: Option<CiSection>,
    #[serde(default)]
    pub gates: Vec<Gate>,
}

#[derive(Debug, Deserialize)]
pub struct CiSection {
    pub workflow: String,
    /// `owner/name` on GitHub. Named here so `gh` never guesses it from whichever
    /// remotes the clone happens to have.
    #[serde(default)]
    pub repository: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Suite {
    #[serde(default)]
    pub dir: String,
    pub command: String,
    #[serde(default)]
    pub needs: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Gate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub means: String,
    #[serde(default)]
    pub criteria: Vec<Criterion>,
}

/// A criterion. Which fields matter depends on `kind`.
#[derive(Debug, Deserialize)]
pub struct Criterion {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub says: Option<String>,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub pallets: Vec<String>,
    #[serde(default)]
    pub require: Vec<String>,
    #[serde(default)]
    pub suite: String,
    #[serde(default)]
    pub workflow: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub expect_status: Option<u16>,
    #[serde(default)]
    pub threshold: Option<f64>,
    #[serde(default)]
    pub gate: String,
}

// ─────────────────────────────── the report ───────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitState {
    Met,
    NotMet,
    Unmeasurable,
}

/// One countable unit, with the reason for its state.
#[derive(Debug, Clone, Serialize)]
pub struct Unit {
    pub label: String,
    pub state: UnitState,
    pub detail: String,
    /// A person asserted it (kind `check`); nothing computed it.
    pub asserted: bool,
}

impl Unit {
    fn new(label: impl Into<String>, state: UnitState, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            state,
            detail: detail.into(),
            asserted: false,
        }
    }

    fn met(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(label, UnitState::Met, detail)
    }

    fn not_met(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(label, UnitState::NotMet, detail)
    }

    fn unmeasurable(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(label, UnitState::Unmeasurable, detail)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CriterionReport {
    pub id: String,
    pub kind: String,
    pub says: Option<String>,
    pub units: Vec<Unit>,
}

/// Met and total units, present only when nothing in the gate is unmeasurable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Bar {
    pub met: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateReport {
    pub id: String,
    pub name: String,
    pub means: String,
    pub met: usize,
    pub not_met: usize,
    pub unmeasurable: usize,
    pub bar: Option<Bar>,
    pub passed: bool,
    pub criteria: Vec<CriterionReport>,
}

/// The most recent run of a suite, as the `tests` kind reads it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRun {
    pub suite: String,
    pub finished_at: String,
    pub commit: Option<String>,
    pub exit_code: Option<i32>,
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    /// The end of the output, so a failed run can be understood from the view.
    pub output_tail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuiteView {
    pub id: String,
    pub dir: String,
    pub command: String,
    pub needs: Option<String>,
    pub running: bool,
    pub last_run: Option<TestRun>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub repo: Option<String>,
    pub status: Option<String>,
    pub accepted: Option<String>,
    /// Anything that stops the file being read as intended, stated plainly.
    pub problems: Vec<String>,
    pub gates: Vec<GateReport>,
    pub suites: Vec<SuiteView>,
}

impl Report {
    fn problem(repo: Option<&Path>, problem: String) -> Self {
        Self {
            repo: repo.map(|p| p.display().to_string()),
            status: None,
            accepted: None,
            problems: vec![problem],
            gates: Vec::new(),
            suites: Vec::new(),
        }
    }
}

// ─────────────────────────────── the inputs ───────────────────────────────

/// The documents the kinds read. `None` means the file could not be read.
#[derive(Debug, Default)]
pub struct Docs {
    pub direction: Option<String>,
    pub adrs: Option<String>,
    pub inventory: Option<String>,
    pub open_questions: Option<String>,
}

/// The latest run of a workflow on a branch, and that branch's head.
#[derive(Debug, Clone)]
pub struct CiRun {
    pub id: u64,
    pub conclusion: String,
    pub status: String,
    pub head_sha: String,
    pub branch_head: String,
    pub url: String,
}

/// Signals gathered outside the documents.
#[derive(Debug, Default)]
pub struct Signals {
    pub runs: HashMap<String, TestRun>,
    pub ci: HashMap<(String, String), Result<CiRun, String>>,
    pub http: HashMap<String, Result<u16, String>>,
    /// Package names in the chain workspace, read only when pallets are listed.
    pub members: Option<Result<Vec<String>, String>>,
    /// Line coverage per pallet from CI, read only when pallets are listed.
    /// `Ok(None)`: no report exists for the branch head.
    pub coverage: Option<Result<Option<HashMap<String, f64>>, String>>,
}

pub fn read_docs(repo: &Path) -> Docs {
    let read = |path: &str| std::fs::read_to_string(repo.join(path)).ok();
    Docs {
        direction: read("docs/DIRECTION.md"),
        adrs: read("docs/decisions/README.md"),
        inventory: read("docs/architecture/MIGRATION_INVENTORY.md"),
        open_questions: read("docs/economics/OPEN_QUESTIONS.md"),
    }
}

// ─────────────────────────────── evaluation ───────────────────────────────

/// Apply the counting rules of `GATES.toml`. Pure: no IO.
pub fn evaluate(file: &GatesFile, docs: &Docs, signals: &Signals) -> Vec<GateReport> {
    let context = Context {
        file,
        signals,
        roadmap: docs.direction.as_deref().map(parse_roadmap),
        adrs: docs.adrs.as_deref().map(parse_adr_statuses),
        resolutions: docs.inventory.as_deref().map(parse_resolutions),
        questions: docs.open_questions.as_deref().map(parse_question_headings),
    };

    let mut reports: Vec<GateReport> = Vec::with_capacity(file.gates.len());
    for gate in &file.gates {
        let criteria = gate
            .criteria
            .iter()
            .map(|criterion| CriterionReport {
                id: criterion.id.clone(),
                kind: criterion.kind.clone(),
                says: criterion.says.clone(),
                units: context.units(criterion, &reports),
            })
            .collect();
        reports.push(summarise(gate, criteria));
    }
    reports
}

struct Context<'a> {
    file: &'a GatesFile,
    signals: &'a Signals,
    roadmap: Option<HashMap<String, bool>>,
    adrs: Option<HashMap<String, String>>,
    resolutions: Option<HashMap<String, String>>,
    questions: Option<HashSet<String>>,
}

impl Context<'_> {
    fn units(&self, c: &Criterion, earlier: &[GateReport]) -> Vec<Unit> {
        match c.kind.as_str() {
            "check" => vec![check_unit(c)],
            "roadmap" => per_item(c, &c.items, |item| self.roadmap_unit(item)),
            "adr" => per_item(c, &c.adrs, |adr| self.adr_unit(adr)),
            "inventory" => per_item(c, &c.questions, |q| self.inventory_unit(q)),
            "open_question" => per_item(c, &c.questions, |q| self.open_question_unit(q)),
            "tests" => vec![self.tests_unit(c)],
            "ci" => vec![self.ci_unit(c)],
            "http" => vec![self.http_unit(c)],
            "pallet" => vec![self.pallet_unit(c)],
            "coverage" => vec![self.coverage_unit(c)],
            "gate" => vec![gate_unit(c, earlier)],
            other => vec![Unit::not_met(
                &c.id,
                format!("unknown kind \"{other}\": this launcher cannot read it"),
            )],
        }
    }

    fn roadmap_unit(&self, item: &str) -> Unit {
        match &self.roadmap {
            None => Unit::not_met(item, "docs/DIRECTION.md could not be read"),
            Some(map) => match map.get(item) {
                Some(true) => Unit::met(item, "checked in docs/DIRECTION.md"),
                Some(false) => Unit::not_met(item, "unchecked in docs/DIRECTION.md"),
                None => Unit::unmeasurable(item, "docs/DIRECTION.md has no checkbox for this item"),
            },
        }
    }

    fn adr_unit(&self, adr: &str) -> Unit {
        match &self.adrs {
            None => Unit::not_met(adr, "docs/decisions/README.md could not be read"),
            Some(map) => match map.get(adr) {
                Some(status) if status.starts_with("Accepted") => Unit::met(adr, status.as_str()),
                Some(status) => Unit::not_met(adr, status.as_str()),
                None => Unit::not_met(adr, "not listed in docs/decisions/README.md"),
            },
        }
    }

    fn inventory_unit(&self, question: &str) -> Unit {
        let Some(rows) = &self.resolutions else {
            return Unit::not_met(
                question,
                "docs/architecture/MIGRATION_INVENTORY.md could not be read",
            );
        };
        let Some(status) = rows.get(question) else {
            return Unit::not_met(
                question,
                "no row in the inventory's Resolution status table",
            );
        };
        let Some(adr) = resolved_by(status) else {
            return Unit::not_met(question, status.as_str());
        };
        match self.adrs.as_ref().and_then(|adrs| adrs.get(&adr)) {
            Some(adr_status) if adr_status.starts_with("Accepted") => {
                Unit::met(question, format!("{status}, which is Accepted"))
            }
            Some(adr_status) => {
                Unit::not_met(question, format!("{status}, but {adr} is \"{adr_status}\""))
            }
            None => Unit::not_met(
                question,
                format!("{status}, but {adr} is not in docs/decisions/README.md"),
            ),
        }
    }

    fn open_question_unit(&self, question: &str) -> Unit {
        match &self.questions {
            None => Unit::not_met(
                question,
                "docs/economics/OPEN_QUESTIONS.md could not be read",
            ),
            Some(open) if open.contains(question) => {
                Unit::not_met(question, "still listed in docs/economics/OPEN_QUESTIONS.md")
            }
            Some(_) => Unit::met(
                question,
                "no longer listed in docs/economics/OPEN_QUESTIONS.md",
            ),
        }
    }

    fn tests_unit(&self, c: &Criterion) -> Unit {
        let label = format!("suite {}", c.suite);
        let Some(suite) = self.file.suites.get(&c.suite) else {
            return Unit::not_met(label, "the suite is not defined in docs/GATES.toml");
        };
        if suite.dir.trim().is_empty() {
            return Unit::unmeasurable(label, "the suite has no directory yet");
        }
        let Some(run) = self.signals.runs.get(&c.suite) else {
            return Unit::unmeasurable(label, "never run: run it from this view");
        };

        let when = describe_run(run);
        if run.exit_code != Some(0) {
            let code = run.exit_code.map_or_else(
                || "without an exit code".to_string(),
                |code| format!("with {code}"),
            );
            return Unit::not_met(label, format!("the run {when} exited {code}"));
        }
        if !run.failed.is_empty() {
            return Unit::not_met(
                label,
                format!(
                    "the run {when} reported failures: {}",
                    run.failed.join(", ")
                ),
            );
        }
        let missing: Vec<&str> = c
            .require
            .iter()
            .filter(|name| !run_passed(run, name))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() {
            return Unit::not_met(
                label,
                format!(
                    "required tests did not pass in the run {when}: {}",
                    missing.join(", ")
                ),
            );
        }
        Unit::met(
            label,
            format!("the run {when}: {} passed", run.passed.len()),
        )
    }

    fn ci_unit(&self, c: &Criterion) -> Unit {
        let label = format!("{} on {}", c.workflow, c.branch);
        match self.signals.ci.get(&(c.workflow.clone(), c.branch.clone())) {
            None => Unit::not_met(label, "not checked yet"),
            Some(Err(error)) => Unit::not_met(label, format!("could not be read: {error}")),
            Some(Ok(run)) => {
                let outcome = if run.conclusion.is_empty() {
                    run.status.as_str()
                } else {
                    run.conclusion.as_str()
                };
                if run.head_sha != run.branch_head {
                    Unit::not_met(
                        label,
                        format!(
                            "the latest run ({outcome}) is for {}, not the branch head {}",
                            short(&run.head_sha),
                            short(&run.branch_head)
                        ),
                    )
                } else if run.conclusion == "success" {
                    Unit::met(
                        label,
                        format!("succeeded on {} ({})", short(&run.head_sha), run.url),
                    )
                } else {
                    Unit::not_met(
                        label,
                        format!("the latest run on the branch head: {outcome} ({})", run.url),
                    )
                }
            }
        }
    }

    fn http_unit(&self, c: &Criterion) -> Unit {
        let url = c.url.trim();
        if url.is_empty() {
            return Unit::unmeasurable(&c.id, "no URL: the service has not been deployed");
        }
        let Some(expect) = c.expect_status else {
            return Unit::not_met(url, "the criterion sets no expect_status");
        };
        match self.signals.http.get(url) {
            None => Unit::not_met(url, "not checked yet"),
            Some(Err(error)) => Unit::not_met(url, format!("no response: {error}")),
            Some(Ok(code)) if *code == expect => Unit::met(url, format!("answered {code}")),
            Some(Ok(code)) => Unit::not_met(url, format!("answered {code}, expected {expect}")),
        }
    }

    fn pallet_unit(&self, c: &Criterion) -> Unit {
        let chain_dir = self.file.suites.get("chain").map_or("", |s| s.dir.trim());
        if c.pallets.is_empty() || chain_dir.is_empty() {
            return Unit::unmeasurable(
                &c.id,
                "no pallets are listed, or the chain has no location yet (inventory Q-8, Q-15)",
            );
        }
        let members = match &self.signals.members {
            None => return Unit::not_met(&c.id, "the chain workspace has not been read"),
            Some(Err(error)) => {
                return Unit::not_met(
                    &c.id,
                    format!("the chain workspace could not be read: {error}"),
                )
            }
            Some(Ok(members)) => members,
        };
        let absent: Vec<&str> = c
            .pallets
            .iter()
            .filter(|pallet| !members.contains(pallet))
            .map(String::as_str)
            .collect();
        if !absent.is_empty() {
            return Unit::not_met(
                &c.id,
                format!("not in the chain workspace: {}", absent.join(", ")),
            );
        }
        let Some(run) = self.signals.runs.get("chain") else {
            return Unit::not_met(&c.id, "the chain suite has not been run");
        };
        let missing: Vec<&str> = c
            .require
            .iter()
            .filter(|name| !run_passed(run, name))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() {
            return Unit::not_met(
                &c.id,
                format!("required tests did not pass: {}", missing.join(", ")),
            );
        }
        Unit::met(
            &c.id,
            format!(
                "all {} pallets present; required tests passed in the run {}",
                c.pallets.len(),
                describe_run(run)
            ),
        )
    }

    fn coverage_unit(&self, c: &Criterion) -> Unit {
        if c.pallets.is_empty() {
            return Unit::unmeasurable(&c.id, "no pallets are listed yet (inventory Q-8, Q-15)");
        }
        let Some(threshold) = c.threshold else {
            return Unit::not_met(&c.id, "the criterion sets no threshold");
        };
        match &self.signals.coverage {
            None => Unit::not_met(&c.id, "not checked yet"),
            Some(Err(error)) => Unit::not_met(&c.id, format!("could not be read: {error}")),
            Some(Ok(None)) => Unit::unmeasurable(
                &c.id,
                "CI has not produced a coverage report for the branch head",
            ),
            Some(Ok(Some(report))) => {
                let short_of: Vec<String> = c
                    .pallets
                    .iter()
                    .filter_map(|pallet| match report.get(pallet) {
                        Some(percent) if *percent >= threshold => None,
                        Some(percent) => Some(format!("{pallet} at {percent}%")),
                        None => Some(format!("{pallet} missing from the report")),
                    })
                    .collect();
                if short_of.is_empty() {
                    Unit::met(
                        &c.id,
                        format!("every listed pallet is at or above {threshold}%"),
                    )
                } else {
                    Unit::not_met(
                        &c.id,
                        format!("below {threshold}%: {}", short_of.join(", ")),
                    )
                }
            }
        }
    }
}

fn per_item(c: &Criterion, list: &[String], unit: impl Fn(&str) -> Unit) -> Vec<Unit> {
    if list.is_empty() {
        return vec![Unit::not_met(
            &c.id,
            "lists no items, so it can never be met",
        )];
    }
    list.iter().map(|item| unit(item)).collect()
}

fn check_unit(c: &Criterion) -> Unit {
    let evidence = c.evidence.trim();
    let mut unit = if c.done && !evidence.is_empty() {
        Unit::met(&c.id, format!("evidence: {evidence}"))
    } else if c.done {
        Unit::not_met(&c.id, "marked done, but no evidence is recorded")
    } else {
        Unit::not_met(&c.id, "not done")
    };
    unit.asserted = true;
    unit
}

fn gate_unit(c: &Criterion, earlier: &[GateReport]) -> Unit {
    match earlier.iter().find(|gate| gate.id == c.gate) {
        Some(gate) if gate.passed => Unit::met(&c.gate, format!("{} is passed", gate.name)),
        Some(gate) => Unit::not_met(
            &c.gate,
            format!(
                "{} is not passed: {} met, {} not met, {} unmeasurable",
                gate.name, gate.met, gate.not_met, gate.unmeasurable
            ),
        ),
        None => Unit::not_met(&c.gate, "no earlier gate has this id"),
    }
}

fn summarise(gate: &Gate, criteria: Vec<CriterionReport>) -> GateReport {
    let (mut met, mut not_met, mut unmeasurable) = (0, 0, 0);
    for unit in criteria.iter().flat_map(|c| &c.units) {
        match unit.state {
            UnitState::Met => met += 1,
            UnitState::NotMet => not_met += 1,
            UnitState::Unmeasurable => unmeasurable += 1,
        }
    }
    let total = met + not_met + unmeasurable;

    GateReport {
        id: gate.id.clone(),
        name: gate.name.clone(),
        means: gate.means.clone(),
        met,
        not_met,
        unmeasurable,
        bar: (unmeasurable == 0 && total > 0).then_some(Bar { met, total }),
        passed: total > 0 && met == total,
        criteria,
    }
}

fn run_passed(run: &TestRun, name: &str) -> bool {
    run.passed
        .iter()
        .any(|passed| passed == name || passed.rsplit("::").next() == Some(name))
}

fn describe_run(run: &TestRun) -> String {
    match &run.commit {
        Some(commit) => format!("at {} on {commit}", run.finished_at),
        None => format!("at {}", run.finished_at),
    }
}

fn short(sha: &str) -> &str {
    sha.get(..10).unwrap_or(sha)
}

// ─────────────────────────────── the parsers ──────────────────────────────

/// Checkbox items from `docs/DIRECTION.md`, keyed as `GATES.toml` names them:
/// `M3.2` under `### M3:`, `L1.3` under `#### L1:`, `P1.2` under `#### P1:` (a
/// product track, ADR-050), `7.1.5` under `### 7.1`.
/// Only unindented `n. [ ]` and `n. [x]` lines are read.
pub fn parse_roadmap(text: &str) -> HashMap<String, bool> {
    let mut section: Option<String> = None;
    let mut items = HashMap::new();

    for line in text.lines() {
        if line.starts_with('#') {
            section = section_key(line);
            continue;
        }
        let Some(key) = &section else { continue };
        let Some((number, rest)) = line.split_once(". ") else {
            continue;
        };
        if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let checked = if rest.starts_with("[x]") {
            true
        } else if rest.starts_with("[ ]") {
            false
        } else {
            continue;
        };
        items.insert(format!("{key}.{number}"), checked);
    }
    items
}

fn section_key(heading: &str) -> Option<String> {
    let title = heading.trim_start_matches('#');
    if heading.len() - title.len() < 2 {
        return None;
    }
    let token = title
        .trim_start()
        .split(|c: char| c == ':' || c.is_whitespace())
        .next()?;

    let milestone = |prefix: char| {
        token.len() > 1
            && token.starts_with(prefix)
            && token[1..].bytes().all(|b| b.is_ascii_digit())
    };
    if milestone('M') || milestone('L') || milestone('P') {
        return Some(token.to_string());
    }

    let parts: Vec<&str> = token.split('.').collect();
    let numbered = parts.len() == 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    numbered.then(|| token.to_string())
}

/// ADR id to its Status cell, from the table in `docs/decisions/README.md`.
pub fn parse_adr_statuses(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(table_cells)
        .filter_map(|cells| {
            let id = find_adr_id(cells.first()?)?;
            Some((id, cells.last()?.to_string()))
        })
        .collect()
}

/// Question id to its Status cell, from the inventory's "Resolution status" table.
pub fn parse_resolutions(text: &str) -> HashMap<String, String> {
    let mut inside = false;
    let mut rows = HashMap::new();

    for line in text.lines() {
        if line.starts_with('#') {
            inside = line.trim_start_matches('#').trim() == "Resolution status";
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(cells) = table_cells(line) {
            if let (Some(id), Some(status)) = (cells.first(), cells.last()) {
                if id.starts_with("Q-") {
                    rows.insert((*id).to_string(), (*status).to_string());
                }
            }
        }
    }
    rows
}

/// `OPEN-n` and `U-n` headings present in `docs/economics/OPEN_QUESTIONS.md`.
pub fn parse_question_headings(text: &str) -> HashSet<String> {
    text.lines()
        .filter(|line| line.starts_with('#'))
        .filter_map(|line| {
            let token = line
                .trim_start_matches('#')
                .trim_start()
                .split(|c: char| c == ':' || c.is_whitespace())
                .next()?;
            let (prefix, number) = token.split_once('-')?;
            let is_question = (prefix == "OPEN" || prefix == "U")
                && !number.is_empty()
                && number.bytes().all(|b| b.is_ascii_digit());
            is_question.then(|| token.to_string())
        })
        .collect()
}

/// Passed and failed test names from `cargo test` output.
pub fn parse_test_output(output: &str) -> (Vec<String>, Vec<String>) {
    let mut passed = Vec::new();
    let mut failed = Vec::new();

    for line in output.lines() {
        let Some(rest) = line.trim().strip_prefix("test ") else {
            continue;
        };
        let Some((name, outcome)) = rest.rsplit_once(" ... ") else {
            continue;
        };
        let outcome = outcome.trim();
        if outcome == "ok" {
            passed.push(name.to_string());
        } else if outcome.starts_with("FAILED") {
            failed.push(name.to_string());
        }
    }
    (passed, failed)
}

fn table_cells(line: &str) -> Option<Vec<&str>> {
    let inner = line.trim().strip_prefix('|')?.strip_suffix('|')?;
    let cells: Vec<&str> = inner.split('|').map(str::trim).collect();
    let separator = cells
        .iter()
        .all(|cell| cell.chars().all(|c| c == '-' || c == ':' || c == ' '));
    (!separator).then_some(cells)
}

fn find_adr_id(text: &str) -> Option<String> {
    let start = text.find("ADR-")?;
    let digits: String = text[start + 4..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    (!digits.is_empty()).then(|| format!("ADR-{digits}"))
}

fn resolved_by(status: &str) -> Option<String> {
    status.strip_prefix("Resolved by ").and_then(find_adr_id)
}

// ────────────────────────────── gathering (IO) ────────────────────────────

/// Readings reused for a short time, with when each was taken.
type Cache<K, V> = Mutex<HashMap<K, (Instant, V)>>;

/// Reads the gates file and its signals for the view, and runs suites on request.
pub struct Dashboard {
    repo: Option<PathBuf>,
    http: reqwest::Client,
    ci_cache: Cache<(String, String), Result<CiRun, String>>,
    http_cache: Cache<String, Result<u16, String>>,
    running: Mutex<HashSet<String>>,
}

impl Dashboard {
    /// A dashboard over the repository [`locate_repo`] finds.
    pub fn new() -> QorResult<Self> {
        Self::with_repo(locate_repo())
    }

    /// A dashboard over `repo`. `None` reports that no repository was found.
    pub fn with_repo(repo: Option<PathBuf>) -> QorResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(concat!("QorLauncher/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| QorError::Internal(format!("cannot build http client: {e}")))?;

        Ok(Self {
            repo,
            http,
            ci_cache: Mutex::new(HashMap::new()),
            http_cache: Mutex::new(HashMap::new()),
            running: Mutex::new(HashSet::new()),
        })
    }

    /// Read everything and evaluate. Never fails: problems are part of the report.
    pub async fn report(&self, data_dir: &Path, refresh: bool) -> Report {
        let Some(repo) = self.repo.as_deref() else {
            return Report::problem(
                None,
                format!(
                    "No Demiurge repository was found, so there is nothing to measure. Start the \
                     launcher from inside the repository, or set DEMIURGE_REPO to its root (the \
                     directory containing {GATES_FILE})."
                ),
            );
        };
        let file = match load_file(repo) {
            Ok(file) => file,
            Err(problem) => return Report::problem(Some(repo), problem),
        };

        let docs = read_docs(repo);
        let signals = self.gather(repo, data_dir, &file, refresh).await;
        let gates = evaluate(&file, &docs, &signals);

        let running = self.running.lock().clone();
        let mut suites: Vec<SuiteView> = file
            .suites
            .iter()
            .map(|(id, suite)| SuiteView {
                id: id.clone(),
                dir: suite.dir.clone(),
                command: suite.command.clone(),
                needs: suite.needs.clone(),
                running: running.contains(id),
                last_run: signals.runs.get(id).cloned(),
            })
            .collect();
        suites.sort_by(|a, b| a.id.cmp(&b.id));

        let mut problems = Vec::new();
        if file.status != "accepted" {
            problems.push(format!(
                "docs/GATES.toml has status \"{}\", not \"accepted\"; its criteria are not yet the owner's.",
                file.status
            ));
        }

        Report {
            repo: Some(repo.display().to_string()),
            status: Some(file.status.clone()),
            accepted: file.accepted.clone(),
            problems,
            gates,
            suites,
        }
    }

    async fn gather(
        &self,
        repo: &Path,
        data_dir: &Path,
        file: &GatesFile,
        refresh: bool,
    ) -> Signals {
        let mut signals = Signals::default();

        for id in file.suites.keys() {
            if let Some(run) = load_run(data_dir, id) {
                signals.runs.insert(id.clone(), run);
            }
        }

        let criteria: Vec<&Criterion> = file.gates.iter().flat_map(|g| &g.criteria).collect();

        let ci_keys: HashSet<(String, String)> = criteria
            .iter()
            .filter(|c| c.kind == "ci")
            .map(|c| (c.workflow.clone(), c.branch.clone()))
            .collect();
        for key in ci_keys {
            let reading = self.ci(repo, file, &key, refresh).await;
            signals.ci.insert(key, reading);
        }

        let urls: HashSet<String> = criteria
            .iter()
            .filter(|c| c.kind == "http" && !c.url.trim().is_empty())
            .map(|c| c.url.trim().to_string())
            .collect();
        for url in urls {
            let reading = self.http_status(&url, refresh).await;
            signals.http.insert(url, reading);
        }

        let chain_dir = file.suites.get("chain").map_or("", |s| s.dir.trim());
        let pallets_listed = criteria
            .iter()
            .any(|c| c.kind == "pallet" && !c.pallets.is_empty());
        if pallets_listed && !chain_dir.is_empty() {
            signals.members = Some(workspace_members(&repo.join(chain_dir)).await);
        }

        if criteria
            .iter()
            .any(|c| c.kind == "coverage" && !c.pallets.is_empty())
        {
            signals.coverage = Some(self.coverage(repo, data_dir, file, refresh).await);
        }

        signals
    }

    async fn ci(
        &self,
        repo: &Path,
        file: &GatesFile,
        key: &(String, String),
        refresh: bool,
    ) -> Result<CiRun, String> {
        if !refresh {
            if let Some(reading) = cached(&self.ci_cache, key, CI_TTL) {
                return reading;
            }
        }
        let reading = match ci_repository(file) {
            Ok(repository) => read_ci(repo, repository, &key.0, &key.1).await,
            Err(error) => Err(error),
        };
        self.ci_cache
            .lock()
            .insert(key.clone(), (Instant::now(), reading.clone()));
        reading
    }

    async fn http_status(&self, url: &str, refresh: bool) -> Result<u16, String> {
        let key = url.to_string();
        if !refresh {
            if let Some(reading) = cached(&self.http_cache, &key, HTTP_TTL) {
                return reading;
            }
        }
        let reading = self
            .http
            .get(url)
            .send()
            .await
            .map(|response| response.status().as_u16())
            .map_err(|e| e.to_string());
        self.http_cache
            .lock()
            .insert(key, (Instant::now(), reading.clone()));
        reading
    }

    async fn coverage(
        &self,
        repo: &Path,
        data_dir: &Path,
        file: &GatesFile,
        refresh: bool,
    ) -> Result<Option<HashMap<String, f64>>, String> {
        let workflow = file
            .ci
            .as_ref()
            .map(|ci| ci.workflow.clone())
            .ok_or("docs/GATES.toml names no [ci] workflow")?;
        let run = self
            .ci(
                repo,
                file,
                &(workflow, COVERAGE_BRANCH.to_string()),
                refresh,
            )
            .await?;
        if run.head_sha != run.branch_head {
            return Ok(None);
        }

        let dir = data_dir.join(RUNS_DIR).join(format!("coverage-{}", run.id));
        if !dir.is_dir() {
            let id = run.id.to_string();
            let target = dir.to_string_lossy().into_owned();
            let download = run_tool(
                repo,
                "gh",
                &[
                    "run",
                    "download",
                    &id,
                    "-R",
                    ci_repository(file)?,
                    "-n",
                    "coverage",
                    "-D",
                    &target,
                ],
            )
            .await;
            match download {
                Ok(_) => {}
                Err(error)
                    if error.contains("no valid artifacts") || error.contains("no artifact") =>
                {
                    return Ok(None)
                }
                Err(error) => return Err(error),
            }
        }

        let mut report = HashMap::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(pallet) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            if let Some(percent) = value
                .pointer("/data/0/totals/lines/percent")
                .and_then(serde_json::Value::as_f64)
            {
                report.insert(pallet.to_string(), percent);
            }
        }
        Ok(Some(report))
    }

    /// Run a suite `docs/GATES.toml` defines and record the result.
    ///
    /// The webview only names the suite. The directory and command come from the
    /// file, and nothing runs through a shell. A suite builds and runs code from
    /// the repository, so it starts only once the person at the keyboard approves
    /// it in `confirm`, the same host dialog that guards signatures (L1.4).
    pub async fn run_suite(
        &self,
        data_dir: &Path,
        suite_id: &str,
        confirm: &dyn Confirm,
    ) -> QorResult<TestRun> {
        let repo = self
            .repo
            .as_deref()
            .ok_or_else(|| QorError::Internal("no Demiurge repository was found".into()))?;
        let file = load_file(repo).map_err(QorError::Internal)?;
        let suite = file.suites.get(suite_id).cloned().ok_or_else(|| {
            QorError::Internal(format!("docs/GATES.toml defines no suite \"{suite_id}\""))
        })?;
        if suite.dir.trim().is_empty() {
            return Err(QorError::Internal(format!(
                "suite \"{suite_id}\" has no directory yet"
            )));
        }
        if run_file(data_dir, suite_id).is_none() {
            return Err(QorError::Internal(format!(
                "\"{suite_id}\" is not a usable suite name"
            )));
        }
        if self.running.lock().contains(suite_id) {
            return Err(QorError::Internal(format!(
                "suite \"{suite_id}\" is already running"
            )));
        }

        let prompt = Prompt {
            title: "Run a test suite?".into(),
            body: format!(
                "Suite: {suite_id}\nDirectory: {}\nCommand: {}\n\nThis builds and runs code from that \
                 directory on this machine, with your permissions. The directory and the command come \
                 from {GATES_FILE}.\n\nApprove only if you trust the repository as it is now.",
                repo.join(suite.dir.trim()).display(),
                suite.command
            ),
            approve: "Run".into(),
        };
        if !confirm.confirm(&prompt).await {
            return Err(QorError::Declined);
        }

        if !self.running.lock().insert(suite_id.to_string()) {
            return Err(QorError::Internal(format!(
                "suite \"{suite_id}\" is already running"
            )));
        }

        let result = execute_suite(repo, data_dir, suite_id, &suite).await;
        self.running.lock().remove(suite_id);

        let run = result?;
        save_run(data_dir, &run)?;
        Ok(run)
    }
}

fn cached<K: Eq + std::hash::Hash, V: Clone>(
    cache: &Mutex<HashMap<K, (Instant, V)>>,
    key: &K,
    ttl: Duration,
) -> Option<V> {
    cache
        .lock()
        .get(key)
        .filter(|(at, _)| at.elapsed() < ttl)
        .map(|(_, value)| value.clone())
}

/// The repository root: `DEMIURGE_REPO`, or the nearest ancestor of the working
/// directory or the executable that contains `docs/GATES.toml`.
pub fn locate_repo() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("DEMIURGE_REPO") {
        let dir = PathBuf::from(dir);
        if dir.join(GATES_FILE).is_file() {
            return Some(dir);
        }
        tracing::warn!(dir = %dir.display(), "DEMIURGE_REPO does not contain {GATES_FILE}");
    }

    [std::env::current_dir().ok(), std::env::current_exe().ok()]
        .into_iter()
        .flatten()
        .find_map(|start| {
            start
                .ancestors()
                .find(|dir| dir.join(GATES_FILE).is_file())
                .map(Path::to_path_buf)
        })
}

fn load_file(repo: &Path) -> Result<GatesFile, String> {
    let text = std::fs::read_to_string(repo.join(GATES_FILE))
        .map_err(|e| format!("{GATES_FILE} could not be read: {e}"))?;
    toml::from_str(&text).map_err(|e| format!("{GATES_FILE} does not parse: {e}"))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhRun {
    database_id: u64,
    #[serde(default)]
    conclusion: String,
    #[serde(default)]
    status: String,
    head_sha: String,
    #[serde(default)]
    url: String,
}

/// The repository `[ci]` names. Without it `gh` would pick one of the clone's remotes
/// itself, and with a private archive beside the public repository it picks the archive.
fn ci_repository(file: &GatesFile) -> Result<&str, String> {
    file.ci
        .as_ref()
        .map(|ci| ci.repository.trim())
        .filter(|r| !r.is_empty())
        .ok_or_else(|| "docs/GATES.toml names no [ci] repository".to_string())
}

async fn read_ci(
    repo: &Path,
    repository: &str,
    workflow: &str,
    branch: &str,
) -> Result<CiRun, String> {
    let listing = run_tool(
        repo,
        "gh",
        &[
            "run",
            "list",
            "-R",
            repository,
            "--workflow",
            workflow,
            "--branch",
            branch,
            "--limit",
            "1",
            "--json",
            "databaseId,conclusion,status,headSha,url",
        ],
    )
    .await?;
    let runs: Vec<GhRun> =
        serde_json::from_str(&listing).map_err(|e| format!("unexpected output from gh: {e}"))?;
    let run = runs
        .into_iter()
        .next()
        .ok_or_else(|| format!("no run of \"{workflow}\" on {branch}"))?;

    let head = run_tool(
        repo,
        "gh",
        &[
            "api",
            &format!("repos/{repository}/branches/{branch}"),
            "--jq",
            ".commit.sha",
        ],
    )
    .await?;

    Ok(CiRun {
        id: run.database_id,
        conclusion: run.conclusion,
        status: run.status,
        head_sha: run.head_sha,
        branch_head: head.trim().to_string(),
        url: run.url,
    })
}

async fn workspace_members(dir: &Path) -> Result<Vec<String>, String> {
    let json = run_tool(
        dir,
        "cargo",
        &["metadata", "--no-deps", "--format-version", "1"],
    )
    .await?;
    let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(value["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .filter_map(|p| p["name"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default())
}

async fn run_tool(dir: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let mut command = tokio::process::Command::new(program);
    command.args(args).current_dir(dir).kill_on_drop(true);
    no_console_window(&mut command);

    let output = command
        .output()
        .await
        .map_err(|e| format!("could not run {program}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {} failed: {}",
            args.first().copied().unwrap_or_default(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn execute_suite(
    repo: &Path,
    data_dir: &Path,
    id: &str,
    suite: &Suite,
) -> QorResult<TestRun> {
    let mut words = suite.command.split_whitespace();
    let program = words
        .next()
        .ok_or_else(|| QorError::Internal(format!("suite \"{id}\" has an empty command")))?;

    let mut command = tokio::process::Command::new(program);
    command
        .args(words)
        .current_dir(repo.join(suite.dir.trim()))
        .kill_on_drop(true);
    if program == "cargo" {
        // A separate target directory. A running launcher holds its own binary
        // open, and a dashboard run should not replace a developer's build.
        command.env(
            "CARGO_TARGET_DIR",
            data_dir.join(RUNS_DIR).join(format!("target-{id}")),
        );
    }
    no_console_window(&mut command);

    let output = command
        .output()
        .await
        .map_err(|e| QorError::Io(format!("could not start `{}`: {e}", suite.command)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let (passed, failed) = parse_test_output(&stdout);
    let commit = run_tool(repo, "git", &["rev-parse", "--short", "HEAD"])
        .await
        .ok()
        .map(|sha| sha.trim().to_string());

    Ok(TestRun {
        suite: id.to_string(),
        finished_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        commit,
        exit_code: output.status.code(),
        passed,
        failed,
        output_tail: tail(&format!("{stdout}\n{stderr}"), 40),
    })
}

fn tail(text: &str, lines: usize) -> String {
    let all: Vec<&str> = text.trim_end().lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

fn run_file(data_dir: &Path, id: &str) -> Option<PathBuf> {
    let safe = !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    safe.then(|| data_dir.join(RUNS_DIR).join(format!("{id}.json")))
}

fn load_run(data_dir: &Path, id: &str) -> Option<TestRun> {
    let raw = std::fs::read_to_string(run_file(data_dir, id)?).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save_run(data_dir: &Path, run: &TestRun) -> QorResult<()> {
    let path = run_file(data_dir, &run.suite).ok_or_else(|| {
        QorError::Internal(format!("\"{}\" is not a usable suite name", run.suite))
    })?;
    std::fs::create_dir_all(data_dir.join(RUNS_DIR))?;
    let json = serde_json::to_vec_pretty(run)
        .map_err(|e| QorError::Internal(format!("cannot serialise the test run: {e}")))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// A launcher started from the desktop has no console; without this, every tool
/// it runs would flash a console window on Windows.
#[cfg(windows)]
fn no_console_window(command: &mut tokio::process::Command) {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn no_console_window(_command: &mut tokio::process::Command) {}

#[cfg(test)]
mod tests {
    use super::*;
    use UnitState::{Met, NotMet, Unmeasurable};

    fn file(text: &str) -> GatesFile {
        toml::from_str(text).expect("the fixture parses")
    }

    /// A file with one gate holding one criterion, plus two suites.
    fn one(criterion: &str) -> String {
        format!(
            "status = \"accepted\"\n\n\
             [suites.unit]\ndir = \"somewhere\"\ncommand = \"cargo test\"\n\n\
             [suites.nowhere]\ndir = \"\"\ncommand = \"cargo test\"\n\n\
             [[gates]]\nid = \"g\"\nname = \"G\"\n\n\
             [[gates.criteria]]\nid = \"c\"\n{criterion}\n"
        )
    }

    fn units(criterion: &str, docs: &Docs, signals: &Signals) -> Vec<Unit> {
        let mut gate = evaluate(&file(&one(criterion)), docs, signals).remove(0);
        gate.criteria.remove(0).units
    }

    fn states(units: &[Unit]) -> Vec<UnitState> {
        units.iter().map(|u| u.state).collect()
    }

    const DIRECTION: &str = "## 7. Roadmap\n\n\
        ### 7.1 Security track\n\n1. [x] Done.\n2. [ ] Open\n   and continued.\n\n\
        ### M2: Decisions\n\n1. [x] Reviewed.\n\n\
        #### L1: Launcher\n\n3. [x] Amounts.\n\n\
        ### The launcher track\n\n4. [x] Under no milestone, so not readable.\n";

    #[test]
    fn roadmap_items_come_from_checkbox_lines_under_their_heading() {
        let items = parse_roadmap(DIRECTION);
        assert_eq!(items.get("7.1.1"), Some(&true));
        assert_eq!(items.get("7.1.2"), Some(&false));
        assert_eq!(items.get("M2.1"), Some(&true));
        assert_eq!(items.get("L1.3"), Some(&true));
        assert_eq!(items.len(), 4, "{items:?}");
    }

    /// The product tracks (ADR-050): `P1.2` under `#### P1:`. Without them every
    /// product item would read as having no checkbox — unmeasurable — while the
    /// checkbox sat in DIRECTION.md in plain sight.
    #[test]
    fn product_track_items_are_read_under_their_heading() {
        let items = parse_roadmap(
            "### The product tracks\n\n\
             #### P1: Qontrol\n\n1. [x] Projects.\n2. [ ] The git layer.\n\n\
             #### P6: Stream (placeholder name)\n\n1. [ ] A decision.\n\n\
             #### Px: not a track\n\n1. [x] Not readable.\n",
        );
        assert_eq!(items.get("P1.1"), Some(&true));
        assert_eq!(items.get("P1.2"), Some(&false));
        assert_eq!(items.get("P6.1"), Some(&false));
        assert_eq!(items.len(), 3, "{items:?}");
    }

    #[test]
    fn a_roadmap_criterion_counts_once_per_item() {
        let docs = Docs {
            direction: Some(DIRECTION.into()),
            ..Docs::default()
        };
        let signals = Signals::default();

        let read = units(
            "kind = \"roadmap\"\nitems = [\"M2.1\", \"7.1.2\", \"M9.1\"]",
            &docs,
            &signals,
        );
        assert_eq!(states(&read), [Met, NotMet, Unmeasurable]);

        let unread = units(
            "kind = \"roadmap\"\nitems = [\"M2.1\"]",
            &Docs::default(),
            &signals,
        );
        assert_eq!(
            states(&unread),
            [NotMet],
            "an unreadable document is neither met nor unmeasurable"
        );

        let empty = units("kind = \"roadmap\"\nitems = []", &docs, &signals);
        assert_eq!(
            states(&empty),
            [NotMet],
            "a criterion listing nothing cannot vanish from the count"
        );
    }

    #[test]
    fn a_check_is_met_only_with_evidence_and_is_labelled_asserted() {
        let signals = Signals::default();
        let with = units(
            "kind = \"check\"\ndone = true\nevidence = \"abc1234\"",
            &Docs::default(),
            &signals,
        );
        assert_eq!(states(&with), [Met]);
        assert!(with[0].asserted);

        let without = units(
            "kind = \"check\"\ndone = true\nevidence = \" \"",
            &Docs::default(),
            &signals,
        );
        assert_eq!(states(&without), [NotMet]);
        assert!(without[0].asserted);
    }

    #[test]
    fn decisions_and_resolutions_must_be_accepted() {
        let adrs = "| ADR | Title | Status |\n| --- | --- | --- |\n\
                    | [ADR-013](a.md) | A | Accepted; with a note |\n\
                    | [ADR-016](b.md) | B | Proposed |\n";
        let inventory = "### Resolution status\n\n| # | Status |\n| --- | --- |\n\
                         | Q-1 | Resolved by ADR-013 |\n| Q-2 | Resolved by ADR-016 |\n| Q-3 | Open |\n";
        let docs = Docs {
            adrs: Some(adrs.into()),
            inventory: Some(inventory.into()),
            ..Docs::default()
        };
        let signals = Signals::default();

        let adr = units(
            "kind = \"adr\"\nadrs = [\"ADR-013\", \"ADR-016\", \"ADR-099\"]",
            &docs,
            &signals,
        );
        assert_eq!(states(&adr), [Met, NotMet, NotMet]);

        let questions = units(
            "kind = \"inventory\"\nquestions = [\"Q-1\", \"Q-2\", \"Q-3\", \"Q-4\"]",
            &docs,
            &signals,
        );
        assert_eq!(states(&questions), [Met, NotMet, NotMet, NotMet]);
    }

    #[test]
    fn an_open_question_is_met_once_its_heading_is_gone() {
        let docs = Docs {
            open_questions: Some(
                "## OPEN items\n\n### OPEN-1: the rate\n\n### U-12: release terms\n".into(),
            ),
            ..Docs::default()
        };
        let signals = Signals::default();
        let read = units(
            "kind = \"open_question\"\nquestions = [\"OPEN-1\", \"OPEN-2\", \"U-12\"]",
            &docs,
            &signals,
        );
        assert_eq!(states(&read), [NotMet, Met, NotMet]);

        let missing = units(
            "kind = \"open_question\"\nquestions = [\"OPEN-2\"]",
            &Docs::default(),
            &signals,
        );
        assert_eq!(
            states(&missing),
            [NotMet],
            "a missing file must not read as every question decided"
        );
    }

    fn run(exit: i32, passed: &[&str], failed: &[&str]) -> TestRun {
        TestRun {
            suite: "unit".into(),
            finished_at: "2026-09-14T00:00:00Z".into(),
            commit: Some("abc1234".into()),
            exit_code: Some(exit),
            passed: passed.iter().map(|s| (*s).to_string()).collect(),
            failed: failed.iter().map(|s| (*s).to_string()).collect(),
            output_tail: String::new(),
        }
    }

    #[test]
    fn a_suite_is_met_only_by_a_clean_run_with_every_required_test() {
        let criterion = "kind = \"tests\"\nsuite = \"unit\"\nrequire = [\"b\"]";
        let docs = Docs::default();
        let mut signals = Signals::default();

        assert_eq!(
            states(&units(criterion, &docs, &signals)),
            [Unmeasurable],
            "never run"
        );

        signals
            .runs
            .insert("unit".into(), run(0, &["module::a", "module::b"], &[]));
        assert_eq!(states(&units(criterion, &docs, &signals)), [Met]);

        signals
            .runs
            .insert("unit".into(), run(0, &["module::a"], &[]));
        assert_eq!(
            states(&units(criterion, &docs, &signals)),
            [NotMet],
            "a required test is missing"
        );

        signals
            .runs
            .insert("unit".into(), run(101, &["module::b"], &["module::c"]));
        assert_eq!(
            states(&units(criterion, &docs, &signals)),
            [NotMet],
            "a failing run"
        );

        let nowhere = units("kind = \"tests\"\nsuite = \"nowhere\"", &docs, &signals);
        assert_eq!(
            states(&nowhere),
            [Unmeasurable],
            "a suite with no directory"
        );
    }

    #[test]
    fn ci_is_met_only_by_success_on_the_branch_head() {
        let criterion = "kind = \"ci\"\nworkflow = \"Pleroma CI\"\nbranch = \"main\"";
        let key = ("Pleroma CI".to_string(), "main".to_string());
        let reading = |conclusion: &str, head: &str| CiRun {
            id: 1,
            conclusion: conclusion.into(),
            status: "completed".into(),
            head_sha: head.into(),
            branch_head: "aaaa".into(),
            url: String::new(),
        };
        let docs = Docs::default();
        let mut signals = Signals::default();

        assert_eq!(states(&units(criterion, &docs, &signals)), [NotMet]);

        signals
            .ci
            .insert(key.clone(), Err("gh is not installed".into()));
        assert_eq!(
            states(&units(criterion, &docs, &signals)),
            [NotMet],
            "this kind defines no unmeasurable state, so an unreadable signal is not met"
        );

        signals
            .ci
            .insert(key.clone(), Ok(reading("success", "bbbb")));
        assert_eq!(
            states(&units(criterion, &docs, &signals)),
            [NotMet],
            "success on an older commit"
        );

        signals
            .ci
            .insert(key.clone(), Ok(reading("startup_failure", "aaaa")));
        assert_eq!(states(&units(criterion, &docs, &signals)), [NotMet]);

        signals.ci.insert(key, Ok(reading("success", "aaaa")));
        assert_eq!(states(&units(criterion, &docs, &signals)), [Met]);
    }

    #[test]
    fn services_and_pallets_without_a_location_are_unmeasurable() {
        let docs = Docs::default();
        let signals = Signals::default();
        let http = units(
            "kind = \"http\"\nurl = \"\"\nexpect_status = 200",
            &docs,
            &signals,
        );
        assert_eq!(states(&http), [Unmeasurable]);
        let pallet = units(
            "kind = \"pallet\"\npallets = []\nrequire = []",
            &docs,
            &signals,
        );
        assert_eq!(states(&pallet), [Unmeasurable]);
        let coverage = units(
            "kind = \"coverage\"\npallets = []\nthreshold = 80",
            &docs,
            &signals,
        );
        assert_eq!(states(&coverage), [Unmeasurable]);
    }

    #[test]
    fn coverage_compares_every_pallet_with_the_threshold() {
        let criterion =
            "kind = \"coverage\"\npallets = [\"pallet-a\", \"pallet-b\"]\nthreshold = 80";
        let docs = Docs::default();
        let mut signals = Signals {
            coverage: Some(Ok(None)),
            ..Signals::default()
        };

        assert_eq!(
            states(&units(criterion, &docs, &signals)),
            [Unmeasurable],
            "no report for the branch head"
        );

        let report =
            |b: f64| HashMap::from([("pallet-a".to_string(), 91.0), ("pallet-b".to_string(), b)]);
        signals.coverage = Some(Ok(Some(report(79.9))));
        assert_eq!(states(&units(criterion, &docs, &signals)), [NotMet]);

        signals.coverage = Some(Ok(Some(report(80.0))));
        assert_eq!(states(&units(criterion, &docs, &signals)), [Met]);
    }

    #[test]
    fn a_bar_appears_only_when_nothing_is_unmeasurable() {
        let text = "status = \"accepted\"\n\n\
            [[gates]]\nid = \"first\"\nname = \"First\"\n\n\
            [[gates.criteria]]\nid = \"a\"\nkind = \"check\"\ndone = true\nevidence = \"abc\"\n\n\
            [[gates.criteria]]\nid = \"b\"\nkind = \"check\"\ndone = false\nevidence = \"\"\n\n\
            [[gates]]\nid = \"second\"\nname = \"Second\"\n\n\
            [[gates.criteria]]\nid = \"c\"\nkind = \"gate\"\ngate = \"first\"\n\n\
            [[gates.criteria]]\nid = \"d\"\nkind = \"http\"\nurl = \"\"\nexpect_status = 200\n";
        let reports = evaluate(&file(text), &Docs::default(), &Signals::default());

        let first = &reports[0];
        assert_eq!((first.met, first.not_met, first.unmeasurable), (1, 1, 0));
        assert_eq!(first.bar, Some(Bar { met: 1, total: 2 }));
        assert!(!first.passed);

        let second = &reports[1];
        assert_eq!((second.met, second.not_met, second.unmeasurable), (0, 1, 1));
        assert_eq!(second.bar, None, "one unmeasurable unit means no bar");
        assert!(!second.passed);
    }

    #[test]
    fn cargo_test_output_is_read_per_test() {
        let output = "running 3 tests\n\
                      test vault::tests::a ... ok\n\
                      test chain::tests::b ... FAILED\n\
                      test c ... ignored\n\n\
                      test result: FAILED. 1 passed; 1 failed\n";
        let (passed, failed) = parse_test_output(output);
        assert_eq!(passed, ["vault::tests::a"]);
        assert_eq!(failed, ["chain::tests::b"]);
    }

    /// A repository holding only a gates file with one suite, `unit`.
    fn fixture_repo(name: &str, command: &str) -> PathBuf {
        let repo =
            std::env::temp_dir().join(format!("qor-gates-test-{name}-{}", std::process::id()));
        std::fs::create_dir_all(repo.join("docs")).unwrap();
        std::fs::create_dir_all(repo.join("suite")).unwrap();
        std::fs::write(
            repo.join(GATES_FILE),
            format!("status = \"accepted\"\n\n[suites.unit]\ndir = \"suite\"\ncommand = \"{command}\"\n"),
        )
        .unwrap();
        repo
    }

    #[tokio::test]
    async fn a_suite_runs_only_once_the_person_at_the_keyboard_approves() {
        use crate::testing::Scripted;

        let repo = fixture_repo("confirm", "cargo --version");
        let data = repo.join("data");
        let dashboard = Dashboard::with_repo(Some(repo.clone())).unwrap();

        let declining = Scripted::declining();
        let refused = dashboard
            .run_suite(&data, "unit", &declining)
            .await
            .unwrap_err();
        assert_eq!(refused.kind(), "declined");
        assert_eq!(declining.times_asked(), 1);
        assert!(
            declining.asked.lock()[0].body.contains("cargo --version"),
            "the prompt names the command it would run"
        );
        assert!(
            load_run(&data, "unit").is_none(),
            "nothing ran and nothing was recorded"
        );

        let unasked = Scripted::approving();
        dashboard
            .run_suite(&data, "undefined", &unasked)
            .await
            .unwrap_err();
        assert_eq!(
            unasked.times_asked(),
            0,
            "a suite the file does not define asks nothing"
        );

        let approving = Scripted::approving();
        let run = dashboard
            .run_suite(&data, "unit", &approving)
            .await
            .unwrap();
        assert_eq!(run.exit_code, Some(0));
        assert_eq!(
            load_run(&data, "unit"),
            Some(run),
            "the run is recorded for the tests kind"
        );

        std::fs::remove_dir_all(&repo).ok();
    }

    /// CI is read from the repository `[ci]` names, never from a remote `gh` picks,
    /// and a `[ci]` without one is refused rather than guessed.
    #[test]
    fn ci_is_read_from_the_named_repository_only() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let file = load_file(&repo).expect("docs/GATES.toml loads");
        assert_eq!(ci_repository(&file), Ok("QOR-MATRIX/demiurge-chain"));

        let unnamed: GatesFile =
            toml::from_str("status = \"x\"\n[ci]\nworkflow = \"Pleroma CI\"\n").unwrap();
        assert_eq!(
            ci_repository(&unnamed),
            Err("docs/GATES.toml names no [ci] repository".to_string())
        );
    }

    /// The real `docs/GATES.toml` parses, every kind in it is one this launcher
    /// reads, every document it names is readable, and the counts are consistent.
    #[test]
    fn the_repository_gates_file_is_readable_by_this_launcher() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let file = load_file(&repo).expect("docs/GATES.toml loads");
        let reports = evaluate(&file, &read_docs(&repo), &Signals::default());
        assert_eq!(reports.len(), file.gates.len());

        for gate in &reports {
            let total = gate.met + gate.not_met + gate.unmeasurable;
            assert_eq!(
                total,
                gate.criteria.iter().map(|c| c.units.len()).sum::<usize>()
            );
            assert_eq!(gate.bar.is_some(), gate.unmeasurable == 0 && total > 0);

            for criterion in &gate.criteria {
                for unit in &criterion.units {
                    assert!(
                        !unit.detail.starts_with("unknown kind"),
                        "{}: {}",
                        criterion.id,
                        unit.detail
                    );
                    assert!(
                        !unit.detail.contains("could not be read"),
                        "{}: {}",
                        criterion.id,
                        unit.detail
                    );
                    assert!(
                        !unit.detail.starts_with("lists no items"),
                        "{}: {}",
                        criterion.id,
                        unit.detail
                    );
                }
            }
        }
    }
}
