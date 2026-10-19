/**
 * The Ascent: the launcher's progression track.
 *
 * # Why this is not XP
 *
 * The easy version of a gamified launcher awards points for opening it. That is
 * transparent to anyone worth keeping, and it trains people to ignore the
 * progression entirely because none of it means anything.
 *
 * Every rite below is instead a **claim about the world that the launcher can
 * verify**: a vault file exists, an account has a non-zero nonce, the chain
 * answered. Nothing is stored, nothing is awarded, and nothing can be gamed
 * without actually doing the thing. The progression is a readout of your real
 * standing in the ecosystem, which is the only kind that stays interesting past
 * the first week.
 *
 * It doubles as onboarding. The next incomplete rite is always the next useful
 * thing to do, so a new arrival is never staring at a launcher wondering where
 * to start.
 */

import type { AccountView, Balance, ChainStatus, Session, VaultStatus } from './ipc';

export type RiteId =
  | 'vault'
  | 'name'
  | 'bound'
  | 'connected'
  | 'funded'
  | 'spent'
  | 'sovereign';

export interface Rite {
  id: RiteId;
  name: string;
  /** What it means, in the user's terms. */
  meaning: string;
  /** How to complete it, shown when incomplete. */
  how: string;
  done: boolean;
}

export interface AscentState {
  rites: Rite[];
  completed: number;
  total: number;
  /** The next thing worth doing, or null when every rite is done. */
  next: Rite | null;
  /** Title earned at the current count. */
  standing: string;
}

/**
 * Titles by rites completed. Gnostic, per the project's naming convention:
 * an Aeon participates, an Archon holds power, the Demiurge makes worlds.
 */
const STANDINGS = [
  'Unnamed', // 0
  'Initiate', // 1
  'Seeker', // 2
  'Aeon', // 3
  'Adept', // 4
  'Archon', // 5
  'Sovereign', // 6
  'Demiurge', // 7
];

export interface AscentInputs {
  vaultStatus: VaultStatus;
  session: Session | null;
  accounts: AccountView[];
  balances: Record<string, Balance>;
  chainStatus: ChainStatus | null;
  /** Request nonce of the active account. Non-zero means it has signed before. */
  activeNonce: number | null;
  activeAddress: string | null;
}

export function computeAscent(input: AscentInputs): AscentState {
  const {
    vaultStatus,
    session,
    accounts,
    balances,
    chainStatus,
    activeNonce,
    activeAddress,
  } = input;

  const vaultExists = vaultStatus.state !== 'absent';

  // "Bound" means the QOR ID actually carries the vault's address, so the
  // identity and the money are provably the same person.
  const bound =
    !!session?.address &&
    accounts.some(
      (a) => a.address.toLowerCase() === session.address!.toLowerCase(),
    );

  const holding = activeAddress
    ? BigInt(balances[activeAddress]?.embers ?? '0') > 0n
    : Object.values(balances).some((b) => BigInt(b.embers) > 0n);

  // A node the launcher is running itself, rather than someone else's.
  const selfHosted =
    !!chainStatus?.reachable &&
    /(127\.0\.0\.1|localhost)/.test(chainStatus.endpoint);

  const rites: Rite[] = [
    {
      id: 'vault',
      name: 'Seal the Vault',
      meaning: 'Your keys exist on this device, and open only for you.',
      how: 'Create a vault and write down the recovery phrase.',
      done: vaultExists,
    },
    {
      id: 'name',
      name: 'Take a Name',
      meaning: 'A QOR ID that carries across every Demiurge world.',
      how: 'Claim a QOR ID from the sign-in screen.',
      done: !!session,
    },
    {
      id: 'connected',
      name: 'Reach the Chain',
      meaning: 'The launcher is talking to a live node.',
      how: 'Point the launcher at a reachable node from the Chain surface.',
      done: !!chainStatus?.reachable,
    },
    {
      id: 'bound',
      name: 'Bind the Two',
      meaning: 'Your name and your keys are provably the same person.',
      how: 'Link your vault address to your QOR ID.',
      done: bound,
    },
    {
      id: 'funded',
      name: 'Hold CGT',
      meaning: 'You are holding CGT.',
      how: 'Receive CGT from someone: on the devnet, test CGT is sent to testers by hand.',
      done: holding,
    },
    {
      id: 'spent',
      name: 'Move Value',
      meaning: 'You have signed a transaction that settled in a block.',
      how: 'Send CGT to another account from the Vault.',
      done: (activeNonce ?? 0) > 0,
    },
    {
      id: 'sovereign',
      name: 'Run the Node',
      meaning: 'You are not trusting anyone else for your view of the chain.',
      how: 'Start a local node and point the launcher at 127.0.0.1.',
      done: selfHosted,
    },
  ];

  const completed = rites.filter((r) => r.done).length;

  return {
    rites,
    completed,
    total: rites.length,
    next: rites.find((r) => !r.done) ?? null,
    standing: STANDINGS[Math.min(completed, STANDINGS.length - 1)]!,
  };
}
