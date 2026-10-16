-- World, friends and circles, and the language of what is said (ADR-092).

-- World: a fourth kind of room, of which there is exactly one. Nobody created it, so its creator is the nil id.
ALTER TABLE rooms DROP CONSTRAINT rooms_kind_check;
ALTER TABLE rooms ADD CONSTRAINT rooms_kind_check CHECK (kind IN ('public', 'private', 'direct', 'world'));
CREATE UNIQUE INDEX rooms_one_world ON rooms (kind) WHERE kind = 'world';
INSERT INTO rooms (id, kind, name, topic, created_by)
VALUES (gen_random_uuid(), 'world', 'World', 'Everyone on QOR, in one room.', '00000000-0000-0000-0000-000000000000');

-- Someone a moderator has stopped from posting in a room until a time.
CREATE TABLE mutes (
    room UUID NOT NULL REFERENCES rooms (id) ON DELETE CASCADE,
    sub UUID NOT NULL,
    until TIMESTAMPTZ NOT NULL,
    muted_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room, sub)
);

-- The language a message was written in, as its sender's launcher found it; and the language a person reads in.
ALTER TABLE messages ADD COLUMN lang VARCHAR(16);
ALTER TABLE profiles ADD COLUMN language VARCHAR(16);

-- Friendship is mutual: a request, then a pair, kept once with the lower id first.
CREATE TABLE friend_requests (
    requester UUID NOT NULL,
    addressee UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (requester, addressee),
    CHECK (requester <> addressee)
);
CREATE INDEX friend_requests_by_addressee ON friend_requests (addressee);

CREATE TABLE friendships (
    low UUID NOT NULL,
    high UUID NOT NULL,
    since TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (low, high),
    CHECK (low < high)
);
CREATE INDEX friendships_by_high ON friendships (high);

-- A circle is one person's own group of their friends; nobody else sees it.
CREATE TABLE circles (
    id UUID PRIMARY KEY,
    owner UUID NOT NULL,
    name VARCHAR(32) NOT NULL,
    color CHAR(7) CHECK (color ~ '^#[0-9a-f]{6}$'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX circles_by_owner ON circles (owner);

CREATE TABLE circle_members (
    circle UUID NOT NULL REFERENCES circles (id) ON DELETE CASCADE,
    sub UUID NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (circle, sub)
);
CREATE INDEX circle_members_by_sub ON circle_members (sub);
