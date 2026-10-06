import { ApiError, api, database, identity, response } from '@/lib/arena-store';
import { finalizedNumber, scanForPayment } from '@/lib/pay-chain';
import { TipError, tipStatus } from '@/lib/tips';

export const dynamic = 'force-dynamic';
type Context = { params: Promise<{ id: string }> };

// Where a tip stands: paid only once a finalised block holds its payment (ADR-076 decision 5).
export async function GET(_req: Request, ctx: Context) {
  return api(async () => {
    const me = await identity();
    const { id } = await ctx.params;
    try {
      return response(await tipStatus(database(), { finalizedNumber, scanForPayment }, me.id, id, Date.now()));
    } catch (e) {
      if (e instanceof TipError) throw new ApiError(e.status, e.message);
      throw e;
    }
  });
}
