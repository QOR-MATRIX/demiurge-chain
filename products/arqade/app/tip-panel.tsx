"use client";
// Tip a game's creator in test CGT, paid from the player's own QOR Launcher (ADR-071 decision 6, ADR-076, ADR-077).
// The site asks for a signed request; the launcher shows it in its own dialog and pays; the site learns of it from
// finalised blocks. The site never sees a key.
import { useEffect, useState } from 'react';

const GAMES: [string, string][] = [
  ['flux', 'Flux Four'],
  ['reversi', 'Rift Reversi'],
  ['void-runner', 'Void Runner'],
  ['synapse', 'Synapse'],
  ['orbital', 'Orbital'],
  ['rift-survivor', 'Rift Survivor'],
];

type Tip = { id: string; amount: string; label: string; status: 'waiting' | 'paid' | 'expired'; block: number | null; link?: string };

const cgt = (sparks: string) => {
  const whole = sparks.length > 18 ? sparks.slice(0, -18) : '0';
  const fraction = sparks.padStart(19, '0').slice(-18).replace(/0+$/, '');
  return `${Number(whole).toLocaleString('en-US')}${fraction ? '.' + fraction : ''} CGT`;
};

export function TipPanel() {
  const [game, setGame] = useState('flux');
  const [amount, setAmount] = useState('10');
  const [tip, setTip] = useState<Tip | null>(null);
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!tip || tip.status !== 'waiting') return;
    const timer = setInterval(async () => {
      try {
        const r = await fetch(`/api/pay/${tip.id}`, { cache: 'no-store' });
        const data = (await r.json()) as Tip & { error?: string };
        if (r.ok) setTip((t) => (t && t.id === data.id ? { ...data, link: t.link } : t));
      } catch {
        // The next poll tries again.
      }
    }, 3000);
    return () => clearInterval(timer);
  }, [tip]);

  async function ask() {
    setBusy(true);
    setMessage('');
    try {
      const r = await fetch('/api/pay/tip', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ game, amount }) });
      const data = (await r.json()) as Tip & { error?: string };
      if (!r.ok) {
        setMessage(r.status === 401 ? 'Sign in with QOR ID first, on the QOR Identity screen.' : data.error || 'The tip could not be prepared.');
        return;
      }
      setTip(data);
      if (data.link) window.location.href = data.link;
    } catch {
      setMessage('The tip could not be prepared. Try again in a moment.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="qor-panel tip-panel" aria-labelledby="tip-heading">
      <h2 id="tip-heading">Tip a creator</h2>
      <p className="muted">Send test CGT on Demiurge Devnet to the people who made a game. A tip is a gift: nothing comes back for it. Your QOR Launcher shows it and asks you to approve; this site never sees your key.</p>
      {!tip || tip.status !== 'waiting' ? (
        <div className="tip-form">
          <label>
            Game
            <select value={game} onChange={(e) => setGame(e.target.value)}>
              {GAMES.map(([id, name]) => (
                <option key={id} value={id}>{name}</option>
              ))}
            </select>
          </label>
          <label>
            Amount (CGT)
            <input inputMode="decimal" value={amount} onChange={(e) => setAmount(e.target.value)} />
          </label>
          <button className="primary" onClick={() => void ask()} disabled={busy}>{busy ? 'Preparing…' : 'Tip with the QOR Launcher'}</button>
        </div>
      ) : (
        <div role="status">
          <p>Approve {cgt(tip.amount)} for “{tip.label}” in your QOR Launcher. This page updates once the payment is final.</p>
          {tip.link && <a className="secondary" href={tip.link}>Open the QOR Launcher again</a>}
        </div>
      )}
      {tip?.status === 'paid' && <p role="status">Thank you. {cgt(tip.amount)} reached the creator, final in block #{tip.block}.</p>}
      {tip?.status === 'expired' && <p role="status">That request expired without a payment. Nothing was paid.</p>}
      {message && <p role="alert" className="qor-notice">{message}</p>}
      <p className="muted">Needs the QOR Launcher on this computer, signed in, and on Demiurge Devnet. At most 100,000 CGT a tip.</p>
    </section>
  );
}
