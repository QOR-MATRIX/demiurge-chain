//! Migrations that change what a database already holds, tested against a real Postgres from the state before them.
//!
//! Every statement here is a literal, an `include_str!` or an upper-case constant, with its values bound, as
//! `sql_hygiene` requires of every query in the service.

use sqlx::PgPool;

/// Migrations 001 to 023, in order: the database migration 024 meets.
const BEFORE_024: [&str; 23] = [
    include_str!("../migrations/001_initial_schema.sql"),
    include_str!("../migrations/002_add_god_role.sql"),
    include_str!("../migrations/003_add_backup_code.sql"),
    include_str!("../migrations/004_music_schema.sql"),
    include_str!("../migrations/005_donations.sql"),
    include_str!("../migrations/006_keypair_auth.sql"),
    include_str!("../migrations/007_agent_accounts.sql"),
    include_str!("../migrations/008_seed_godmode.sql"),
    include_str!("../migrations/009_fix_on_chain_address.sql"),
    include_str!("../migrations/010_disable_seeded_admin.sql"),
    include_str!("../migrations/011_remove_server_generated_agent_keys.sql"),
    include_str!("../migrations/012_correct_inverted_email_verified.sql"),
    include_str!("../migrations/013_refuse_nil_user_id.sql"),
    include_str!("../migrations/014_single_use_backup_codes.sql"),
    include_str!("../migrations/015_verification_resend_limit.sql"),
    include_str!("../migrations/016_pending_email_change.sql"),
    include_str!("../migrations/017_email_suppressions.sql"),
    include_str!("../migrations/018_chain_accounts_as_bytes.sql"),
    include_str!("../migrations/019_unique_usernames.sql"),
    include_str!("../migrations/020_progress.sql"),
    include_str!("../migrations/021_progress_for_existing_accounts.sql"),
    include_str!("../migrations/022_avatars.sql"),
    include_str!("../migrations/023_chosen_names.sql"),
];
const MIGRATION_024: &str = include_str!("../migrations/024_retire_donations.sql");
const MIGRATION_025: &str = include_str!("../migrations/025_agent_spending_limit_not_negative.sql");

async fn migrate_before_024(db: &PgPool) {
    for sql in BEFORE_024 {
        sqlx::raw_sql(sql)
            .execute(db)
            .await
            .expect("an earlier migration");
    }
}

async fn relation_exists(db: &PgPool, name: &str) -> bool {
    sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
        .bind(name)
        .fetch_one(db)
        .await
        .expect("the catalogue")
}

async fn donor_column_exists(db: &PgPool, column: &str) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'donor_profiles' AND column_name = $1)",
    )
    .bind(column)
    .fetch_one(db)
    .await
    .expect("the catalogue")
}

async fn function_exists(db: &PgPool, name: &str) -> bool {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_proc WHERE proname = $1)")
        .bind(name)
        .fetch_one(db)
        .await
        .expect("the catalogue")
}

