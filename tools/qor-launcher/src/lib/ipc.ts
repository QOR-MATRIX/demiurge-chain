/**
 * The typed boundary between the UI and the Rust host.
 *
 * Every call into the host goes through this file. Nothing else in the frontend
 * imports `invoke` directly, which means the set of things the UI is able to ask
 * for is the exported surface of this module and can be read in one sitting.
 *
 * Note what is absent: there is no `getPrivateKey`, no `getSeed`, no
 * `exportAccount`. The UI can ask the host to *sign*; it cannot ask for the key.
 * That asymmetry is the point of putting custody in Rust.
 */

import { invoke } from '@tauri-apps/api/core';

/* ────────────────────────────────── types ───────────────────────────────── */

/** Errors from the host arrive as `{ kind, message }`, never as bare strings. */
export interface QorError {
  kind: QorErrorKind;
  message: string;
}

export type QorErrorKind =
  | 'vault_locked'
  | 'no_vault'
  | 'vault_exists'
  | 'vault_open'
  | 'bad_passphrase'
  | 'passphrase_vault'
  | 'bad_mnemonic'
  | 'vault_corrupt'
  | 'not_authenticated'
  | 'auth'
  | 'rpc'
  | 'network'
  | 'bad_address'
  | 'bad_amount'
  | 'insufficient_funds'
  | 'keychain'
  | 'io'
  | 'declined'
  | 'hello'
  | 'qontrol'
  | 'payment_refused'
  | 'internal';

/** An account's level and XP, as QOR ID keeps them (ADR-078). */
export interface Progress {
  level: number;
  xp: number;
  /** Total XP at which the current level began, and the next one begins. */
  level_xp: number;
  next_level_xp: number;
  next_unlock: string | null;
  unlocked: string[];
  tasks: { key: string; name: string; xp: number; done: boolean }[];
  welcome: { amount_cgt: string; status: 'not_yet' | 'owed' | 'paid' };
}

export interface AccountView {
  /** SS58 at the chain's prefix: what a person reads, copies and pastes. */
  address: string;
  /** The same account as raw hex. Advanced views only (ADR-024). */
  account_id: string;
  /** The derivation suffix: empty for the first account, `//1`, `//2`, … after it. */
  path: string;
  index: number;
  label: string;
}

export type VaultStatus =
  | { state: 'absent' }
  /**
   * `keychain` opens with nothing asked (ADR-056). A vault sealed before it moves
   * to the keychain once: `hello` with one last Windows Hello, `passphrase` with
   * its passphrase typed once more.
   */
  | { state: 'locked'; sealed_with: 'keychain' | 'hello' | 'passphrase' }
  | { state: 'unlocked'; accounts: AccountView[] };

/**
 * Where unlocking or creating the vault led (ADR-016). The host signs in to
 * QOR ID as part of the unlock, so the Gate only has to go where this says.
 */
export interface Arrival {
  accounts: AccountView[];
  /** Present when the person is signed in. */
  session: Session | null;
  /** The vault's key has no QOR ID yet: ask for a name. */
  needs_name: boolean;
  /** Why signing in did not happen, when it could not. */
  sign_in_problem: string | null;
}

export interface Session {
  qor_id: string;
  username: string;
  /** Retired with ADR-075: always 1. The QOR ID is the name alone. */
  discriminator: number;
  role: string;
  address: string | null;
  avatar_url: string | null;
}

export interface ChainStatus {
  endpoint: string;
  reachable: boolean;
  /**
   * What the node calls itself, from `system_chain`. Two chains in this
   * repository listen on 9944 and one of them is untrusted, so this is what
   * tells them apart (ADR-040).
   */
  chain_name: string | null;
  block_number: number | null;
  /** The highest finalised block. The chain this replaced had no finality. */
  finalized_number: number | null;
  latency_ms: number | null;
  detail: string | null;
}

/** Static facts about CGT. No supply figure: see `TokenInfo` in `lib.rs`. */
export interface TokenInfo {
  symbol: string;
  name: string;
  decimals: number;
  sub_unit: string;
}

