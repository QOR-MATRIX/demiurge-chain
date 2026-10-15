-- Social's own database (ADR-091, docs/blueprints/social.md section 3).
--
-- A person is QOR ID's `sub`. Their QOR ID, avatar and level are QOR ID's, copied here only as QOR ID last returned
-- them, so a card can be shown while they are away. Message bodies are sealed before they arrive (src/seal.rs).

CREATE TABLE profiles (
    sub UUID PRIMARY KEY,
    qor_id VARCHAR(20) NOT NULL,
    avatar_url VARCHAR(512),
    level INTEGER NOT NULL DEFAULT 0,
    display VARCHAR(48) NOT NULL DEFAULT '',
    bio VARCHAR(600) NOT NULL DEFAULT '',
    accent CHAR(7) CHECK (accent ~ '^#[0-9a-f]{6}$'),
    banner JSONB NOT NULL DEFAULT '{}'::jsonb,
    showcase JSONB NOT NULL DEFAULT '[]'::jsonb,
    status_line VARCHAR(80) NOT NULL DEFAULT '',
    seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX profiles_by_qor_id ON profiles (LOWER(qor_id));

CREATE TABLE rooms (
    id UUID PRIMARY KEY,
    kind VARCHAR(8) NOT NULL CHECK (kind IN ('public', 'private', 'direct')),
    name VARCHAR(48) NOT NULL DEFAULT '',
    topic VARCHAR(280) NOT NULL DEFAULT '',
    ambience JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- The two people of a direct room, in order, so one pair has one room.
    direct_key VARCHAR(73) UNIQUE,
    created_by UUID NOT NULL,
    -- The sequence the next message takes; gapless per room.
    next_seq BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK ((kind = 'direct') = (direct_key IS NOT NULL))
);

CREATE TABLE members (
    room UUID NOT NULL REFERENCES rooms (id) ON DELETE CASCADE,
    sub UUID NOT NULL,
    role VARCHAR(10) NOT NULL CHECK (role IN ('owner', 'moderator', 'member')),
    last_read BIGINT NOT NULL DEFAULT 0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room, sub)
);
CREATE INDEX members_by_sub ON members (sub);

CREATE TABLE invites (
    room UUID NOT NULL REFERENCES rooms (id) ON DELETE CASCADE,
    invitee UUID NOT NULL,
    inviter UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room, invitee)
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    room UUID NOT NULL REFERENCES rooms (id) ON DELETE CASCADE,
    seq BIGINT NOT NULL,
    author UUID NOT NULL,
    -- Which of the service's keys sealed it, and the nonce; the room and sequence are its associated data.
    key_version SMALLINT NOT NULL,
    nonce BYTEA NOT NULL,
    -- NULL once deleted: a deleted message keeps its place, never its words.
    sealed BYTEA,
    reply_to UUID REFERENCES messages (id) ON DELETE SET NULL,
    -- What the client sent with it, so a retry after a dropped connection is not a second message.
    client_nonce VARCHAR(64),
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (room, seq),
    UNIQUE (author, client_nonce)
);

CREATE TABLE reactions (
    message UUID NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    sub UUID NOT NULL,
    emoji VARCHAR(32) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (message, sub, emoji)
);

CREATE TABLE blocks (
    blocker UUID NOT NULL,
    blocked UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (blocker, blocked),
    CHECK (blocker <> blocked)
);
CREATE INDEX blocks_by_blocked ON blocks (blocked);

CREATE TABLE reports (
    id UUID PRIMARY KEY,
    message UUID NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    reporter UUID NOT NULL,
    reason VARCHAR(16) NOT NULL CHECK (reason IN ('spam', 'harassment', 'hate', 'violence', 'sexual', 'other')),
    note VARCHAR(500) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    UNIQUE (message, reporter)
);

-- Rate limits per person and action, so several sockets share one budget (docs/blueprints/social.md section 4).
CREATE TABLE rate_buckets (
    sub UUID NOT NULL,
    action VARCHAR(16) NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (sub, action)
);
