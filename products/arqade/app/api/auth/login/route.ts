import { deps, backToIdentity } from '@/lib/qor-deps';
import { LOGIN_COOKIE, LOGIN_TTL_MS, QorError, setCookie, startLogin } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// Start a QOR ID sign-in: off to QOR ID's own page, with a state this browser alone will carry back.
export async function GET(request: Request) {
  try {
    const { location, state } = await startLogin(deps());
    const secure = new URL(request.url).protocol === 'https:';
    return new Response(null, {
      status: 302,
      headers: {
        Location: location,
        'Set-Cookie': setCookie(LOGIN_COOKIE, state, LOGIN_TTL_MS / 1000, secure, '/api/auth'),
        'Cache-Control': 'no-store',
      },
    });
  } catch (e) {
    const reason = e instanceof QorError && e.status === 503 ? 'unconfigured' : 'unavailable';
    return Response.redirect(backToIdentity(request, { qor_error: reason }), 302);
  }
}
