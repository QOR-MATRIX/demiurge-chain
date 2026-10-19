-- Migration 025: an agent's spending limit is never negative (19 October 2026).
--
-- An agent's spending limit is a whole number of CGT, recorded and not yet enforced (Q-9). The service refuses a
-- negative one; so does the table now. A row that already holds one keeps no meaning, so it is cleared first.
UPDATE users SET agent_spending_limit = NULL WHERE agent_spending_limit < 0;
ALTER TABLE users ADD CONSTRAINT agent_spending_limit_not_negative CHECK (agent_spending_limit >= 0);
COMMENT ON COLUMN users.agent_spending_limit IS
    'A whole number of CGT an agent may spend, recorded and not enforced (Q-9). pallet-agent-caps (M5.2) will enforce caps in embers.';
