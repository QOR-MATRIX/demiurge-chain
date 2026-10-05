import { deps } from '@/lib/qor-deps';
import { fromThisSite } from '@/lib/origin';
import { SESSION_COOKIE, cookie, logout, setCookie } from '@/lib/qor-session';

export const dynamic = 'force-dynamic';

// Sign out: only from this site's own pages (the Origin header), then QOR ID ends its session too.
export async function POST(request: Request) {
  const url = new URL(request.url);
  if (!fromThisSite(request)) return new Response(null, { status: 403 });
  // Sign-out always clears this browser's cookie, even if the database cannot be reached.
  await logout(deps(), cookie(request.headers.get('cookie'), SESSION_COOKIE)).catch(() => undefined);
  return new Response(null, {
    status: 204,
    headers: { 'Set-Cookie': setCookie(SESSION_COOKIE, '', 0, url.protocol === 'https:'), 'Cache-Control': 'no-store' },
  });
}
