-- Room levels and account standing (ADR-093).

-- A room's levels, highest first: owner, admin, moderator, vip, member.
ALTER TABLE members DROP CONSTRAINT members_role_check;
ALTER TABLE members ADD CONSTRAINT members_role_check CHECK (role IN ('owner', 'admin', 'moderator', 'vip', 'member'));

-- Elite accounts: staff in every room they are in, granted and taken away only by godmode.
CREATE TABLE elites (
    sub UUID PRIMARY KEY,
    granted_by UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
