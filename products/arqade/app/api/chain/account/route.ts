import { readAccount } from '@/lib/account';
import { httpRpc } from '@/lib/chain';

export const dynamic = 'force-dynamic';

// Read-only: one account's CGT and DRC-369 assets on Demiurge Devnet, at the finalized block (P7.2).
// The address is the only input; the host, methods and network are fixed, and the genesis is checked.
export async function GET(request: Request) {
  const address = new URL(request.url).searchParams.get('address') ?? '';
  try {
    const snapshot = await readAccount(httpRpc(), address);
    return Response.json({ ...snapshot, mode: 'read-only' }, { headers: { 'Cache-Control': 'private, max-age=6' } });
  } catch (e) {
    const message = e instanceof Error ? e.message : '';
    const refused = e instanceof RangeError || message.startsWith('Wrong network');
    return Response.json(
      { error: refused ? message : 'Demiurge Devnet could not be read. Try again later.' },
      { status: e instanceof RangeError ? 400 : 503, headers: { 'Cache-Control': 'no-store' } },
    );
  }
}