export interface Balance {
  address: string;
  /** Integer Sparks as a decimal string. `u128` does not fit in a JS number. */
  sparks: string;
  cgt: string;
  display: string;
}

export interface TransferReceipt {
  tx_hash: string;
  from: string;
  to: string;
  amount_sparks: string;
  amount_cgt: string;
  /** The finalised block the transfer is in, not merely the one it entered. */
  block_hash: string;
}

export interface HistoryEntry {
  hash: string;
  from: string;
  to: string | null;
  amount_sparks: string | null;
  amount_cgt: string | null;
  nonce: number;
  block_number: number | null;
  direction: 'in' | 'out' | null;
  status: string;
}

export interface ClaimResult {
  success: boolean;
  amount_sparks: string;
  amount_cgt: string;
  message: string;
}

export interface Probe {
  label: string;
  rpc_url: string;
  auth_url: string;
  rpc_ok: boolean;
  auth_ok: boolean;
  detail: string | null;
}

/** Release gates (roadmap L2.2). The host computes every field from docs/GATES.toml. */
export type UnitState = 'met' | 'not_met' | 'unmeasurable';

export interface GateUnit {
  label: string;
  state: UnitState;
  detail: string;
  /** A person asserted it (kind `check`); nothing computed it. */
  asserted: boolean;
}

export interface GateCriterion {
  id: string;
  kind: string;
  says: string | null;
  units: GateUnit[];
}

export interface GateReport {
  id: string;
  name: string;
  means: string;
  met: number;
  not_met: number;
  unmeasurable: number;
  /** Present only when no unit in the gate is unmeasurable. */
  bar: { met: number; total: number } | null;
  passed: boolean;
  criteria: GateCriterion[];
}

export interface TestRun {
  suite: string;
  finished_at: string;
  commit: string | null;
  exit_code: number | null;
  passed: string[];
  failed: string[];
  output_tail: string;
}

export interface SuiteView {
  id: string;
  dir: string;
  command: string;
  needs: string | null;
  running: boolean;
  last_run: TestRun | null;
}

export interface GatesReport {
  repo: string | null;
  status: string | null;
  accepted: string | null;
  problems: string[];
  gates: GateReport[];
  suites: SuiteView[];
}

export interface LauncherState {
  version: string;
  vault: VaultStatus;
  session: Session | null;
  chain_endpoint: string;
  auth_endpoint: string;
  token: TokenInfo;
}

/* ───────────────────────────── error handling ───────────────────────────── */

/** Narrow an unknown catch value to a `QorError`. */
export function asQorError(error: unknown): QorError {
  if (
    typeof error === 'object' &&
    error !== null &&
    'kind' in error &&
    'message' in error
  ) {
    return error as QorError;
  }
  return {
    kind: 'internal',
    message: error instanceof Error ? error.message : String(error),
  };
}

/**
 * Wording shown to the user for each failure kind.
 *
 * The host's messages are accurate but written for a developer reading a log.
 * These are written for someone who is trying to get into their wallet, so each
 * one says what to do next rather than what went wrong internally.
 */
const FRIENDLY: Partial<Record<QorErrorKind, string>> = {
  vault_locked: 'Your vault is locked. Unlock it to continue.',
  no_vault: 'There is no vault on this device yet.',
  vault_exists: 'A vault already exists here. Remove it before creating another.',
  vault_open: 'The vault is open. Lock it before restoring another.',
  bad_passphrase: 'That passphrase is not correct.',
  passphrase_vault:
    'This vault was sealed with a passphrase. Type it once more and it will never be asked again, or restore from your recovery phrase.',
  bad_mnemonic: 'That recovery phrase is not valid. Check the spelling and word order.',
  not_authenticated: 'Your session has expired. Sign in again.',
  network: 'Cannot reach the network. Check your connection.',
  insufficient_funds: 'You do not have enough CGT for that transfer.',
  keychain:
    'This computer’s keychain did not give up the vault’s key. Try again, or restore the vault from your recovery phrase.',
  declined: 'You cancelled, so nothing was signed, changed or opened.',
};

export function explain(error: unknown): string {
  const qor = asQorError(error);
  return FRIENDLY[qor.kind] ?? qor.message;
}

