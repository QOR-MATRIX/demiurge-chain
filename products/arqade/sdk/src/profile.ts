// A project's profile: the manifest a Cartridge carries, which ARQADE renders as the game's store page, and the
// requirements each stage must meet before it publishes itself (ADR-071, accepted 4 October 2026). Nothing here touches the chain.

import { checkSparks, SPARKS_PER_CGT } from './amount';

export const PROFILE_SCHEMA = 'arqade.project/1';

/** The owner's ceiling on a game's price, 4 October 2026 (ADR-071 decision 1). Free is allowed. */
export const MAX_GAME_PRICE = BigInt(10000) * SPARKS_PER_CGT;

export const STAGES = ['concept', 'prototype', 'early-access', 'released'] as const;
export type Stage = (typeof STAGES)[number];

/** Media by ADR-047 fingerprint once a store exists, or an external https URL meanwhile (ADR-071 decision 9). */
export type MediaRef = { url?: string; fingerprint?: string; alt: string };

export type Price = { kind: 'free' } | { kind: 'fixed'; sparks: string };

export type ItemOffer = { id: string; name: string; description: string; sparks: string; editionCap?: number };

export type ProjectProfile = {
  schema: string;
  name: string;
  slug: string;
  pitch: string;
  description: string;
  genres: string[];
  creator: { qorSub: string; displayName: string };
  stage: Stage;
  art: { direction: string; cover?: MediaRef; screenshots: MediaRef[] };
  build?: { hash: string; entry: string; sizeBytes: number; controls: string[]; permanent: boolean };
  contentRating?: 'everyone' | 'teen' | 'mature';
  ruleVersions?: string[];
  price?: Price;
  support?: { url: string };
  sandbox?: { passedForBuild: string };
  items?: ItemOffer[];
};

const SLUG = /^[a-z0-9](?:[a-z0-9-]{1,38}[a-z0-9])$/;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const HASH = /^[0-9a-f]{64}$/;
const RULE_VERSION = /^[a-z0-9][a-z0-9._-]{0,47}@[0-9]{1,9}$/;
const ITEM_ID = /^[a-z0-9][a-z0-9-]{0,47}$/;

function httpsUrl(s: string | undefined): boolean {
  if (!s) return false;
  try {
    return new URL(s).protocol === 'https:';
  } catch {
    return false;
  }
}

function mediaOk(m: MediaRef | undefined): boolean {
  return !!m && m.alt.trim().length > 0 && (httpsUrl(m.url) || (!!m.fingerprint && /^[0-9a-f]{82}$/.test(m.fingerprint)));
}

/** A game's price, refused above the ceiling or with excess precision. `null` if it is acceptable. */
export function priceProblem(price: Price | undefined): string | null {
  if (!price) return 'no price set (free is a price)';
  if (price.kind === 'free') return null;
  if (!/^(0|[1-9][0-9]*)$/.test(price.sparks)) return 'price is not a whole number of Sparks';
  const sparks = BigInt(price.sparks);
  if (sparks === BigInt(0)) return 'a fixed price of zero: use free';
  if (sparks > MAX_GAME_PRICE) return 'price is above 10,000 CGT';
  return null;
}

const REQUIREMENTS: Record<Stage, (p: ProjectProfile) => string[]> = {
  concept: (p) => {
    const m: string[] = [];
    if (p.schema !== PROFILE_SCHEMA) m.push(`schema must be ${PROFILE_SCHEMA}`);
    if (p.name.trim().length < 2 || p.name.length > 60) m.push('a name of 2 to 60 characters');
    if (!SLUG.test(p.slug)) m.push('a slug: 3 to 40 lower-case letters, digits and hyphens');
    if (p.pitch.trim().length < 10 || p.pitch.length > 140) m.push('a pitch of 10 to 140 characters');
    if (p.description.trim().length < 80) m.push('a description of at least 80 characters');
    if (p.genres.length < 1 || p.genres.length > 3) m.push('one to three genres');
    if (!UUID.test(p.creator.qorSub) || /^[0-]+$/.test(p.creator.qorSub)) m.push("the creator's QOR ID (its account id)");
    if (!p.creator.displayName.trim()) m.push("the creator's display name");
    if (p.art.direction.trim().length < 20) m.push('an art direction of at least 20 characters');
    return m;
  },
  prototype: (p) => {
    const m: string[] = [];
    if (!p.build) return ['a playable build'];
    if (!HASH.test(p.build.hash)) m.push("the build's hash (BLAKE3-256, 64 hex characters)");
    if (!p.build.entry.trim()) m.push("the build's entry file");
    if (!Number.isSafeInteger(p.build.sizeBytes) || p.build.sizeBytes <= 0) m.push("the build's size in bytes");
    if (p.build.controls.length === 0) m.push('how the game is controlled');
    if (!p.contentRating) m.push('a content rating');
    if (!mediaOk(p.art.cover)) m.push('a cover image with alt text');
    return m;
  },
  'early-access': (p) => {
    const m: string[] = [];
    if (!p.ruleVersions?.length || !p.ruleVersions.every((v) => RULE_VERSION.test(v))) m.push('rule versions as name@number');
    const pp = priceProblem(p.price);
    if (pp) m.push(pp);
    if (!httpsUrl(p.support?.url)) m.push('a support page (https)');
    if (!p.build || p.sandbox?.passedForBuild !== p.build.hash) m.push('sandbox checks passed for this exact build');
    if (p.art.screenshots.filter(mediaOk).length < 3) m.push('three screenshots with alt text');
    return m;
  },
  released: (p) => {
    const m: string[] = [];
    if (!p.build?.permanent) m.push('a permanent build');
    const ids = new Set<string>();
    for (const item of p.items ?? []) {
      if (!ITEM_ID.test(item.id) || ids.has(item.id)) m.push(`item ${JSON.stringify(item.id)}: a unique id`);
      ids.add(item.id);
      if (!item.name.trim() || !item.description.trim()) m.push(`item ${item.id}: a name and a description`);
      if (!/^[1-9][0-9]*$/.test(item.sparks)) m.push(`item ${item.id}: a price in whole Sparks above zero`);
      else {
        try {
          checkSparks(BigInt(item.sparks));
        } catch {
          m.push(`item ${item.id}: a price within u128`);
        }
      }
      if (item.editionCap !== undefined && (!Number.isSafeInteger(item.editionCap) || item.editionCap <= 0)) {
        m.push(`item ${item.id}: an edition cap above zero, or none`);
      }
    }
    return m;
  },
};

/** What is missing before `target` can publish: every earlier stage's requirements, then its own, labelled. */
export function checkReadiness(p: ProjectProfile, target: Stage): { stage: Stage; missing: string }[] {
  const out: { stage: Stage; missing: string }[] = [];
  for (const stage of STAGES.slice(0, STAGES.indexOf(target) + 1)) {
    for (const missing of REQUIREMENTS[stage](p)) out.push({ stage, missing });
  }
  return out;
}

/** The furthest stage the profile already meets, or null if not even a concept. */
export function readyStage(p: ProjectProfile): Stage | null {
  let reached: Stage | null = null;
  for (const stage of STAGES) {
    if (REQUIREMENTS[stage](p).length > 0) break;
    reached = stage;
  }
  return reached;
}
