// ARQ Wallets from a game's side (ADR-070): where a game's wallet lives, the ids that make a payout happen
// once, and the exact arguments the chain's calls take. Nothing here signs or holds a key: a game server's
// payout authority signs with its own key, and a developer's governor calls are signed in their own Vault.

import { blake2b } from '@noble/hashes/blake2.js';
import { checkSparks } from './amount';
import type { ArqWalletPolicy } from './arq-wallet-policy';

/** `PalletId(*b"dmg/arqw")`, in `chain/runtime/src/assets.rs`. Changing it would move every wallet. */
export const ARQ_WALLET_PALLET_ID = 'dmg/arqw';
/** The SS58 prefix of Demiurge's networks until mainnet (ADR-024). */
export const SS58_PREFIX = 42;

const encoder = new TextEncoder();

function u32le(n: number): Uint8Array {
  if (!Number.isInteger(n) || n < 0 || n > 0xffffffff) throw new RangeError(`not a u32: ${n}`);
  return Uint8Array.of(n & 0xff, (n >>> 8) & 0xff, (n >>> 16) & 0xff, (n >>> 24) & 0xff);
}

/**
 * The 32 bytes of a Cartridge's wallet account: `"modl" ++ PalletId ++ SCALE((collection, item))`, zero-padded,
 * which is `PalletId::into_sub_account_truncating`. The chain's `account_of` is pinned to the same bytes.
 */
export function arqWalletAccountId(collection: number, item: number): Uint8Array {
  const out = new Uint8Array(32);
  out.set(encoder.encode('modl'), 0);
  out.set(encoder.encode(ARQ_WALLET_PALLET_ID), 4);
  out.set(u32le(collection), 12);
  out.set(u32le(item), 16);
  return out;
}

const BASE58 = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

function base58(bytes: Uint8Array): string {
  let n = BigInt('0x' + Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('') || '0');
  let out = '';
  const fiftyEight = BigInt(58);
  while (n > BigInt(0)) {
    out = BASE58[Number(n % fiftyEight)] + out;
    n /= fiftyEight;
  }
  for (const b of bytes) {
    if (b !== 0) break;
    out = '1' + out;
  }
  return out;
}

/** An account's SS58 address (simple prefixes, 0 to 63). */
export function ss58(accountId: Uint8Array, prefix = SS58_PREFIX): string {
  if (accountId.length !== 32) throw new RangeError('an account id is 32 bytes');
  if (!Number.isInteger(prefix) || prefix < 0 || prefix > 63) throw new RangeError('only simple SS58 prefixes');
  const body = new Uint8Array(33);
  body[0] = prefix;
  body.set(accountId, 1);
  const preimage = new Uint8Array(7 + 33);
  preimage.set(encoder.encode('SS58PRE'), 0);
  preimage.set(body, 7);
  const checksum = blake2b(preimage, { dkLen: 64 }).subarray(0, 2);
  const full = new Uint8Array(35);
  full.set(body, 0);
  full.set(checksum, 33);
  return base58(full);
}

/** The SS58 address of a Cartridge's ARQ Wallet, created or not. */
export function arqWalletAddress(collection: number, item: number): string {
  return ss58(arqWalletAccountId(collection, item));
}

function hex(bytes: Uint8Array): string {
  return '0x' + Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

function domainId(domain: string, game: string, parts: string[]): string {
  if (!game || parts.length === 0 || parts.some((p) => !p)) throw new RangeError('a game and at least one non-empty part');
  // Length-prefixed, so ("ab","c") and ("a","bc") can never collide.
  const fields = [domain, game, ...parts].map((f) => `${f.length}:${f}`).join('|');
  return hex(blake2b(encoder.encode(fields), { dkLen: 32 }));
}

/**
 * The outcome id of a payout: the same inputs always give the same 32 bytes, so a server that retries after a
 * timeout names the same outcome and the chain pays it once (ADR-070 decision 7). Derive it from what makes the
 * outcome unique in your game, such as a match id and the winner's seat.
 */
export function outcomeId(game: string, ...parts: string[]): string {
  return domainId('arqade:outcome:v1', game, parts);
}

/** A round's id, derived the same way, in its own domain so it can never equal an outcome id. */
export function roundId(game: string, ...parts: string[]): string {
  return domainId('arqade:round:v1', game, parts);
}

/** A policy as the chain's `create` and `set_policy` take it: amounts as decimal strings of Sparks. */
export function toChainPolicy(p: ArqWalletPolicy) {
  return {
    maxPayout: checkSparks(p.maxPayout).toString(),
    epochBlocks: p.epochBlocks,
    epochBudget: checkSparks(p.epochBudget).toString(),
    perRecipientPerEpoch: checkSparks(p.perRecipientPerEpoch).toString(),
    ruleVersions: [...p.ruleVersions],
    accrualExpiry: p.accrualExpiryBlocks,
    loosenDelay: p.loosenDelayBlocks,
  };
}

export type Payout = {
  collection: number;
  item: number;
  outcome: string;
  issuedAt: number;
  ruleVersion: string;
  to: string;
  amount: bigint;
};

/** `payout`'s arguments, in the chain's order, checked: for `api.tx.arqWallet.payout(...args)` on a server. */
export function payoutArgs(p: Payout): [number, number, string, number, string, string, string] {
  if (!/^0x[0-9a-f]{64}$/.test(p.outcome)) throw new RangeError('an outcome id is 32 bytes of hex; use outcomeId()');
  if (!Number.isInteger(p.issuedAt) || p.issuedAt < 0) throw new RangeError('issuedAt is a block number');
  if (checkSparks(p.amount) === BigInt(0)) throw new RangeError('a payout of nothing');
  u32le(p.collection);
  u32le(p.item);
  return [p.collection, p.item, p.outcome, p.issuedAt, p.ruleVersion, p.to, p.amount.toString()];
}

function unbase58(text: string): Uint8Array {
  let n = BigInt(0);
  const fiftyEight = BigInt(58);
  for (const ch of text) {
    const v = BASE58.indexOf(ch);
    if (v < 0) throw new RangeError('not base58');
    n = n * fiftyEight + BigInt(v);
  }
  const hexed = n.toString(16);
  const body = hexed === '0' ? '' : hexed.length % 2 ? '0' + hexed : hexed;
  const bytes = Uint8Array.from(body.match(/../g) ?? [], (h) => parseInt(h, 16));
  let zeros = 0;
  while (zeros < text.length && text[zeros] === '1') zeros++;
  const out = new Uint8Array(zeros + bytes.length);
  out.set(bytes, zeros);
  return out;
}

/**
 * An SS58 address's 32-byte account id, refused unless its checksum holds and its prefix is `prefix`
 * (42 on Demiurge's networks until mainnet, ADR-024). An address for another network is not quietly accepted.
 */
export function ss58Decode(address: string, prefix = SS58_PREFIX): Uint8Array {
  if (typeof address !== 'string' || address.length < 46 || address.length > 50) throw new RangeError('Not an SS58 address');
  const bytes = unbase58(address);
  if (bytes.length !== 35) throw new RangeError('Not a 32-byte SS58 address');
  if (bytes[0] !== prefix) throw new RangeError(`An address for another network (prefix ${bytes[0]}, expected ${prefix})`);
  const id = bytes.subarray(1, 33);
  if (ss58(id, prefix) !== address) throw new RangeError('The address checksum does not hold');
  return id.slice();
}