/* ──────────────────────────────── commands ──────────────────────────────── */

const call = <T,>(command: string, args?: Record<string, unknown>): Promise<T> =>
  invoke<T>(command, args);

export const shell = {
  state: () => call<LauncherState>('launcher_state'),
};

export const vault = {
  status: () => call<VaultStatus>('vault_status'),
  /** Returns a phrase for the user to transcribe. Not persisted until create. */
  generatePhrase: () => call<string>('vault_generate_phrase'),
  /** Creates the vault, its key in the keychain, then tries QOR ID (ADR-016, ADR-056). */
  create: (phrase: string) => call<Arrival>('vault_create', { phrase }),
  /** The address a recovery phrase opens. Stores nothing. */
  previewPhrase: (phrase: string) => call<string>('vault_preview_phrase', { phrase }),
  /** Seal a vault from a phrase already held; a vault already here is set aside after a host dialog. */
  restore: (phrase: string) => call<Arrival>('vault_restore', { phrase }),
  /** Opens the vault, then tries QOR ID. Nothing is asked, except one last Windows Hello for a vault sealed under it. */
  unlock: () => call<Arrival>('vault_unlock'),
  /** A passphrase vault from before ADR-055: its passphrase once more, and it moves to the keychain. */
  moveToKeychain: (passphrase: string) =>
    call<Arrival>('vault_move_to_keychain', { passphrase }),
  lock: () => call<void>('vault_lock'),
  addAccount: (label: string) => call<AccountView>('vault_add_account', { label }),
  /** The host asks in its own dialog first. */
  exportPhrase: () => call<string>('vault_export_phrase'),
};

/** A project on this machine, read from its repository. */
export interface QontrolProject {
  path: string;
  name: string;
  branch: string | null;
  changes: { path: string; state: string }[];
  history: { id: string; short: string; summary: string; author: string; time: number }[];
  branches: { name: string; head: boolean }[];
  /** False when the staging helper is missing, so the view can say so. */
  can_commit: boolean;
}

export type ScaffoldKind = 'code' | 'music' | 'game';

/** One line of a file's diff, numbered as the file numbers it. */
export interface DiffLine {
  kind: 'hunk' | 'context' | 'add' | 'remove';
  text: string;
  /** Its number before the change, where it has one. */
  old: number | null;
  /** Its number after the change, where it has one. */
  new: number | null;
}

/**
 * What changed inside one file, read as git reads it: line endings and
 * `.gitattributes` applied first, so a conversion is never shown as a change.
 */
export type DiffBody =
  | { kind: 'text'; lines: DiffLine[]; truncated: boolean }
  | { kind: 'binary'; old_size: number | null; new_size: number | null }
  | { kind: 'same' }
  | { kind: 'empty' }
  | { kind: 'not_a_file'; reason: string };

export interface FileDiff {
  path: string;
  state: string;
  /** Where a moved file came from. */
  from: string | null;
  body: DiffBody;
}

/** Why the guard holds a file back before it is committed. */
export type GuardConcern = 'large' | 'credential';

/**
 * One file the guard holds back until the person says yes to it. `reason` is
 * plain words ("looks like a private key", "72 MiB"); the host never sends the
 * text that matched.
 */
export interface GuardWarning {
  path: string;
  concern: GuardConcern;
  reason: string;
}

/**
 * Qontrol. Everything here reads the repository on disk; the view holds no
 * copy of what it believes the repository contains.
 */
