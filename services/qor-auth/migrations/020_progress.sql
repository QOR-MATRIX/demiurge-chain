-- Levels, tasks and the welcome grant (ADR-078, accepted by the owner on 6 October 2026).
--
-- progress_events: each task an account completed, once (the primary key), with the XP it granted. An account's XP is
-- the sum; its level follows from that (src/progress.rs). Only QOR ID or a registered app's server writes here.
--
-- welcome_grants: the 100 CGT grant an account is owed once its tutorial, email and key are confirmed. One per account,
-- per email address and per chain account (the unique keys), whatever happens to the account later. Sparks as an exact
-- decimal. QOR ID records what is owed; it moves no CGT.
--
-- signup_addresses: a keyed hash of the network address an account was created from, never the address, kept 30 days,
-- for the owner's limit of three accounts per address.

CREATE TABLE progress_events (
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    task VARCHAR(40) NOT NULL,
    xp INTEGER NOT NULL CHECK (xp > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, task)
);

CREATE TABLE welcome_grants (
    user_id UUID PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    email_lower VARCHAR(255) NOT NULL UNIQUE,
    chain_account_id BYTEA NOT NULL UNIQUE CHECK (octet_length(chain_account_id) = 32),
    amount_sparks VARCHAR(40) NOT NULL CHECK (amount_sparks ~ '^[1-9][0-9]*$'),
    status VARCHAR(16) NOT NULL DEFAULT 'owed' CHECK (status IN ('owed', 'paid')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at TIMESTAMPTZ,
    paid_block VARCHAR(80)
);

CREATE TABLE signup_addresses (
    addr_hash CHAR(64) NOT NULL,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX signup_addresses_hash_idx ON signup_addresses (addr_hash, created_at);
