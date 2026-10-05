import { backToIdentity, deps } from '@/lib/qor-deps';
import { LOGIN_COOKIE, SESSION_COOKIE, SESSION_TTL_MS, cookie, finishLogin, setCookie } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// QOR ID sends the browser back here with a code. The server exchanges it; the browser gets only a cookie.
export async function GET(request: Request) {
  const url = new URL(request.url);
  const secure = url.protocol === 'https:';
  const clearLogin = setCookie(LOGIN_COOKIE, '', 0, secure, '/api/auth');
  try {
    const { session } = await finishLogin(deps(), url.searchParams, cookie(request.headers.get('cookie'), LOGIN_COOKIE));
    const headers = new Headers({ Location: backToIdentity({ qor: 'signed_in' }), 'Cache-Control': 'no-store' });
    headers.append('Set-Cookie', setCookie(SESSION_COOKIE, session, SESSION_TTL_MS / 1000, secure));
    headers.append('Set-Cookie', clearLogin);
    return new Response(null, { status: 302, headers });
  } catch {
    return new Response(null, {
      status: 302,
      headers: { Location: backToIdentity({ qor_error: 'refused' }), 'Set-Cookie': clearLogin, 'Cache-Control': 'no-store' },
    });
  }
}
