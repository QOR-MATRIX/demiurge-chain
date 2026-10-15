//! The check that every SQL statement is a literal with bound parameters.
//!
//! sqlx logs each statement's full text at debug level. A value formatted into SQL text, rather than
//! bound as a parameter, would reach the log, and be open to injection besides. Every query is a
//! literal today; this makes that a rule the build enforces, rather than a habit and a note.
//!
//! It reads the crate's source and fails on:
//! - a call to `sqlx::query`, `query_as`, `query_scalar`, `query_with`, `query_as_with`,
//!   `query_scalar_with` or `raw_sql` whose SQL is not fixed at compile time: a string literal,
//!   `include_str!`, an upper-case constant, or a loop variable over an upper-case constant;
//! - importing one of those functions, which would let a call escape the `sqlx::` prefix read here;
//! - `QueryBuilder::new` given anything but a literal, or `push` given `format!`;
//! - `format!` or `push_str` whose text reads as SQL.
//!
//! The query macros (`sqlx::query!` and the rest) accept only literals, and pass. This file is left
//! out of the scan of the source, because its tests quote the patterns it looks for.
//!
//! Copied from QOR ID's (services/qor-auth/src/sql_hygiene.rs), whose rule Social adopts from its first commit
//! (ADR-091); CI runs it with the other tests.

use std::fs;
use std::path::{Path, PathBuf};

const FUNCTIONS: [&str; 7] = [
    "query",
    "query_as",
    "query_scalar",
    "query_with",
    "query_as_with",
    "query_scalar_with",
    "raw_sql",
];

const SQL_WORDS: [&str; 7] = [
    "SELECT",
    "INSERT INTO",
    "UPDATE",
    "DELETE FROM",
    "WHERE",
    "VALUES",
    "RETURNING",
];

/// What the scan of one piece of source found.
struct Scan {
    problems: Vec<String>,
    /// How many sqlx query calls it checked, so a scan that saw nothing cannot pass as clean.
    queries: usize,
}