export const qontrol = {
  pickFolder: () => call<string | null>('qontrol_pick_folder'),
  open: (path: string) => call<QontrolProject>('qontrol_open', { path }),
  create: (parent: string, name: string, kind: ScaffoldKind) =>
    call<QontrolProject>('qontrol_create', { parent, name, kind }),
  read: (path: string) => call<QontrolProject>('qontrol_read', { path }),
  diff: (path: string, file?: string) => call<FileDiff[]>('qontrol_diff', { path, file }),
  /** What a commit of `files` (or of everything) would hold that needs a yes. Stages nothing. */
  check: (path: string, files?: string[]) =>
    call<GuardWarning[]>('qontrol_check', { path, files }),
  /**
   * Commit `files`, or everything when none are given. The host runs the guard
   * again and refuses while any warning is missing from `accepted`.
   */
  commit: (
    path: string,
    message: string,
    files?: string[],
    accepted?: { path: string; concern: GuardConcern }[],
  ) => call<QontrolProject>('qontrol_commit', { path, message, files, accepted }),
  /** Make a branch at HEAD. It is not switched to. */
  branch: (path: string, name: string) => call<QontrolProject>('qontrol_branch', { path, name }),
  /** Switch branches; refused, touching nothing, if an uncommitted change would be overwritten. */
  switchTo: (path: string, branch: string) =>
    call<QontrolProject>('qontrol_switch', { path, branch }),
  /** Put modified or removed files back as last committed. */
  discard: (path: string, files: string[]) =>
    call<QontrolProject>('qontrol_discard', { path, files }),
};

/** A content reference, as the host renders it (ADR-047 decision 3). */
export interface ContentView {
  /** `BLAKE3-256`, `SHA-256` or `BLAKE2-256`. */
  algo: string;
  /** The 32-byte root as 64 hex characters. */
  root: string;
  /** Length of what the root hashes, in bytes. */
  size: number;
}

/** A pinned commit: a git object id, never a branch. */
export interface CommitView {
  /** `SHA-1` or `SHA-256`. */
  kind: string;
  id: string;
}

/** One DRC-369 asset an account holds, read by the host from chain storage. */
export interface OwnedAsset {
  collection: number;
  item: number;
  name: string;
  origin: ContentView;
  current: ContentView;
  commit: CommitView | null;
  revisable: boolean;
  /** Its listing, if it is offered for sale, read from the chain with it. */
  listing: ListingView | null;
}

/** An asset's listing on chain: who is selling it, and for how much. */
export interface ListingView {
  seller: string;
  /** Integer Sparks as a decimal string. `u128` does not fit in a JS number. */
  price_sparks: string;
  /** The price in CGT, grouped for reading. Display only. */
  price_cgt: string;
  /** Whoever listed it no longer holds the asset, so nobody can buy from it. */
  void: boolean;
}

/** An asset's royalty terms, as its creator set them on chain. */
export interface TermsView {
  recipients: { address: string; share: string }[];
  /** What a sale of any remix of this asset owes those recipients. */
  remix: string;
}

export type PayoutKind = 'source' | 'royalty' | 'seller';

/** One part of a sale's price, and who receives it. */
export interface Payout {
  kind: PayoutKind;
  address: string;
  /** A royalty recipient's share. Null for the seller and for a source's recipients. */
  share: string | null;
  amount_sparks: string;
  amount_cgt: string;
  /** The recipient is the buyer, so this part never leaves their account. */
  to_buyer: boolean;
}

/**
 * What a sale at one price pays, part by part, in the order the chain pays it.
 * The chain's own answer, which the host asks it for; the view only draws it.
 */
export interface Breakdown {
  price_sparks: string;
  price_cgt: string;
  /** The asset this one was remixed from, when a sale owes it a share. */
  source: TradeItem | null;
  source_share: string | null;
  payouts: Payout[];
  /** Why the chain would refuse a sale at this price as things stand, in words. */
  blocked: string | null;
}

/** One asset, whoever holds it, and everything about selling or buying it. */
export interface SaleView {
  asset: OwnedAsset;
  holder: string;
  held_by_viewer: boolean;
  derived_from: TradeItem | null;
  terms: TermsView | null;
  source_terms: TermsView | null;
  /** Present only while the asset can be bought. */
  breakdown: Breakdown | null;
  /** What the asking account can spend, in CGT. */
  viewer_free_cgt: string | null;
  /** Why the asking account cannot buy it, in words. Null when it can. */
  cannot_buy: string | null;
  /** When a fingerprint was pasted with the number: whether the asset still carries it. */
  pasted_root_matches: boolean | null;
}

export interface ListReceipt {
  collection: number;
  item: number;
  price_sparks: string;
  price_cgt: string;
  tx_hash: string;
  block_hash: string;
}

