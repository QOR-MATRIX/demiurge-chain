// Payments through the QOR Launcher (ADR-076, ADR-077), for tips to a game's creator (ADR-071 decision 6, "Tips").
//
// ARQADE's server signs a request, and the browser opens it as a `qor://pay` link. The launcher checks the signature
// against the key it ships with, shows the request in its own dialog, and on approval pays the transfer and a remark
// naming the request in one `batch_all`. ARQADE never holds a key that can move CGT; its key only says "this request
// is ARQADE's". It learns a request was paid by finding the remark's event and the transfer in one extrinsic of a
// finalised block (`findPayment`).
//
// The request's fields and order are the launcher's (`tools/qor-launcher/src-tauri/src/pay.rs`, `Signed`): it refuses
// any other field. Devnet only, at most 100,000 CGT, valid for at most fifteen minutes (ADR-076 decision 6).

import { DEVNET } from './chain';

export const APP_ID = 'arqade';
/** The owner's cap per request, in Sparks: 100,000 CGT (ADR-076 decision 6). */
export const CAP_SPARKS = BigInt(100_000) * BigInt(10) ** BigInt(18);
/** How long a request stays valid. The launcher refuses anything over fifteen minutes. */
export const REQUEST_TTL_SECS = 10 * 60;

export type PayRequest = {
  v: 1;
  app: string;
  id: string;
  to: string;
  amount: string;
  label: string;
  genesis: string;
  exp: number;
};

const hex = (bytes: Uint8Array) => Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');

/** A fresh request id: 32 hex characters. */
export function requestId(): string {
  return hex(crypto.getRandomValues(new Uint8Array(16)));
}

/** The request's JSON, in the launcher's field order. */
export function requestJson(r: PayRequest): string {
  return JSON.stringify({ v: r.v, app: r.app, id: r.id, to: r.to, amount: r.amount, label: r.label, genesis: r.genesis, exp: r.exp });
}

/** Build a request. Amount in Sparks; refused outside 1..=cap. */
export function newRequest(to: string, amountSparks: bigint, label: string, nowSecs: number, id = requestId()): PayRequest {
  if (amountSparks <= BigInt(0) || amountSparks > CAP_SPARKS) throw new Error('The amount must be above zero and at most 100,000 CGT.');
  const clean = label.trim();
  if (!clean || [...clean].length > 80 || /[\u0000-\u001f\u007f]/.test(clean)) throw new Error('The description is not valid.');
  return { v: 1, app: APP_ID, id, to, amount: amountSparks.toString(), label: clean, genesis: DEVNET.genesis, exp: nowSecs + REQUEST_TTL_SECS };
}

/** ARQADE's request-signing key, from the server's environment (`QOR_PAY_SIGNING_KEY`, PKCS#8, base64). */
export async function signingKey(pkcs8Base64: string): Promise<CryptoKey> {
  const der = Uint8Array.from(atob(pkcs8Base64.trim()), (c) => c.charCodeAt(0));
  return crypto.subtle.importKey('pkcs8', der, { name: 'Ed25519' }, false, ['sign']);
}

/** The `qor://pay` link: the request's bytes and ARQADE's Ed25519 signature over exactly those bytes, both hex. */
export async function payLink(r: PayRequest, key: CryptoKey): Promise<string> {
  const bytes = new TextEncoder().encode(requestJson(r));
  const signature = new Uint8Array(await crypto.subtle.sign({ name: 'Ed25519' }, key, bytes));
  return `qor://pay?r=${hex(bytes)}&s=${hex(signature)}`;
}

/** The remark the launcher sends with the payment. */
export function remarkOf(r: Pick<PayRequest, 'app' | 'id'>): string {
  return `qor-pay:${r.app}:${r.id}`;
}

// ── Finding the payment ────────────────────────────────────────────────────────────────────────────────────────

/** One event as read from a block: its extrinsic index (null outside one), and its pallet, name and fields. */
export type BlockEvent = { extrinsic: number | null; section: string; method: string; data: Record<string, string> };

/**
 * Whether a block's events hold this request's payment: in one extrinsic, the `System.Remarked` event for the request's
 * remark hash and a `Balances.Transfer` to the right account of exactly the right amount. Both in one extrinsic is what
 * the launcher's `batch_all` produces; a remark alone, or a transfer alone, or a remark beside some other transfer, is
 * not a payment. Returns the payer, or null.
 */
export function paymentIn(events: BlockEvent[], remarkHash: string, to: string, amount: string): string | null {
  const remarked = events.filter((e) => e.section === 'system' && e.method === 'Remarked' && e.data.hash?.toLowerCase() === remarkHash.toLowerCase() && e.extrinsic !== null);
  for (const remark of remarked) {
    const transfer = events.find(
      (e) => e.extrinsic === remark.extrinsic && e.section === 'balances' && e.method === 'Transfer' && e.data.to === to && e.data.amount === amount && e.data.from === remark.data.sender,
    );
    const failed = events.some((e) => e.extrinsic === remark.extrinsic && e.section === 'system' && e.method === 'ExtrinsicFailed');
    if (transfer && !failed) return transfer.data.from;
  }
  return null;
}
