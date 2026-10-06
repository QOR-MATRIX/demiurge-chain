// Tips to a game's creator (ADR-071 decision 6), paid through the QOR Launcher (ADR-076, ADR-077).
//
// `askTip` signs a request and records it; `tipStatus` searches finalised blocks for its payment, a few dozen blocks
// per call, and records what it found. The chain and the clock are passed in, so the whole flow runs in tests.

import { parseCgt } from '../sdk/src/amount';
import { CAP_SPARKS, newRequest, payLink, type PayRequest } from './pay';
import type { Db } from './db';

/** ARQADE's own games, which a player may tip their creator for. */
export const GAMES: Record<string, string> = {
  'void-runner': 'Void Runner',
  synapse: 'Synapse',
  orbital: 'Orbital',
  'rift-survivor': 'Rift Survivor',
  flux: 'Flux Four',
  reversi: 'Rift Reversi',
};

/** Open requests one player may hold at once. */
const OPEN_LIMIT = 5;
/** Blocks searched beyond a request's expiry before it is called expired: a payment approved at the last second still
 * finalises a few blocks later. */
const GRACE_BLOCKS = 10;

export class TipError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

export type TipChain = {
  finalizedNumber(): Promise<number>;
  scanForPayment(
    request: { app: string; id: string; to: string; amount: string },
    from: number,
    to: number,
  ): Promise<{ found: { block: number; hash: string; payer: string } | null; scannedTo: number }>;
};

export type TipSettings = { key: CryptoKey; creator: string };

export type TipView = {
  id: string;
  game: string;
  amount: string;
  label: string;
  status: 'waiting' | 'paid' | 'expired';
  expires: number;
  block: number | null;
  payer: string | null;
  link?: string;
};

type Row = {
  id: string; game: string; player: string; to_address: string; amount: string; label: string;
  created: number; expires: number; start_block: number; scanned_to: number; status: TipView['status'];
  paid_block: number | null; payer: string | null;
};

const view = (r: Row, link?: string): TipView => ({
  id: r.id, game: r.game, amount: r.amount, label: r.label, status: r.status, expires: r.expires,
  block: r.paid_block, payer: r.payer, ...(link ? { link } : {}),
});

/** Ask for a tip: a signed `qor://pay` link, recorded as waiting. `amountCgt` is the CGT text the player typed. */
export async function askTip(db: Db, chain: TipChain, settings: TipSettings, player: string, game: string, amountCgt: string, nowMs: number): Promise<TipView> {
  const name = GAMES[game];
  if (!name) throw new TipError(400, 'Choose one of the arcade’s games.');
  let sparks: bigint;
  try {
    sparks = parseCgt(amountCgt.trim());
  } catch {
    throw new TipError(400, 'Enter an amount of CGT, such as 10 or 2.5.');
  }
  if (sparks <= BigInt(0) || sparks > CAP_SPARKS) throw new TipError(400, 'A tip is above zero and at most 100,000 CGT.');
  const open = await db
    .prepare("SELECT COUNT(*) AS n FROM tips WHERE player = ? AND status = 'waiting' AND expires > ?")
    .bind(player, Math.floor(nowMs / 1000))
    .first<{ n: number }>();
  if ((open?.n ?? 0) >= OPEN_LIMIT) throw new TipError(429, 'Finish or let expire the tips already waiting before asking for another.');

  const start = await chain.finalizedNumber();
  const request: PayRequest = newRequest(settings.creator, sparks, `Tip for ${name}`, Math.floor(nowMs / 1000));
  const link = await payLink(request, settings.key);
  await db
    .prepare(
      "INSERT INTO tips (id, game, player, to_address, amount, label, created, expires, start_block, scanned_to, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'waiting')",
    )
    .bind(request.id, game, player, request.to, request.amount, request.label, nowMs, request.exp, start + 1, start)
    .run();
  const row = await db.prepare('SELECT * FROM tips WHERE id = ?').bind(request.id).first<Row>();
  return view(row!, link);
}

/** Where a tip stands, searching the finalised blocks not yet searched. Only the player who asked may see it. */
export async function tipStatus(db: Db, chain: TipChain, player: string, id: string, nowMs: number): Promise<TipView> {
  if (!/^[0-9a-f]{32}$/.test(id)) throw new TipError(400, 'Not a tip.');
  const row = await db.prepare('SELECT * FROM tips WHERE id = ? AND player = ?').bind(id, player).first<Row>();
  if (!row) throw new TipError(404, 'That tip could not be found.');
  if (row.status !== 'waiting') return view(row);

  const head = await chain.finalizedNumber();
  if (head < row.scanned_to + 1) return view(row);
  const { found, scannedTo } = await chain.scanForPayment(
    { app: 'arqade', id: row.id, to: row.to_address, amount: row.amount },
    row.scanned_to + 1,
    head,
  );
  if (found) {
    await db
      .prepare("UPDATE tips SET status = 'paid', scanned_to = ?, paid_block = ?, paid_hash = ?, payer = ? WHERE id = ? AND status = 'waiting'")
      .bind(scannedTo, found.block, found.hash, found.payer, row.id)
      .run();
  } else {
    // Expired once its time has passed and the blocks after it have been searched too.
    const expiredBlocks = Math.ceil(Math.max(0, Math.floor(nowMs / 1000) - row.expires) / 6);
    const expired = nowMs / 1000 > row.expires && expiredBlocks >= GRACE_BLOCKS && scannedTo >= head;
    await db
      .prepare('UPDATE tips SET scanned_to = ?, status = ? WHERE id = ? AND status = ?')
      .bind(scannedTo, expired ? 'expired' : 'waiting', row.id, 'waiting')
      .run();
  }
  return view((await db.prepare('SELECT * FROM tips WHERE id = ?').bind(row.id).first<Row>())!);
}
