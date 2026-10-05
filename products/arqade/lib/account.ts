// One account's CGT and DRC-369 assets on Demiurge Devnet, read-only, at the finalized block (P7.2).
//
// The balance is `System::Account` from storage; the assets are the chain's own answer to the runtime API
// `Drc369Api::assets_of`. Both are decoded here exactly as the runtime encodes them (SCALE), with every length
// and bound checked; tests pin the layouts. Amounts stay integer Sparks (bigint) and leave as decimal strings.
// Nothing here signs, and the network is checked by its genesis before anything is read.

import { blake2b } from '@noble/hashes/blake2.js';
import { EXISTENTIAL_DEPOSIT } from '../sdk/src/amount';
import { ss58Decode } from '../sdk/src/arq-wallet';
import { readChain, type Rpc } from './chain';

/** `twox128("System") ++ twox128("Account")`: a fixed prefix of every Substrate chain. */
const SYSTEM_ACCOUNT_PREFIX = '26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9';
/** The most assets one answer carries; the count says how many there are. */
export const MAX_ASSETS_SHOWN = 100;

const toHex = (b: Uint8Array) => Array.from(b, (x) => x.toString(16).padStart(2, '0')).join('');

function fromHex(h: unknown): Uint8Array {
  if (typeof h !== 'string' || !/^0x([0-9a-f]{2})*$/i.test(h)) throw Error('Invalid chain response');
  return Uint8Array.from(h.slice(2).match(/../g) ?? [], (x) => parseInt(x, 16));
}

/** The storage key of an account's `System::Account` entry (blake2_128_concat of its id). */
export function systemAccountKey(id: Uint8Array): string {
  return '0x' + SYSTEM_ACCOUNT_PREFIX + toHex(blake2b(id, { dkLen: 16 })) + toHex(id);
}

/** A cursor over SCALE bytes that refuses to read past the end. */
class Scale {
  private at = 0;
  constructor(private readonly bytes: Uint8Array) {}
  private take(n: number): Uint8Array {
    if (n < 0 || this.at + n > this.bytes.length) throw Error('Invalid chain response');
    const out = this.bytes.subarray(this.at, this.at + n);
    this.at += n;
    return out;
  }
  u8() { return this.take(1)[0]; }
  u32() { const b = this.take(4); return (b[0] | (b[1] << 8) | (b[2] << 16)) + b[3] * 0x1000000; }
  u64() { return this.uint(8); }
  u128() { return this.uint(16); }
  private uint(n: number) {
    const b = this.take(n);
    let v = BigInt(0);
    for (let i = n - 1; i >= 0; i--) v = (v << BigInt(8)) | BigInt(b[i]);
    return v;
  }
  bytesN(n: number) { return this.take(n); }
  bool() { const v = this.u8(); if (v > 1) throw Error('Invalid chain response'); return v === 1; }
  option() { const v = this.u8(); if (v > 1) throw Error('Invalid chain response'); return v === 1; }
  /** SCALE compact, the length prefix of a vector. Lengths above 2^30 are refused as nonsense here. */
  compact(): number {
    const first = this.u8();
    switch (first & 3) {
      case 0: return first >> 2;
      case 1: return ((first | (this.u8() << 8)) >> 2);
      case 2: { const b = [first, this.u8(), this.u8(), this.u8()]; return ((b[0] | (b[1] << 8) | (b[2] << 16)) + b[3] * 0x1000000) / 4 >>> 0; }
      default: throw Error('Invalid chain response');
    }
  }
  done() { if (this.at !== this.bytes.length) throw Error('Invalid chain response'); }
}

export type Balance = { free: bigint; reserved: bigint; frozen: bigint; spendable: bigint };

/**
 * `AccountInfo { nonce, consumers, providers, sufficients: u32, data: { free, reserved, frozen, flags: u128 } }`.
 * An account with no entry holds nothing. `spendable` is what `pallet-balances` lets leave while keeping the
 * account open: free less the larger of (frozen beyond reserved) and the existential deposit.
 */
