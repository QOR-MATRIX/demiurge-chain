// Reading finalised blocks for `qor://pay` payments (ADR-076 decision 5). Server-only.
//
// Events are decoded with `@polkadot/api` against the node's own metadata, over HTTP, so nothing here can sign or
// submit. Accounts are compared as raw 32-byte ids (hex), whatever SS58 prefix an address was written with.

import { ApiPromise, HttpProvider } from '@polkadot/api';
import { blake2AsHex, decodeAddress } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';
import { DEVNET } from './chain';
import { paymentIn, type BlockEvent } from './pay';

const shared = globalThis as unknown as { __arqadeApi?: Promise<ApiPromise> };

/** One API per server instance, over the devnet's HTTP endpoint, checked to be the devnet by genesis. */
async function api(): Promise<ApiPromise> {
  shared.__arqadeApi ??= (async () => {
    const made = await ApiPromise.create({ provider: new HttpProvider(DEVNET.rpc), noInitWarn: true });
    if (made.genesisHash.toHex() !== DEVNET.genesis) throw new Error('The node is not Demiurge Devnet.');
    return made;
  })().catch((e) => {
    shared.__arqadeApi = undefined;
    throw e;
  });
  return shared.__arqadeApi;
}

/** An address as its raw account id, hex. */
export function accountHex(address: string): string {
  return u8aToHex(decodeAddress(address));
}

/** The hash `System.Remarked` carries for a remark. */
export function remarkHash(remark: string): string {
  return blake2AsHex(new TextEncoder().encode(remark), 256);
}

export async function finalizedNumber(): Promise<number> {
  const a = await api();
  const head = await a.rpc.chain.getFinalizedHead();
  return (await a.rpc.chain.getHeader(head)).number.toNumber();
}

/** A block's events, in the shape `paymentIn` reads, with every account as hex. */
export async function eventsAt(number: number, on?: ApiPromise): Promise<{ hash: string; events: BlockEvent[] }> {
  const a = on ?? (await api());
  const hash = await a.rpc.chain.getBlockHash(number);
  const records = (await (await a.at(hash)).query.system.events()) as unknown as Array<{
    phase: { isApplyExtrinsic: boolean; asApplyExtrinsic: { toNumber(): number } };
    event: { section: string; method: string; data: Array<{ toString(): string }> & { names?: string[] | null } };
  }>;
  const events = records.map(({ phase, event }) => {
    const data: Record<string, string> = {};
    // Each field as its codec's own text: an account as SS58 (then its id, hex), a balance as an exact decimal, a
    // hash as hex. Never through JSON numbers, which lose digits above 2^53.
    (event.data.names ?? []).forEach((name, i) => {
      const text = event.data[i]?.toString() ?? '';
      // @polkadot/api renames a field that clashes with a codec method (`hash` becomes `hash_`); the metadata's own
      // name is used, so `Remarked`'s field is `hash`, as in the runtime.
      data[name.replace(/_$/, '')] = /^[1-9A-HJ-NP-Za-km-z]{46,48}$/.test(text) ? accountHex(text) : text;
    });
    return { extrinsic: phase.isApplyExtrinsic ? phase.asApplyExtrinsic.toNumber() : null, section: event.section, method: event.method, data };
  });
  return { hash: hash.toHex(), events };
}

/**
 * Scan finalised blocks `from..=to` (at most `limit`) for a request's payment. `on` is for tests against another
 * node; the site always reads Demiurge Devnet. Returns the block it was found in and
 * the payer, or the last block scanned.
 */
export async function scanForPayment(
  request: { app: string; id: string; to: string; amount: string },
  from: number,
  to: number,
  limit = 40,
  on?: ApiPromise,
): Promise<{ found: { block: number; hash: string; payer: string } | null; scannedTo: number }> {
  const remark = remarkHash(`qor-pay:${request.app}:${request.id}`);
  const recipient = accountHex(request.to);
  const last = Math.min(to, from + limit - 1);
  for (let n = from; n <= last; n++) {
    const { hash, events } = await eventsAt(n, on);
    const payer = paymentIn(events, remark, recipient, request.amount);
    if (payer) return { found: { block: n, hash, payer }, scannedTo: n };
  }
  return { found: null, scannedTo: last };
}
