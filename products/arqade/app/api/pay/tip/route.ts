import { ApiError, api, body, database, identity, response } from '@/lib/arena-store';
import { finalizedNumber, scanForPayment } from '@/lib/pay-chain';
import { tipSettings } from '@/lib/tip-settings';
import { TipError, askTip } from '@/lib/tips';

export const dynamic = 'force-dynamic';

// Ask to tip a game's creator: a signed qor://pay link for the player's launcher (ADR-076).
export async function POST(req: Request) {
  return api(async () => {
    const data = await body(req);
    const me = await identity();
    const settings = await tipSettings();
    if (!settings) throw new ApiError(503, 'Tips are not set up on this site yet.');
    if (typeof data.game !== 'string' || typeof data.amount !== 'string') throw new ApiError(400, 'Choose a game and an amount.');
    try {
      return response(await askTip(database(), { finalizedNumber, scanForPayment }, settings, me.id, data.game, data.amount, Date.now()), 201);
    } catch (e) {
      if (e instanceof TipError) throw new ApiError(e.status, e.message);
      throw e;
    }
  });
}
