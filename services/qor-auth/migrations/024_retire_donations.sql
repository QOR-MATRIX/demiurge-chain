-- Migration 024: the donation scheme of migration 005, retired (19 October 2026).
--
-- Migration 005 came from before the realignment. Its trigger gave fiat donors, by tier, a "staking rewards" bonus of
-- 2% to 10% and a fee discount of up to 10%, and its tables counted CGT "distributed" for dollars. Each is an economic
-- value docs/economics/OPEN_QUESTIONS.md leaves open (OPEN-4, U-7), presented as decided, and the language ADR-008 and
-- AGENTS.md section 6 rule out. No code in this service reads or writes any of it.
--
-- Nothing is deleted that might be a record of a payment: the three tables move, whole, to a schema of their own,
-- `retired`, where the service never looks, and their foreign keys still follow an account that is deleted. What goes
-- is what was computed, never paid: the trigger and functions that set the perks, and the perk columns they wrote.

DROP TRIGGER IF EXISTS trigger_update_donor_perks ON donor_profiles;
DROP FUNCTION IF EXISTS update_donor_perks();
DROP FUNCTION IF EXISTS get_effective_tier(UUID);

ALTER TABLE donor_profiles
    DROP COLUMN IF EXISTS staking_bonus_bps,
    DROP COLUMN IF EXISTS gas_discount_bps,
    DROP COLUMN IF EXISTS xp_rate_bonus_bps,
    DROP COLUMN IF EXISTS free_mints_remaining,
    DROP COLUMN IF EXISTS chat_privileges;

CREATE SCHEMA IF NOT EXISTS retired;
COMMENT ON SCHEMA retired IS
    'Tables QOR ID no longer uses, kept whole because they may record payments. Nothing in the service reads them.';

ALTER TABLE donations SET SCHEMA retired;
ALTER TABLE donor_profiles SET SCHEMA retired;
ALTER TABLE subscription_events SET SCHEMA retired;
ALTER TYPE donation_status SET SCHEMA retired;
ALTER TYPE subscription_status SET SCHEMA retired;

COMMENT ON TABLE retired.donations IS 'Retired by migration 024: donations under the scheme of migration 005';
COMMENT ON TABLE retired.donor_profiles IS 'Retired by migration 024: donor tiers under the scheme of migration 005';
COMMENT ON TABLE retired.subscription_events IS 'Retired by migration 024: subscriptions under the scheme of migration 005';