export interface UnlistReceipt {
  collection: number;
  item: number;
  tx_hash: string;
  block_hash: string;
}

/** A finalised sale, as the chain's own `Sold` event reports it. */
export interface SaleReceipt {
  collection: number;
  item: number;
  name: string;
  seller: string;
  buyer: string;
  price_sparks: string;
  price_cgt: string;
  payouts: Payout[];
  tx_hash: string;
  block_hash: string;
}

export interface MintReceipt {
  collection: number;
  item: number;
  name: string;
  reference: ContentView;
  commit: CommitView;
  tx_hash: string;
  block_hash: string;
}

export interface PermanenceReceipt {
  collection: number;
  item: number;
  tx_hash: string;
  block_hash: string;
}

/** One question a listing category asks. No options means free text. */
export interface ListingField {
  id: string;
  label: string;
  options: string[];
}

/** One kind of thing a listing can be about, and what it asks. */
export interface ListingCategory {
  id: string;
  name: string;
  note: string | null;
  fields: ListingField[];
}

export interface ListingDetail {
  field: string;
  value: string;
}

/**
 * An asset's description, drafted on this machine and published nowhere. The
 * listing a buyer can act on is the price on chain (`sales` below).
 */
export interface Listing {
  collection: number;
  item: number;
  title: string;
  category: string;
  details: ListingDetail[];
  price_cgt: string;
  price_sparks: string;
  notes: string;
  created: number;
  updated: number;
}

/**
 * Drafted descriptions (L4.6). **A draft is not published**: the chain holds a
 * listing's price and nothing else, so the Market has no description to show
 * (M5.4), and a draft never leaves this machine. The vocabulary
 * comes from the host, so the form keeps none of its own.
 */
export const listings = {
  vocabulary: () => call<ListingCategory[]>('listing_vocabulary'),
  drafts: () => call<Listing[]>('listing_drafts'),
  save: (draft: {
    collection: number;
    item: number;
    title: string;
    category: string;
    details: ListingDetail[];
    price_cgt: string;
    notes: string;
  }) => call<Listing>('listing_save', { draft }),
  discard: (collection: number, item: number) =>
    call<void>('listing_discard', { collection, item }),
};

/**
 * Selling and buying, settled on chain in CGT (L4.6, ADR-061).
 *
 * The host reads the listing and the royalty terms from chain storage, works
 * out what a sale pays with the chain's own arithmetic, draws its own dialog
 * and signs. The view sends an asset and a typed price, and draws what comes
 * back. A buyer reaches an asset by its number, or from the Market (`market`
 * below), which lists what the chain holds and has no search.
 */
export const sales = {
  /** One asset by its number, or by the reference its holder copied. */
  find: (asset: string, viewer: string | null) =>
    call<SaleView>('drc369_sale', { asset, viewer }),
  /** What a sale at a typed price would pay. The host parses the price, exactly. */
  preview: (collection: number, item: number, price: string) =>
    call<Breakdown>('drc369_sale_preview', { collection, item, price }),
  /** List an asset, or change its price. The host's dialog says what a sale pays. */
  list: (from: string, collection: number, item: number, price: string) =>
    call<ListReceipt>('drc369_list', { from, collection, item, price }),
  unlist: (from: string, collection: number, item: number) =>
    call<UnlistReceipt>('drc369_unlist', { from, collection, item }),
  /**
   * Buy a listed asset. `priceSparks` and `root` are what was on screen: if
   * either has changed on chain, the host asks nobody and sends nothing.
   */
  buy: (from: string, collection: number, item: number, priceSparks: string, root: string) =>
    call<SaleReceipt>('drc369_buy', { from, collection, item, priceSparks, root }),
};

/** Whose listings the Market shows (`chain/market.rs`'s `Show`). */
export type MarketShow = 'all' | 'others' | 'yours';

/**
 * The order the Market shows listings in (`Order`). There is no "newest": a
 * listing on chain carries no time, so the host offers none.
 */
export type MarketOrder = 'price_low' | 'price_high' | 'number';

