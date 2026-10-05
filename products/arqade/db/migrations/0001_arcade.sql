-- ARQADE's database on Postgres (ADR-074). Times are milliseconds since the epoch, hence bigint.
-- Applied in order by scripts/migrate.mjs, which records each file in arqade_migrations.

-- A player is a QOR ID account: `id` is the SHA-256 of its QOR `sub`, `alias` its QOR ID (name#0001).
CREATE TABLE players (
  id text PRIMARY KEY,
  alias text NOT NULL,
  created bigint NOT NULL,
  seen bigint NOT NULL,
  last_chat bigint NOT NULL DEFAULT 0,
  chat_gate text
);
CREATE INDEX players_seen_idx ON players (seen);

CREATE TABLE matches (
  id text PRIMARY KEY,
  game text NOT NULL,
  host text NOT NULL REFERENCES players (id),
  guest text REFERENCES players (id),
  status text NOT NULL,
  board text NOT NULL,
  turn integer NOT NULL,
  winner integer,
  moves integer NOT NULL DEFAULT 0,
  revision integer NOT NULL DEFAULT 0,
  created bigint NOT NULL,
  updated bigint NOT NULL,
  deadline bigint NOT NULL,
  last_action text,
  settled integer NOT NULL DEFAULT 0
);
CREATE INDEX matches_status_created_idx ON matches (status, created);
CREATE INDEX matches_host_status_idx ON matches (host, status);
CREATE INDEX matches_guest_status_idx ON matches (guest, status);

CREATE TABLE messages (
  id text PRIMARY KEY,
  player text NOT NULL REFERENCES players (id),
  body text NOT NULL,
  created bigint NOT NULL,
  deleted integer NOT NULL DEFAULT 0
);
CREATE INDEX messages_created_idx ON messages (created);

CREATE TABLE reports (
  message text NOT NULL REFERENCES messages (id),
  player text NOT NULL REFERENCES players (id),
  reason text NOT NULL,
  created bigint NOT NULL,
  PRIMARY KEY (message, player)
);

CREATE TABLE standings (
  player text NOT NULL REFERENCES players (id),
  game text NOT NULL,
  wins integer NOT NULL DEFAULT 0,
  losses integer NOT NULL DEFAULT 0,
  draws integer NOT NULL DEFAULT 0,
  points integer NOT NULL DEFAULT 0,
  updated bigint NOT NULL,
  PRIMARY KEY (player, game)
);
CREATE INDEX standings_game_points_idx ON standings (game, points);

-- QOR ID sign-in (ADR-073): a pending sign-in, single use for ten minutes, and a signed-in session
-- whose id is the SHA-256 of the browser's cookie, never the cookie itself. `checked` is when QOR ID
-- last confirmed the session (ADR-074).
CREATE TABLE qor_logins (
  state text PRIMARY KEY,
  verifier text NOT NULL,
  created bigint NOT NULL
);
CREATE INDEX qor_logins_created_idx ON qor_logins (created);

CREATE TABLE qor_sessions (
  id text PRIMARY KEY,
  sub text NOT NULL,
  qor_id text NOT NULL,
  username text NOT NULL,
  chain_account text,
  access_token text NOT NULL,
  refresh_token text NOT NULL,
  created bigint NOT NULL,
  expires bigint NOT NULL,
  checked bigint NOT NULL
);
CREATE INDEX qor_sessions_expires_idx ON qor_sessions (expires);
CREATE INDEX qor_sessions_sub_idx ON qor_sessions (sub);
