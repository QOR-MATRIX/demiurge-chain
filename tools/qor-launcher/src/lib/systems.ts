/**
 * The ecosystem registry.
 *
 * Every Demiurge system the launcher can reach, with an honest account of what
 * each one currently is. This is the data behind the Nexus.
 *
 * # On honesty in the registry
 *
 * It is tempting to list everything as available and let the user discover the
 * gaps. That produces a launcher that lies, and a community that stops trusting
 * the status indicator entirely. Each entry therefore carries a real `state`,
 * and the Nexus renders the three states differently rather than uniformly:
 *
 *   live     you can open it right now
 *   local    it exists and runs, but needs its dev server started
 *   forming  specified, not built
 *
 * A `forming` tile says what it is waiting on. That is more motivating to a
 * developer audience than a fake "coming soon" badge, and it doubles as a
 * roadmap that cannot drift from reality, because it sits next to the thing it
 * describes.
 */

export type SystemState = 'live' | 'local' | 'forming';

export type Domain = 'create' | 'play' | 'exchange' | 'commune' | 'forge' | 'operate';

export interface System {
  id: string;
  name: string;
  /** One line. What it is, not what it aspires to. */
  tagline: string;
  domain: Domain;
  state: SystemState;
  /** Internal launcher surface, if this opens in-app. */
  surface?: string;
  /** External URL, if this opens outside. */
  url?: string;
  /** Local development URL, for `local` systems. */
  localUrl?: string;
  /** For `forming`: the one thing blocking it. */
  blockedOn?: string;
  /** Glyph key, resolved to an icon in the view. */
  glyph: string;
}

export const DOMAINS: Record<Domain, { name: string; note: string }> = {
  create: { name: 'Create', note: 'Make things that did not exist' },
  play: { name: 'Play', note: 'Worlds, games, and the engines under them' },
  exchange: { name: 'Exchange', note: 'Value, assets, and who owns what' },
  commune: { name: 'Commune', note: 'The people you do all of it with' },
  forge: { name: 'Forge', note: 'Agents, SDKs, and the machinery beneath' },
  operate: { name: 'Operate', note: 'The chain itself, and your place in it' },
};