/** Whether the account looking can buy a listing, and if not, why (`Standing`). */
export type MarketStanding = 'buyable' | 'yours' | 'void' | 'held_in_place';

/** One listing, as the host read it at one finalised block (`MarketListing`). */
export interface MarketListing {
  /** The asset, with its listing: name, content reference, price and seller. */
  asset: OwnedAsset;
  /** Who holds it now. The seller, unless the listing is void. */
  holder: string;
  held_by_viewer: boolean;
  /** The asset it was remixed from, if its minter declared one. */
  derived_from: TradeItem | null;
  standing: MarketStanding;
  /** Why it cannot be bought, in words. Null when it can. */
  reason: string | null;
}

/**
 * One look at the Market (`MarketPage`). Every count is the host's: the view
 * draws them and works none of them out.
 */
export interface MarketPage {
  /** The window that was asked for, in the host's order. */
  listings: MarketListing[];
  /** Where the window starts among the listings that match. */
  offset: number;
  /** How many listings match what was asked for, among those read. */
  matching: number;
  /** How many listings the walk read from the chain, whoever made them. */
  on_chain: number;
  /** How many of those the account looking made. */
  yours: number;
  /** The chain holds more listings than `bound`; the rest were not read. */
  truncated: boolean;
  /** The most listings one look reads. */
  bound: number;
  /** Listings in this window whose asset could not be read, left out of `listings`. */
  unreadable: number;
  /** The finalised block all of it was read at. */
  block_number: number;
  block_hash: string;
  /** The node it was read through, and what that node calls its chain. */
  endpoint: string;
  chain_name: string;
}

/**
 * How many listings the Market asks for at a time. The host's own default is
 * the same number (`market::PAGE`); it is sent explicitly so the view's pages
 * and the host's windows cannot disagree.
 */
export const MARKET_PAGE = 24;

/**
 * The Market (L7.2's first slice). There is no indexer (ADR-028), so the host
 * walks the chain's listing storage through the connected node at its latest
 * finalised block, bounded, and reads details only for the window asked for.
 * It reads and signs nothing; buying goes through `sales.buy` like every
 * other purchase.
 */
export const market = {
  page: (
    viewer: string | null,
    show: MarketShow,
    order: MarketOrder,
    offset: number,
    limit: number = MARKET_PAGE,
  ) => call<MarketPage>('drc369_market', { viewer, show, order, offset, limit }),
};

/** One account this machine has traded with. Local, and never sent anywhere. */
export interface TradePartner {
  address: string;
  label: string | null;
  last_traded: number;
  trades: number;
}

/** One asset in a trade. */
export interface TradeItem {
  collection: number;
  item: number;
}

export interface TradeReceipt {
  moved: TradeItem[];
  /** The recipient as this chain writes an address, not as it was typed. */
  to: string;
  tx_hash: string;
  block_hash: string;
}

/**
 * The most assets one trade carries, and the longest message it may hold. Both
 * are the host's limits (`chain/assets.rs`), repeated here only so the view can
 * stop a person before the host has to refuse them.
 */
export const TRADE_LIMIT = 16;
export const MESSAGE_LIMIT = 256;

/**
 * DRC-369 (M4.1). The host computes the reference, draws the dialog and signs;
 * the view sends a project path and an account, and draws what comes back.
 */
export const assets = {
  /** Mint the commit the project's HEAD points at. */
  mint: (path: string, from: string) => call<MintReceipt>('qontrol_mint', { path, from }),
  /** What an account holds, read from chain storage every time. */
  of: (address: string) => call<OwnedAsset[]>('drc369_assets', { address }),
  /** One-way. The host's dialog says so before anything is signed. */
  makePermanent: (from: string, collection: number, item: number) =>
    call<PermanenceReceipt>('drc369_make_permanent', { from, collection, item }),
  /**
   * Send assets to another account, all in one transaction (L4.5, ADR-053).
   * Irreversible, which the host's dialog says before anything is signed.
   */
  trade: (
    from: string,
    to: string,
    items: TradeItem[],
    message: string | null,
    /** The QOR ID the recipient was reached by, when it was not an address. */
    label: string | null,
  ) => call<TradeReceipt>('drc369_trade', { from, to, items, message, label }),
  /** Who this machine has traded with, most recent first. */
  partners: () => call<TradePartner[]>('trade_partners'),
};

