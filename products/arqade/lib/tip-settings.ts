// The server's settings for tips: ARQADE's request-signing key and the account tips to its own games go to.
// Both are set in Vercel; without either, tips are off and say so.
import { accountHex } from './pay-chain';
import { signingKey } from './pay';
import type { TipSettings } from './tips';

let cached: Promise<TipSettings | null> | undefined;

export function tipSettings(env: Record<string, string | undefined> = process.env): Promise<TipSettings | null> {
  cached ??= (async () => {
    const pkcs8 = env.QOR_PAY_SIGNING_KEY, creator = env.QOR_PAY_TIP_ADDRESS?.trim();
    if (!pkcs8 || !creator) return null;
    accountHex(creator); // throws on an address that is not one
    return { key: await signingKey(pkcs8), creator };
  })().catch((e) => {
    cached = undefined;
    throw e;
  });
  return cached;
}
