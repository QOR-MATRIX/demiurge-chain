-- Chosen names (ADR-091 decision 5, accepted by the owner on 15 October 2026, amending ADR-075).
--
-- An account registered by key without a name was given `key_` and the first four bytes of its account in hex. Such an
-- account chooses its username once, through POST /api/v1/profile/username; after that the name is fixed, as ADR-075
-- has it. users.username_chosen is false only while an account is still on a name QOR ID made for it.

ALTER TABLE users ADD COLUMN username_chosen BOOLEAN NOT NULL DEFAULT TRUE;

-- The names QOR ID made: exactly its pattern, on a key account of a person. A person who chose such a name by hand may
-- choose once more, which takes nothing from anyone.
UPDATE users
SET username_chosen = FALSE
WHERE auth_method = 'keypair'
  AND account_type = 'human'
  AND username ~ '^key_[0-9a-f]{8}$';