/// Every line that is only a comment, blanked, so line numbers still match.
fn without_comment_lines(source: &str) -> String {
    source
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("//") {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn skip_space(code: &str, mut at: usize) -> usize {
    while let Some(c) = code[at..].chars().next().filter(|c| c.is_whitespace()) {
        at += c.len_utf8();
    }
    at
}

/// The position after the `>` that closes the `<` at `at`.
fn after_angles(code: &str, at: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, c) in code[at..].char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(at + offset + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether the argument that starts `rest` is a string literal or an upper-case constant.
fn is_literal(rest: &str) -> bool {
    // A literal, or a file's text read in at compile time.
    if rest.starts_with('"')
        || rest.starts_with("r\"")
        || rest.starts_with("r#")
        || rest.starts_with("include_str!(")
    {
        return true;
    }
    let path: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
        .collect();
    let name = path.rsplit("::").next().unwrap_or_default();
    let after = rest[path.len()..].trim_start();
    name.starts_with(|c: char| c.is_ascii_uppercase())
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && (after.starts_with(',') || after.starts_with(')'))
}

/// Whether `name` is bound, before `at`, by iterating an upper-case constant, as in
/// `for sql in MIGRATIONS_BEFORE_012 {`. Every value it takes is then fixed at compile time.
fn iterates_a_constant(code: &str, at: usize, name: &str) -> bool {
    let binding = format!("for {name} in ");
    let Some(found) = code[..at].rfind(&binding) else {
        return false;
    };
    let rest = code[found + binding.len()..].trim_start_matches('&');
    let constant: String = rest
        .chars()
        .take_while(|c: &char| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
        .collect();
    constant.starts_with(|c: char| c.is_ascii_uppercase())
        && rest[constant.len()..].trim_start().starts_with('{')
}

/// The text of the string literal that starts `rest`, if one does.
fn string_at(rest: &str) -> Option<&str> {
    if let Some(body) = rest.strip_prefix('"') {
        let mut escaped = false;
        for (i, c) in body.char_indices() {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Some(&body[..i]);
            }
        }
        None
    } else {
        let raw = rest.strip_prefix('r')?;
        let hashes = raw.chars().take_while(|c| *c == '#').count();
        let body = raw[hashes..].strip_prefix('"')?;
        let close = format!("\"{}", "#".repeat(hashes));
        body.find(&close).map(|end| &body[..end])
    }
}

fn reads_as_sql(text: &str) -> bool {
    let word_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
    SQL_WORDS.iter().any(|word| {
        text.match_indices(word).any(|(at, _)| {
            !text[..at].chars().next_back().is_some_and(word_char)
                && !text[at + word.len()..]
                    .chars()
                    .next()
                    .is_some_and(word_char)
        })
    })
}

fn scan(source: &str) -> Scan {
    let code = without_comment_lines(source);
    let line = |at: usize| code[..at].matches('\n').count() + 1;
    let mut problems = Vec::new();
    let mut queries = 0;

    // sqlx's query functions, given anything but a literal.
    let mut from = 0;
    while let Some(found) = code[from..].find("sqlx::") {
        let start = from + found;
        from = start + "sqlx::".len();
        let name: String = code[from..]
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '_')
            .collect();
        if !FUNCTIONS.contains(&name.as_str()) {
            continue;
        }
        let mut at = skip_space(&code, from + name.len());
        if code[at..].starts_with("::<") {
            let Some(end) = after_angles(&code, at + 2) else {
                continue;
            };
            at = skip_space(&code, end);
        }
        // Not a call: a path in a `use`, or a macro such as `query!`.
        if !code[at..].starts_with('(') {
            continue;
        }
        queries += 1;
        let argument = skip_space(&code, at + 1);
        let variable: String = code[argument..]
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        let over_a_constant = !variable.is_empty()
            && code[argument + variable.len()..]
                .trim_start()
                .starts_with(')')
            && iterates_a_constant(&code, start, &variable);
        if !is_literal(&code[argument..]) && !over_a_constant {
            problems.push(format!(
                "line {}: sqlx::{name} is given something other than a string literal",
                line(start)
            ));
        }
    }

    // Importing a query function would let its calls escape the prefix read above.
    for (at, _) in code.match_indices("use sqlx::") {
        let statement = code[at..].split(';').next().unwrap_or_default();
        for word in statement.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if FUNCTIONS.contains(&word) {
                problems.push(format!(
                    "line {}: imports sqlx::{word}, which would let a query escape this check",
                    line(at)
                ));
            }
        }
    }

    // QueryBuilder: a literal to start, and bound values after.
    if code.contains("QueryBuilder") {
        for (at, call) in code.match_indices("QueryBuilder::new(") {
            if !is_literal(&code[skip_space(&code, at + call.len())..]) {
                problems.push(format!(
                    "line {}: QueryBuilder::new is given something other than a string literal",
                    line(at)
                ));
            }
        }
        for (at, call) in code.match_indices(".push(") {
            let argument = &code[skip_space(&code, at + call.len())..];
            if argument.starts_with("format!") || argument.starts_with("&format!") {
                problems.push(format!(
                    "line {}: QueryBuilder::push is given format!",
                    line(at)
                ));
            }
        }
    }

    // SQL text assembled at run time, whatever it is passed to later.
    for call in ["format!(", "push_str("] {
        for (at, _) in code.match_indices(call) {
            if string_at(&code[skip_space(&code, at + call.len())..]).is_some_and(reads_as_sql) {
                problems.push(format!(
                    "line {}: {call} builds text that reads as SQL",
                    line(at)
                ));
            }
        }
    }

    Scan { problems, queries }
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("the source directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn every_query_in_the_source_is_a_literal_with_bound_parameters() {
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );

    let mut queries = 0;
    let mut problems = Vec::new();
    for path in files
        .iter()
        .filter(|path| !path.ends_with("sql_hygiene.rs"))
    {
        let scan = scan(&fs::read_to_string(path).expect("source"));
        queries += scan.queries;
        problems.extend(
            scan.problems
                .into_iter()
                .map(|problem| format!("{}: {problem}", path.display())),
        );
    }

    // A clean result means nothing if the scan saw none of the service's queries.
    assert!(
        queries >= 50,
        "the scan should see the service's queries; it saw {queries}"
    );
    assert!(
        problems.is_empty(),
        "SQL built from values; bind them as parameters instead:\n{}",
        problems.join("\n")
    );
}

#[test]
fn a_query_built_from_values_is_found() {
    for (snippet, expected) in [
        (
            "sqlx::query(&format!(\"SELECT id FROM users WHERE email = '{email}'\"))",
            "sqlx::query is given",
        ),
        (
            "let sql = build(); sqlx::query(&sql).execute(&db)",
            "sqlx::query is given",
        ),
        (
            "sqlx::query_as::<_, (Uuid, Option<String>)>(sql.as_str())",
            "sqlx::query_as is given",
        ),
        (
            "sqlx::query_scalar::<_, i64>(\n    &statement,\n)",
            "sqlx::query_scalar is given",
        ),
        ("sqlx::raw_sql(&statement)", "sqlx::raw_sql is given"),
        (
            "for sql in statements {\n    sqlx::raw_sql(sql).execute(&db);\n}",
            "sqlx::raw_sql is given",
        ),
        (
            "let sql = pick(); sqlx::raw_sql(sql)",
            "sqlx::raw_sql is given",
        ),
        ("use sqlx::{query, PgPool};", "imports sqlx::query"),
        ("use sqlx::query_as as fetch;", "imports sqlx::query_as"),
        (
            "let q = format!(\"UPDATE users SET name = '{}'\", name);",
            "format!( builds text",
        ),
        ("sql.push_str(\" WHERE id = \");", "push_str( builds text"),
        (
            "let mut b = QueryBuilder::new(sql);",
            "QueryBuilder::new is given",
        ),
        (
            "let mut b = QueryBuilder::new(\"SELECT id FROM users\");\nb.push(format!(\" AND email = '{email}'\"));",
            "QueryBuilder::push is given format!",
        ),
    ] {
        let found = scan(snippet).problems;
        assert!(
            found.iter().any(|problem| problem.contains(expected)),
            "{snippet:?} should be found as {expected:?}; found {found:?}"
        );
    }
}

#[test]
fn literal_queries_with_bound_parameters_pass() {
    for snippet in [
        "sqlx::query(\"UPDATE users SET name = $1 WHERE id = $2\").bind(name).bind(id)",
        "sqlx::query(\n    r#\"\n    SELECT id FROM users WHERE email = $1\n    \"#,\n)\n.bind(email)",
        "sqlx::query_as::<_, (Uuid, Option<String>, chrono::DateTime<chrono::Utc>)>(\n    \"SELECT id, email, created_at FROM users\",\n)",
        "sqlx::query_scalar(COUNT_USERS).fetch_one(&db)",
        "sqlx::query_scalar(queries::COUNT_USERS)",
        "sqlx::query!(\"SELECT 1\")",
        "use sqlx::PgPool;\nuse sqlx::postgres::PgPoolOptions;",
        "format!(\"Someone asked to reset the password for the QOR ID {username}.\")",
        "// sqlx::query(&format!(\"SELECT {x}\"))",
        "lines.push(format!(\"{reason}\"));",
        "for sql in MIGRATIONS_BEFORE_012 {\n    sqlx::raw_sql(sql).execute(&db).await.expect(\"earlier migration\");\n}",
        "sqlx::raw_sql(include_str!(\n    \"../../migrations/012_correct_inverted_email_verified.sql\"\n))",
    ] {
        let scan = scan(snippet);
        assert!(
            scan.problems.is_empty(),
            "{snippet:?} should pass; found {:?}",
            scan.problems
        );
    }
    assert_eq!(
        scan("sqlx::query(\"SELECT 1\").execute(&db)").queries,
        1,
        "a literal query is still counted"
    );
}
