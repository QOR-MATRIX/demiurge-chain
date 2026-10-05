import { deps } from '@/lib/qor-deps';
import { SESSION_COOKIE, cookie, logout, setCookie } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// Sign out: only from this site's own pages (the Origin header), then QOR ID ends its session too.
export async function POST(request: Request) {
  const url = new URL(request.url);
  if (request.headers.get('origin') !== url.origin) return new Response(null, { status: 403 });
  await logout(deps(), cookie(request.headers.get('cookie'), SESSION_COOKIE));
  return new Response(null, {
    status: 204,
    headers: { 'Set-Cookie': setCookie(SESSION_COOKIE, '', 0, url.protocol === 'https:'), 'Cache-Control': 'no-store' },
  });
}
