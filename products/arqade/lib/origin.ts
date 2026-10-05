// Whether a request came from this site's own pages: its Origin names the host the request was sent to.
// The Host header is used rather than the request's URL, which a server behind a proxy (or `next start`)
// may rebuild with another name. A browser sets both; another site's page cannot make them agree.
export function fromThisSite(req: Request): boolean {
  const origin = req.headers.get('origin');
  if (!origin) return false;
  const host = req.headers.get('host') ?? new URL(req.url).host;
  try {
    return new URL(origin).host === host;
  } catch {
    return false;
  }
}
