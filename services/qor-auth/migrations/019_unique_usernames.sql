-- One name per account (ADR-075, the owner's decision of 5 October 2026). A QOR ID is the username
-- alone; the `#0001` discriminator is no longer shown or used.
--
-- Accounts that already share a name (only agent accounts could: every sign-up for people refused a
-- taken name) are resolved oldest-first, as the owner chose: the earliest-created account keeps the
-- name, each later one becomes `name_2`, `name_3`… (truncated to fit the 20-character limit), and the
-- name it had is kept in `renamed_from` so the account can be told at its next sign-in. Nothing is
-- deleted.

ALTER TABLE users ADD COLUMN renamed_from VARCHAR(20);

DO $$
DECLARE
    dup RECORD;
    n INTEGER;
    candidate TEXT;
BEGIN
    FOR dup IN
        SELECT id, username
        FROM (
            SELECT id, username,
                   ROW_NUMBER() OVER (PARTITION BY LOWER(username) ORDER BY created_at, id) AS place
            FROM users
        ) ranked
        WHERE place > 1
        ORDER BY LOWER(username), place
    LOOP
        n := 2;
        LOOP
            candidate := LEFT(LOWER(dup.username), 20 - LENGTH('_' || n)) || '_' || n;
            EXIT WHEN NOT EXISTS (SELECT 1 FROM users WHERE LOWER(username) = candidate);
            n := n + 1;
        END LOOP;
        UPDATE users SET renamed_from = username, username = candidate WHERE id = dup.id;
    END LOOP;
END $$;

-- With every name unique, the discriminator carries nothing. It stays a column, always 1, so that
-- clients reading it (the QOR Launcher 0.1.6) keep working.
UPDATE users SET discriminator = 1 WHERE discriminator <> 1;
ALTER TABLE users ALTER COLUMN discriminator SET DEFAULT 1;
COMMENT ON COLUMN users.discriminator IS 'Unused since ADR-075: always 1. A username is unique on its own.';

-- The rule itself: one account per name, whatever its letter case.
CREATE UNIQUE INDEX users_username_unique ON users (LOWER(username));
