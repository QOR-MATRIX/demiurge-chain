import { identity } from '@/lib/arena-store';
import { backToIdentity, deps } from '@/lib/qor-deps';
import { LOGIN_COOKIE, SESSION_COOKIE, SESSION_TTL_MS, cookie, finishLogin, linkPlayer, setCookie } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// QOR ID sends the browser back here with a code. The server exchanges it; the browser gets only a cookie.
export async function GET(request: Request) {
  const url = new URL(request.url);
  const secure = url.protocol === 'https:';
  const clearLogin = setCookie(LOGIN_COOKIE, '', 0, secure, '/api/auth');
  const d = deps();
  try {
    const { session, profile } = await finishLogin(d, url.searchParams, cookie(request.headers.get('cookie'), LOGIN_COOKIE));
    // A player signed in to the arcade's host and to QOR ID in the same request has proved both: bind them.
    let link = 'none';
    try {
      const player = (await identity()) as { id?: string };
      if (player?.id) link = await linkPlayer(d.db, player.id, profile.sub);
    } catch {
      link = 'none';
    }
    const headers = new Headers({ Location: backToIdentity(request, { qor: 'signed_in', link }), 'Cache-Control': 'no-store' });
    headers.append('Set-Cookie', setCookie(SESSION_COOKIE, session, SESSION_TTL_MS / 1000, secure));
    headers.append('Set-Cookie', clearLogin);
    return new Response(null, { status: 302, headers });
  } catch {
    return new Response(null, {
      status: 302,
      headers: { Location: backToIdentity(request, { qor_error: 'refused' }), 'Set-Cookie': clearLogin, 'Cache-Control': 'no-store' },
    });
  }
}
