// Read-only view of Demiurge Devnet (ADR-068), over the public node's standard Substrate JSON-RPC.
// Replaces a call to `chain_getBlockNumber`, a method of the deleted custom chain, at an address that never
// served this one. Nothing here signs, holds a key or chooses a host: the endpoint and the genesis are fixed.

export const DEVNET = {
  name: 'Demiurge Devnet',
  rpc: 'https://rpc.qorsync.dev',
  // chain/specs/demiurge_devnet.raw.json, recorded in chain/DEPLOY-RAILWAY.md.
  genesis: '0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a',
} as const;

export type ChainSnapshot = {
  network: string;
  genesis: string;
  finalized: number;
  best: number;
  observedAt: string;
};

export type Rpc = (method: string, params: unknown[]) => Promise<unknown>;

const HASH = /^0x[0-9a-f]{64}$/;

/** A header's `number` is a hex string; anything else, or a value past a safe integer, is refused. */
export function blockNumber(header: unknown): number {
  const raw = (header as { number?: unknown } | null)?.number;
  if (typeof raw !== 'string' || !/^0x[0-9a-f]{1,13}$/i.test(raw)) throw Error('Invalid block header');
  const n = Number.parseInt(raw.slice(2), 16);
  if (!Number.isSafeInteger(n)) throw Error('Invalid block header');
  return n;
}

/** Reads the network's identity first and refuses any chain whose genesis is not the devnet's. */
export async function readChain(rpc: Rpc, now: () => Date = () => new Date()): Promise<ChainSnapshot> {
  const genesis = await rpc('chain_getBlockHash', [0]);
  if (genesis !== DEVNET.genesis) throw Error('Wrong network: the genesis is not Demiurge Devnet');
  const [network, finalizedHash, bestHeader] = await Promise.all([
    rpc('system_chain', []),
    rpc('chain_getFinalizedHead', []),
    rpc('chain_getHeader', []),
  ]);
  if (network !== DEVNET.name) throw Error('Wrong network: the chain is not Demiurge Devnet');
  if (typeof finalizedHash !== 'string' || !HASH.test(finalizedHash)) throw Error('Invalid finalized head');
  const finalized = blockNumber(await rpc('chain_getHeader', [finalizedHash]));
  const best = blockNumber(bestHeader);
  if (finalized > best) throw Error('Invalid chain response');
  return { network, genesis, finalized, best, observedAt: now().toISOString() };
}

/** JSON-RPC over HTTP to the fixed devnet endpoint, with a timeout per call. */
export function httpRpc(fetcher: typeof fetch = fetch, timeoutMs = 7000): Rpc {
  let id = 0;
  return async (method, params) => {
    const response = await fetcher(DEVNET.rpc, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: ++id, method, params }),
      signal: AbortSignal.timeout(timeoutMs),
    });
    if (!response.ok) throw Error('RPC unavailable');
    const data = (await response.json()) as { result?: unknown; error?: unknown };
    if (data.error !== undefined || !('result' in data)) throw Error('RPC error');
    return data.result;
  };
}
