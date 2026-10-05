import { deps } from '@/lib/qor-deps';
import { SESSION_COOKIE, cookie, currentProfile, setCookie } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// Who is signed in, asked of QOR ID each time, so a sign-out or a revocation there shows here at once.
export async function GET(request: Request) {
  const d = deps();
  const held = cookie(request.headers.get('cookie'), SESSION_COOKIE);
  try {
    const profile = await currentProfile(d, held);
    const headers = new Headers({ 'Cache-Control': 'no-store' });
    if (held && !profile) headers.append('Set-Cookie', setCookie(SESSION_COOKIE, '', 0, new URL(request.url).protocol === 'https:'));
    return Response.json(
      profile
        ? { configured: true, signedIn: true, username: profile.username, qorId: profile.qorId, chainAccount: profile.chainAccount }
        : { configured: d.config !== null, signedIn: false },
      { headers },
    );
  } catch {
    return Response.json({ configured: d.config !== null, signedIn: false, error: 'QOR ID could not be reached.' }, { status: 503, headers: { 'Cache-Control': 'no-store' } });
  }
}
