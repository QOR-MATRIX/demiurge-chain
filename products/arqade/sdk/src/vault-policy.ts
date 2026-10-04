// A Game Vault's payout policy (ADR-070, Proposed), checked before the governor signs it. Every value is the
// developer's own; this module sets none. The protocol's bounds on delays and windows are U-16 and are passed in,
// never assumed.

import { checkSparks } from './amount';

export type VaultPolicy = {
  maxPayout: bigint;
  epochBlocks: number;
  epochBudget: bigint;
  perRecipientPerEpoch: bigint;
  ruleVersions: string[];
  accrualExpiryBlocks: number;
  loosenDelayBlocks: number;
};

/** What the connected runtime allows. Unknown until the pallet exists; a caller supplies what it read. */
export type ProtocolBounds = {
  minLoosenDelayBlocks: number;
  maxAccrualExpiryBlocks: number;
  maxRuleVersions: number;
};

const RULE_VERSION = /^[a-z0-9][a-z0-9._-]{0,47}@[0-9]{1,9}$/;

/** Every problem with a policy, in plain words; an empty list means it may be signed. */
export function validatePolicy(p: VaultPolicy, bounds: ProtocolBounds): string[] {
  const problems: string[] = [];
  for (const key of ['maxPayout', 'epochBudget', 'perRecipientPerEpoch'] as const) {
    try {
      checkSparks(p[key]);
      if (p[key] === BigInt(0)) problems.push(`${key} is zero, so nothing could ever be paid`);
    } catch {
      problems.push(`${key} is outside 0..u128`);
    }
  }
  for (const key of ['epochBlocks', 'accrualExpiryBlocks', 'loosenDelayBlocks'] as const) {
    if (!Number.isSafeInteger(p[key]) || p[key] <= 0) problems.push(`${key} must be a whole number of blocks above zero`);
  }
  if (p.maxPayout > p.perRecipientPerEpoch) problems.push('maxPayout is larger than perRecipientPerEpoch, so it could never be paid');
  if (p.perRecipientPerEpoch > p.epochBudget) problems.push('perRecipientPerEpoch is larger than epochBudget');
  if (p.loosenDelayBlocks < bounds.minLoosenDelayBlocks) problems.push(`loosenDelayBlocks is below the protocol's minimum of ${bounds.minLoosenDelayBlocks}`);
  if (p.accrualExpiryBlocks > bounds.maxAccrualExpiryBlocks) problems.push(`accrualExpiryBlocks is above the protocol's maximum of ${bounds.maxAccrualExpiryBlocks}`);
  if (p.ruleVersions.length === 0) problems.push('ruleVersions is empty, so no payout could name a rule');
  if (p.ruleVersions.length > bounds.maxRuleVersions) problems.push(`more than ${bounds.maxRuleVersions} rule versions`);
  if (new Set(p.ruleVersions).size !== p.ruleVersions.length) problems.push('ruleVersions repeats an entry');
  for (const v of p.ruleVersions) if (!RULE_VERSION.test(v)) problems.push(`rule version ${JSON.stringify(v)} is not name@number`);
  return problems;
}

/**
 * Whether moving from `current` to `next` loosens anything. A loosening waits loosenDelayBlocks on chain; a
 * tightening applies at once (ADR-070 decision 6). Adding a rule version loosens; removing one tightens.
 */
export function loosens(current: VaultPolicy, next: VaultPolicy): string[] {
  const out: string[] = [];
  if (next.maxPayout > current.maxPayout) out.push('maxPayout');
  if (next.epochBudget > current.epochBudget) out.push('epochBudget');
  if (next.perRecipientPerEpoch > current.perRecipientPerEpoch) out.push('perRecipientPerEpoch');
  // A shorter epoch refreshes budgets sooner, so more can be paid per block.
  if (next.epochBlocks < current.epochBlocks) out.push('epochBlocks');
  if (next.accrualExpiryBlocks > current.accrualExpiryBlocks) out.push('accrualExpiryBlocks');
  if (next.loosenDelayBlocks < current.loosenDelayBlocks) out.push('loosenDelayBlocks');
  if (next.ruleVersions.some((v) => !current.ruleVersions.includes(v))) out.push('ruleVersions');
  return out;
}