export const chain = {
  status: () => call<ChainStatus>('chain_status'),
  setEndpoint: (endpoint: string) => call<void>('chain_set_endpoint', { endpoint }),
  balance: (address: string) => call<Balance>('cgt_balance', { address }),
  /** Parse a typed amount in the host so denomination logic lives in one place. */
  parseAmount: (amount: string) => call<Balance>('cgt_parse_amount', { amount }),
  send: (from: string, to: string, amount: string) =>
    call<TransferReceipt>('cgt_send', { from, to, amount }),
  /** Non-zero proves the account has signed something the chain accepted. */
  nonce: (address: string) => call<number>('cgt_nonce', { address }),
  claimStarter: (address: string) => call<ClaimResult>('cgt_claim_starter', { address }),
  history: (address: string, limit = 50) =>
    call<HistoryEntry[]>('cgt_history', { address, limit }),
};

/** Endpoint discovery and selection, usable before sign-in. */
export const net = {
  probe: () => call<Probe[]>('probe_endpoints'),
  use: (rpcUrl: string, authUrl: string) =>
    call<void>('use_endpoints', { rpcUrl, authUrl }),
};

export const identity = {
  login: (identifier: string, password: string) =>
    call<Session>('qor_login', { identifier, password }),
  loginWithKey: (address: string) => call<Session>('qor_login_with_key', { address }),
  registerWithKey: (address: string, username?: string) =>
    call<Session>('qor_register_with_key', { address, username: username ?? null }),
  usernameAvailable: (username: string) =>
    call<boolean>('qor_username_available', { username }),
  linkWallet: (address: string) => call<Session>('qor_link_wallet', { address }),
  restore: () => call<Session>('qor_restore'),
  /** Level, XP, tasks and next unlock, kept by QOR ID (ADR-078). */
  progress: () => call<Progress>('qor_progress'),
  /** The tutorial is finished: XP once. */
  tutorialDone: () => call<Progress>('qor_tutorial_done'),
  /** Try QOR ID again with the open vault, as opening it does. */
  signIn: () => call<Arrival>('qor_sign_in'),
  logout: () => call<void>('qor_logout'),
  setEndpoint: (endpoint: string) => call<void>('qor_set_auth_endpoint', { endpoint }),
};

/** The development dashboard (roadmap L2.2). */
export const gates = {
  /** `refresh` bypasses the host's short caches on CI and service readings. */
  report: (refresh = false) => call<GatesReport>('gates_report', { refresh }),
  /** The host asks for approval in its own dialog before anything runs. */
  runSuite: (suite: string) => call<TestRun>('gates_run_suite', { suite }),
};

/* ───────────────────────────────── helpers ──────────────────────────────── */

/**
 * Whether a string is the right *shape* for an address: SS58, or hex from an
 * advanced view (ADR-024).
 *
 * Shape only. The checksum and the network's prefix are checked in the host,
 * which is the authority on both, so this never decides that an address is
 * good — only that it is not yet worth sending.
 */
export function looksLikeAddress(value: string): boolean {
  const trimmed = value.trim();
  const hex = /^(0x)?[0-9a-fA-F]{64}$/;
  // Base58 has no 0, O, I or l. A 32-byte account at a one- or two-byte prefix
  // encodes to 47 to 49 characters.
  const ss58 = /^[1-9A-HJ-NP-Za-km-z]{46,50}$/;
  return ss58.test(trimmed) || hex.test(trimmed);
}

/** `0x1234…cdef` for display. Full values stay available for copying. */
export function shortAddress(address: string, lead = 6, tail = 4): string {
  if (address.length <= lead + tail + 2) return address;
  return `${address.slice(0, lead)}…${address.slice(-tail)}`;
}

/** `mm:ss` for the auto-lock countdown. */
export function countdown(seconds: number): string {
  const safe = Math.max(0, Math.floor(seconds));
  const m = Math.floor(safe / 60);
  const s = safe % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}
