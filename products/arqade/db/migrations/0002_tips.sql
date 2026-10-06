-- Tips to a game's creator, paid through the QOR Launcher (ADR-071 decision 6 "Tips", ADR-076, ADR-077).
-- A row is a request ARQADE signed. It becomes `paid` only when a finalised block holds its remark's event and the
-- transfer in one extrinsic; `scanned_to` is the last finalised block searched. Amounts are Sparks as exact decimals.

CREATE TABLE tips (
  id text PRIMARY KEY,
  game text NOT NULL,
  -- The signed-in player who asked (players.id); the payer is whoever's launcher approves it.
  player text NOT NULL REFERENCES players (id),
  to_address text NOT NULL,
  amount text NOT NULL,
  label text NOT NULL,
  created bigint NOT NULL,
  expires bigint NOT NULL,
  start_block bigint NOT NULL,
  scanned_to bigint NOT NULL,
  status text NOT NULL,
  paid_block bigint,
  paid_hash text,
  payer text
);
CREATE INDEX tips_status_idx ON tips (status, created);
CREATE INDEX tips_player_idx ON tips (player, created);
