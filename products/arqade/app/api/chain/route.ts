import { httpRpc, readChain } from '@/lib/chain';

export const dynamic = 'force-dynamic';

// Fixed, read-only: callers cannot choose a host, method or params. Demiurge Devnet only (ADR-068, ADR-069).
export async function GET() {
  try {
    const snapshot = await readChain(httpRpc());
    return Response.json({ ...snapshot, mode: 'read-only' }, { headers: { 'Cache-Control': 'private, max-age=6' } });
  } catch (e) {
    const wrongNetwork = e instanceof Error && e.message.startsWith('Wrong network');
    return Response.json(
      { error: wrongNetwork ? e.message : 'Demiurge Devnet could not be read. Try again later.' },
      { status: 503, headers: { 'Cache-Control': 'no-store' } },
    );
  }
}