/// Migration 024 retires migration 005's donation scheme (AGENTS.md sections 5 and 6): what was computed goes (the
/// trigger that set a "staking" bonus and a fee discount by fiat tier, and the perk columns it wrote), and nothing
/// that might record a payment is lost. The three tables move whole to the schema `retired`, still tied to accounts.
#[sqlx::test(migrations = false)]
async fn migration_024_keeps_every_record_and_drops_what_was_computed(db: PgPool) {
    migrate_before_024(&db).await;

    let user: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, email_verified) VALUES ('donor', 'x', TRUE) RETURNING id",
    )
    .fetch_one(&db)
    .await
    .expect("an account");
    sqlx::query(
        "INSERT INTO donor_profiles (user_id, current_tier, lifetime_cents) VALUES ($1, 3, 2500)",
    )
    .bind(user)
    .execute(&db)
    .await
    .expect("a donor profile");
    let set_by_005: i32 =
        sqlx::query_scalar("SELECT staking_bonus_bps FROM donor_profiles WHERE user_id = $1")
            .bind(user)
            .fetch_one(&db)
            .await
            .expect("the perk the trigger set");
    assert_eq!(set_by_005, 600, "migration 005's trigger, as it stood");
    sqlx::query(
        "INSERT INTO donations (user_id, amount_cents, status, tier_at_time, stripe_session_id) \
         VALUES ($1, 2500, 'completed', 3, 'cs_test_kept')",
    )
    .bind(user)
    .execute(&db)
    .await
    .expect("a donation");
    sqlx::query(
        "INSERT INTO subscription_events (user_id, event_type, amount_cents) VALUES ($1, 'created', 900)",
    )
    .bind(user)
    .execute(&db)
    .await
    .expect("a subscription event");

    sqlx::raw_sql(MIGRATION_024)
        .execute(&db)
        .await
        .expect("migration 024");

    // Every record is kept, in `retired`.
    let kept: (i32, String) = sqlx::query_as(
        "SELECT amount_cents, stripe_session_id FROM retired.donations WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db)
    .await
    .expect("the donation, kept");
    assert_eq!(kept, (2500, "cs_test_kept".to_string()));
    let tier: (i32, i32) = sqlx::query_as(
        "SELECT current_tier, lifetime_cents FROM retired.donor_profiles WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db)
    .await
    .expect("the donor profile, kept");
    assert_eq!(tier, (3, 2500));
    let events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM retired.subscription_events")
        .fetch_one(&db)
        .await
        .expect("the subscription events, kept");
    assert_eq!(events, 1);

    // Nothing of it is where the service looks, and what was computed is gone.
    for table in [
        "public.donations",
        "public.donor_profiles",
        "public.subscription_events",
    ] {
        assert!(
            !relation_exists(&db, table).await,
            "{table} should have moved"
        );
    }
    for column in [
        "staking_bonus_bps",
        "gas_discount_bps",
        "xp_rate_bonus_bps",
        "free_mints_remaining",
        "chat_privileges",
    ] {
        assert!(
            !donor_column_exists(&db, column).await,
            "{column} should have been dropped"
        );
    }
    for function in ["update_donor_perks", "get_effective_tier"] {
        assert!(
            !function_exists(&db, function).await,
            "{function} should have been dropped"
        );
    }

    // A tier changed now computes nothing.
    sqlx::query("UPDATE retired.donor_profiles SET current_tier = 5 WHERE user_id = $1")
        .bind(user)
        .execute(&db)
        .await
        .expect("an update with no trigger");

    // Deleting the account still deletes what was tied to it.
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user)
        .execute(&db)
        .await
        .expect("the account deleted");
    let left: i64 = sqlx::query_scalar(
        "SELECT (SELECT COUNT(*) FROM retired.donations) + (SELECT COUNT(*) FROM retired.donor_profiles) \
         + (SELECT COUNT(*) FROM retired.subscription_events)",
    )
    .fetch_one(&db)
    .await
    .expect("counts");
    assert_eq!(left, 0);
}

/// On a database that never had a donation, 024 runs just the same: production's state is not known to hold any.
#[sqlx::test(migrations = false)]
async fn migration_024_runs_on_a_database_with_no_donations(db: PgPool) {
    migrate_before_024(&db).await;
    sqlx::raw_sql(MIGRATION_024)
        .execute(&db)
        .await
        .expect("migration 024");
    assert!(relation_exists(&db, "retired.donations").await);
}

/// Migration 025: an agent's spending limit is never negative. One already stored is cleared, since it never meant
/// anything, and the table refuses another; zero and a positive limit are kept.
#[sqlx::test(migrations = false)]
async fn migration_025_refuses_a_negative_spending_limit(db: PgPool) {
    migrate_before_024(&db).await;
    sqlx::raw_sql(MIGRATION_024)
        .execute(&db)
        .await
        .expect("migration 024");
    sqlx::raw_sql(
        "INSERT INTO users (username, password_hash, email_verified, agent_spending_limit) VALUES \
         ('negative', 'x', TRUE, -5), ('nothing', 'x', TRUE, 0), ('some', 'x', TRUE, 250)",
    )
    .execute(&db)
    .await
    .expect("limits as the service stored them before");

    sqlx::raw_sql(MIGRATION_025)
        .execute(&db)
        .await
        .expect("migration 025");

    let limits: Vec<(String, Option<i64>)> =
        sqlx::query_as("SELECT username, agent_spending_limit FROM users ORDER BY username")
            .fetch_all(&db)
            .await
            .expect("the limits");
    assert_eq!(
        limits,
        [
            ("negative".to_string(), None),
            ("nothing".to_string(), Some(0)),
            ("some".to_string(), Some(250)),
        ]
    );
    let refused = sqlx::query("UPDATE users SET agent_spending_limit = -1 WHERE username = 'some'")
        .execute(&db)
        .await;
    assert!(refused.is_err(), "a negative limit is refused by the table");
}
