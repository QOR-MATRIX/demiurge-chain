import { deps, backToIdentity } from '@/lib/qor-deps';
import { LOGIN_COOKIE, LOGIN_TTL_MS, QorError, configFrom, setCookie, startLogin } from '@/lib/qor-session';
import { failure } from '@/lib/db';

export const dynamic = 'force-dynamic';

// Start a QOR ID sign-in: off to QOR ID's own page, with a state this browser alone will carry back.
export async function GET(request: Request) {
  // Unconfigured is said as such, before the database is needed.
  if (!configFrom(process.env)) return back('unconfigured');
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
    // QOR ID is not contacted to start a sign-in: a failure here is this site's own (its settings or its
    // database), and is said so rather than blamed on QOR ID.
    const reason = e instanceof QorError && e.status === 503 ? 'unconfigured' : 'site';
    if (reason === 'site') console.error('Sign-in could not start:', failure(e));
    return back(reason);
  }
}

function back(reason: string) {
  return new Response(null, { status: 302, headers: { Location: backToIdentity({ qor_error: reason }), 'Cache-Control': 'no-store' } });
}
