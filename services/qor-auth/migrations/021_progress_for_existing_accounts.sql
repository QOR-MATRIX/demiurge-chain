-- Accounts created before levels existed (ADR-078) get the XP for what they had already done: a verified email address
-- and a linked key. XP is otherwise granted at the moment a task happens, and for them it happened before migration 020.
-- An account without an email address is marked verified (it has nothing to verify), so it is not counted as one that
-- verified. The welcome grant still waits for the tutorial: reporting it records the grant, as for anyone.

INSERT INTO progress_events (user_id, task, xp)
SELECT id, 'verify-email', 25 FROM users
WHERE email IS NOT NULL AND email_verified AND status = 'active'
ON CONFLICT DO NOTHING;

INSERT INTO progress_events (user_id, task, xp)
SELECT id, 'link-key', 25 FROM users
WHERE chain_account_id IS NOT NULL AND status = 'active'
ON CONFLICT DO NOTHING;
