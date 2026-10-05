import { deps } from '@/lib/qor-deps';
import { SESSION_COOKIE, configFrom, cookie, currentProfile, setCookie } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// Who is signed in, asked of QOR ID each time (the identity card), so a sign-out or a revocation there
// shows here at once. The live arcade accepts an answer up to 30 seconds old (ADR-074).
export async function GET(request: Request) {
  const held = cookie(request.headers.get('cookie'), SESSION_COOKIE);
  const configured = configFrom(process.env) !== null;
  try {
    const profile = await currentProfile(deps(), held);
    const headers = new Headers({ 'Cache-Control': 'no-store' });
    if (held && !profile) headers.append('Set-Cookie', setCookie(SESSION_COOKIE, '', 0, new URL(request.url).protocol === 'https:'));
    return Response.json(
      profile
        ? { configured: true, signedIn: true, username: profile.username, qorId: profile.qorId, chainAccount: profile.chainAccount }
        : { configured, signedIn: false },
      { headers },
    );
  } catch {
    return Response.json({ configured, signedIn: false, error: 'Sign-in status could not be read. Try again in a moment.' }, { status: 503, headers: { 'Cache-Control': 'no-store' } });
  }
}