export function decodeAccount(raw: unknown): Balance {
  if (raw === null) return { free: BigInt(0), reserved: BigInt(0), frozen: BigInt(0), spendable: BigInt(0) };
  const s = new Scale(fromHex(raw));
  for (let i = 0; i < 4; i++) s.u32();
  const free = s.u128();
  const reserved = s.u128();
  const frozen = s.u128();
  s.u128();
  s.done();
  const locked = frozen > reserved ? frozen - reserved : BigInt(0);
  const untouchable = locked > EXISTENTIAL_DEPOSIT ? locked : EXISTENTIAL_DEPOSIT;
  return { free, reserved, frozen, spendable: free > untouchable ? free - untouchable : BigInt(0) };
}

export type OwnedAsset = {
  collection: number;
  item: number;
  name: string;
  content: string;
  origin: string;
  revisable: boolean;
  derivedFrom: [number, number] | null;
  remixDepth: number;
};

function contentRef(s: Scale): string {
  const algo = s.u8();
  if (algo > 2) throw Error('Invalid chain response');
  const root = toHex(s.bytesN(32));
  s.u64();
  return (algo === 0 ? 'blake3:' : algo === 1 ? 'sha256:' : 'blake2b:') + root;
}

/** `Vec<OwnedAsset>` as `Drc369Api_assets_of` returns it. */
export function decodeOwnedAssets(raw: unknown): OwnedAsset[] {
  const s = new Scale(fromHex(raw));
  const n = s.compact();
  if (n > 100_000) throw Error('Invalid chain response');
  const out: OwnedAsset[] = [];
  for (let i = 0; i < n; i++) {
    const collection = s.u32();
    const item = s.u32();
    const origin = contentRef(s);
    const content = contentRef(s);
    if (s.option()) {
      const kind = s.u8();
      if (kind > 1) throw Error('Invalid chain response');
      s.bytesN(kind === 0 ? 20 : 32);
    }
    const revisable = s.bool();
    const derivedFrom: [number, number] | null = s.option() ? [s.u32(), s.u32()] : null;
    const remixDepth = s.u8();
    const nameLen = s.compact();
    if (nameLen > 256) throw Error('Invalid chain response');
    const name = new TextDecoder('utf-8', { fatal: false }).decode(s.bytesN(nameLen));
    out.push({ collection, item, name, content, origin, revisable, derivedFrom, remixDepth });
  }
  s.done();
  return out;
}

export type AccountSnapshot = {
  address: string;
  network: string;
  finalized: number;
  balance: Record<keyof Balance, string>;
  assets: OwnedAsset[];
  assetCount: number;
  observedAt: string;
};

/** Read one account at the finalized block of a network whose genesis is the devnet's. */
export async function readAccount(rpc: Rpc, address: string, now: () => Date = () => new Date()): Promise<AccountSnapshot> {
  const id = ss58Decode(address);
  const chain = await readChain(rpc, now);
  const at = await rpc('chain_getFinalizedHead', []);
  if (typeof at !== 'string' || !/^0x[0-9a-f]{64}$/.test(at)) throw Error('Invalid finalized head');
  const [rawAccount, rawAssets] = await Promise.all([
    rpc('state_getStorage', [systemAccountKey(id), at]),
    rpc('state_call', ['Drc369Api_assets_of', '0x' + toHex(id), at]),
  ]);
  const b = decodeAccount(rawAccount);
  const assets = decodeOwnedAssets(rawAssets);
  return {
    address,
    network: chain.network,
    finalized: chain.finalized,
    balance: { free: b.free.toString(), reserved: b.reserved.toString(), frozen: b.frozen.toString(), spendable: b.spendable.toString() },
    assets: assets.slice(0, MAX_ASSETS_SHOWN),
    assetCount: assets.length,
    observedAt: now().toISOString(),
  };
}
