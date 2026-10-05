"use client";
// The QOR Identity card's live rows (P7.3): signed in or not, as which QOR ID, with which chain account.
// Sign-in happens on QOR ID's own page; this site's server holds the tokens and the browser only a cookie.
import { useEffect, useState } from 'react';

type Me = { configured: boolean; signedIn: boolean; username?: string; qorId?: string; chainAccount?: string | null; error?: string };

const NOTICES: Record<string, string> = {
  unconfigured: 'QOR ID sign-in is not configured for this site yet.',
  unavailable: 'QOR ID could not be reached. Try again in a moment.',
  site: 'Sign-in could not start on this site. Try again in a moment.',
  refused: 'The sign-in did not complete. Nothing was shared. Try again.',
};

export function QorPanel() {
  const [me, setMe] = useState<Me | null>(null);
  const [notice, setNotice] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    const params = new URLSearchParams(window.location.search);
    const parts = [NOTICES[params.get('qor_error') ?? '']].filter(Boolean);
    if (params.has('qor_error') || params.has('qor')) {
      // Keep the address clean once the result has been read.
      window.history.replaceState(null, '', window.location.pathname + window.location.hash);
    }
    fetch('/api/auth/me', { cache: 'no-store' })
      .then((r) => r.json() as Promise<Me>)
      .catch(() => ({ configured: true, signedIn: false, error: 'QOR ID could not be reached.' }))
      .then((data) => {
        if (!live) return;
        setMe(data);
        setNotice([...parts, data.error].filter(Boolean).join(' '));
      });
    return () => {
      live = false;
    };
  }, []);

  async function signOut() {
    setBusy(true);
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
    } finally {
      setMe((m) => (m ? { configured: m.configured, signedIn: false } : m));
      setNotice('Signed out here and at QOR ID.');
      setBusy(false);
    }
  }

  const status = !me ? 'Checking…' : me.signedIn ? 'Signed in' : me.configured ? 'Not signed in' : 'Not configured';
  return (
    <div className="qor-panel">
      <div className="integration-row"><span>QOR authentication</span><span>{status}</span></div>
      <div className="integration-row"><span>Verified account</span><span>{me?.signedIn ? me.qorId : 'None'}</span></div>
      <div className="integration-row">
        <span>Chain account</span>
        <span>{me?.signedIn ? (me.chainAccount ? `${me.chainAccount.slice(0, 6)}…${me.chainAccount.slice(-4)}` : 'No key linked yet') : '—'}</span>
      </div>
      <div className="integration-row"><span>CGT settlement</span><span>Not connected</span></div>
      {notice && <p role="status" className="qor-notice">{notice}</p>}
      {me?.signedIn ? (
        <button className="secondary" onClick={() => void signOut()} disabled={busy}>{busy ? 'Signing out…' : 'Sign out'}</button>
      ) : (
        me?.configured !== false && <a className="primary qor-signin" href="/api/auth/login">Sign in with QOR ID</a>
      )}
      {me?.signedIn && !me.chainAccount && <p className="muted qor-hint">Link a key from the QOR Launcher to give this account a chain identity.</p>}
    </div>
  );
}
