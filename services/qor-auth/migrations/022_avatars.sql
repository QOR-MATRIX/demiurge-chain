-- Avatars (ADR-079, accepted by the owner on 6 October 2026), kept by QOR ID in its own database.
--
-- avatars: an account's current avatar, a clean re-encoding (never the bytes uploaded), named by the SHA-256 of what is
-- stored. users.avatar_url points at it (`/avatars/<hash>`).
-- avatar_reports: who reported which avatar and why, once per reporter.
-- avatar_removals: an avatar the owner removed, so the account can be told.

CREATE TABLE avatars (
    user_id UUID PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    -- Not unique: two accounts may upload the same picture, which then re-encodes to the same bytes.
    hash CHAR(64) NOT NULL,
    content_type VARCHAR(16) NOT NULL CHECK (content_type IN ('image/png', 'image/gif')),
    bytes BYTEA NOT NULL CHECK (octet_length(bytes) <= 2097152),
    -- For an animated avatar, its first frame as a PNG, for people who asked for less motion (ADR-079 decision 6).
    still BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX avatars_hash_idx ON avatars (hash);

CREATE TABLE avatar_reports (
    hash CHAR(64) NOT NULL,
    reporter UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    reason VARCHAR(32) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (hash, reporter)
);

CREATE TABLE avatar_removals (
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    hash CHAR(64) NOT NULL,
    removed_by UUID REFERENCES users (id) ON DELETE SET NULL,
    removed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    seen BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX avatar_removals_user_idx ON avatar_removals (user_id, seen);
