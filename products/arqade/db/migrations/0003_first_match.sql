-- Whether a player's first finished match has been reported to QOR ID as an ADR-078 task, so it is reported once.
ALTER TABLE players ADD COLUMN first_match_reported integer NOT NULL DEFAULT 0;