export const SYSTEMS: System[] = [
  // ── Operate ──────────────────────────────────────────────────────────────
  {
    id: 'vault',
    name: 'CGT Vault',
    tagline: 'Hold, send and receive the Creator God Token.',
    domain: 'exchange',
    state: 'live',
    surface: 'vault',
    glyph: 'wallet',
  },
  {
    id: 'chain',
    name: 'Chain',
    tagline: 'Node health, block production, and where you are pointed.',
    domain: 'operate',
    state: 'live',
    surface: 'chain',
    glyph: 'radio',
  },
  {
    id: 'inventory',
    name: 'Inventory',
    tagline: 'DRC-369 assets you own, with their state and history.',
    domain: 'exchange',
    state: 'live',
    surface: 'inventory',
    glyph: 'boxes',
  },
  {
    id: 'market',
    name: 'Market',
    tagline: 'What is listed for sale on chain, read through your node. No search yet.',
    domain: 'exchange',
    state: 'live',
    surface: 'market',
    glyph: 'store',
  },
  {
    id: 'explorer',
    name: 'Explorer',
    tagline: 'Blocks, transactions and accounts, as they are written.',
    domain: 'operate',
    state: 'local',
    url: 'https://demiurge.cloud/explorer',
    localUrl: 'http://localhost:3000/explorer',
    glyph: 'search',
  },
  {
    id: 'gates',
    name: 'Release gates',
    tagline: 'Alpha, Beta and Public Release, measured from the signals docs/GATES.toml defines.',
    domain: 'operate',
    state: 'live',
    surface: 'gates',
    glyph: 'checks',
  },
  {
    id: 'staking',
    name: 'Staking',
    tagline: 'Become an Archon, or nominate one as an Aeon.',
    domain: 'operate',
    state: 'forming',
    blockedOn: 'Staking writes to a counter, not to the consensus validator set.',
    glyph: 'layers',
  },

  // ── Create ───────────────────────────────────────────────────────────────
  {
    id: 'studio',
    name: 'DRC-369 Studio',
    tagline: 'Mint assets that carry state, physics and memory.',
    domain: 'create',
    state: 'local',
    url: 'https://demiurge.cloud/create',
    localUrl: 'http://localhost:4001',
    glyph: 'sparkles',
  },
  {
    id: 'sophia',
    name: 'Sophia',
    tagline: 'The ecosystem intelligence. Ask it anything on-chain.',
    domain: 'create',
    state: 'local',
    url: 'https://demiurge.cloud/sophia',
    localUrl: 'http://localhost:3003',
    glyph: 'brain',
  },
  {
    id: 'scatter',
    name: 'Scatter',
    tagline: '3D and generative scene tooling for on-chain worlds.',
    domain: 'create',
    state: 'local',
    url: 'https://demiurge.cloud/scatter3d',
    localUrl: 'http://localhost:3000/scatter3d',
    glyph: 'shapes',
  },
  {
    id: 'music',
    name: 'Resonance',
    tagline: 'Release, stream and earn on sound.',
    domain: 'create',
    state: 'forming',
    blockedOn: 'The music service does not compile against its own schema.',
    glyph: 'music',
  },

  // ── Play ─────────────────────────────────────────────────────────────────
  {
    id: 'library',
    name: 'Library',
    tagline: 'Install, patch and launch, with entitlements read from chain.',
    domain: 'play',
    state: 'forming',
    blockedOn: 'No content addressing on chain, so downloads cannot be verified.',
    glyph: 'library',
  },
  {
    id: 'worlds',
    name: 'Worlds',
    tagline: 'The Unreal client and the persistent spaces it opens onto.',
    domain: 'play',
    state: 'forming',
    blockedOn: 'The UE5 client has no content assets and no vendored websocket plugin.',
    glyph: 'globe',
  },

  // ── Commune ──────────────────────────────────────────────────────────────
  {
    id: 'social',
    name: 'Social',
    tagline: 'Rooms, messages, presence, profiles and feeds under one QOR ID.',
    domain: 'commune',
    state: 'forming',
    blockedOn: 'The launcher needs a persistent socket; the node publishes to none.',
    glyph: 'messages',
  },
  {
    id: 'mesh',
    name: 'Mesh',
    tagline: 'Community-seeded distribution. The network gets faster as it grows.',
    domain: 'commune',
    state: 'forming',
    blockedOn: 'Seeder rewards need settled transfers and on-chain manifests.',
    glyph: 'share',
  },

  // ── Forge ────────────────────────────────────────────────────────────────
  {
    id: 'agents',
    name: 'Agent Foundry',
    tagline: 'Deploy agents as first-class citizens with their own keys.',
    domain: 'forge',
    state: 'local',
    url: 'https://demiurge.cloud/agents',
    localUrl: 'http://localhost:3000/agents',
    glyph: 'bot',
  },
  {
    id: 'developers',
    name: 'Developers',
    tagline: 'SDKs, the RPC surface, and how to build against all of it.',
    domain: 'forge',
    state: 'local',
    url: 'https://demiurge.cloud/developers',
    localUrl: 'http://localhost:3000/developers',
    glyph: 'code',
  },
  {
    id: 'bounties',
    name: 'Bounties',
    tagline: 'Paid work posted by the community, settled in CGT.',
    domain: 'forge',
    state: 'local',
    url: 'https://demiurge.cloud/bounties',
    localUrl: 'http://localhost:3000/bounties',
    glyph: 'target',
  },
];

/** Systems grouped by domain, in display order. */
export function byDomain(): { domain: Domain; systems: System[] }[] {
  const order: Domain[] = ['exchange', 'create', 'play', 'commune', 'forge', 'operate'];
  return order
    .map((domain) => ({ domain, systems: SYSTEMS.filter((s) => s.domain === domain) }))
    .filter((group) => group.systems.length > 0);
}

/** Case-insensitive search across name, tagline and domain. */
export function searchSystems(query: string): System[] {
  const q = query.trim().toLowerCase();
  if (!q) return SYSTEMS;

  return SYSTEMS.filter((s) =>
    [s.name, s.tagline, DOMAINS[s.domain].name].some((field) =>
      field.toLowerCase().includes(q),
    ),
  );
}

export const STATE_LABEL: Record<SystemState, string> = {
  live: 'Ready',
  local: 'Connect',
  forming: 'Forming',
};
