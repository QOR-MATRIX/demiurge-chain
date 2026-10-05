"use client";
// Look up any account on Demiurge Devnet, read-only (P7.2): its test CGT and its DRC-369 assets, at the finalized
// block. Nothing is signed and nothing is stored. A failed refresh keeps the last answer on screen, marked stale,
// rather than showing zero.
import { useState } from 'react';
import { RefreshCw } from 'lucide-react';
import { formatCgt } from '@/sdk/src/amount';

type Snapshot = {
  address: string;
  finalized: number;
  balance: { free: string; reserved: string; frozen: string; spendable: string };
  assets: { collection: number; item: number; name: string; content: string; revisable: boolean; derivedFrom: [number, number] | null }[];
  assetCount: number;
  observedAt: string;
};

/** Exact CGT, grouped for reading: the whole part gets separators, the fraction is never rounded. */
const cgt = (sparks: string) => {
  const [whole, fraction] = formatCgt(BigInt(sparks)).split('.');
  return BigInt(whole).toLocaleString('en-US') + (fraction ? '.' + fraction : '');
};

export function AccountLookup() {
  const [address, setAddress] = useState('');
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [stale, setStale] = useState(false);
  const [status, setStatus] = useState('');
  const [busy, setBusy] = useState(false);

  async function look() {
    const wanted = address.trim();
    if (!wanted) return;
    setBusy(true);
    setStatus('Reading Demiurge Devnet…');
    try {
      const r = await fetch('/api/chain/account?address=' + encodeURIComponent(wanted));
      const data = (await r.json()) as Snapshot & { error?: string };
      if (!r.ok) throw Error(data.error || 'Demiurge Devnet is unavailable.');
      setSnapshot(data);
      setStale(false);
      setStatus('');
    } catch (e) {
      setStale(snapshot !== null && snapshot.address === wanted);
      if (snapshot?.address !== wanted) setSnapshot(null);
      setStatus(e instanceof Error ? e.message : 'Demiurge Devnet is unavailable.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="account-lookup">
      <div className="eyebrow">ACCOUNT · READ-ONLY</div>
      <p className="muted">Paste any address to see its test CGT and DRC-369 assets on Demiurge Devnet. Nothing is signed.</p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void look();
        }}
      >
        <label htmlFor="account-address" className="muted">Address (SS58, starts with 5)</label>
        <input
          id="account-address"
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          spellCheck={false}
          autoComplete="off"
          placeholder="5…"
        />
        <button className="secondary" type="submit" disabled={busy || !address.trim()}>
          <RefreshCw size={15} />
          {busy ? 'Reading…' : 'Look up'}
        </button>
      </form>
      {status && <div role="status" className="connection-status">{status}</div>}
      {snapshot && (
        <div className="account-result" aria-live="polite">
          {stale && <div className="connection-status">Showing the last answer, from {new Date(snapshot.observedAt).toLocaleTimeString()}: it may be out of date.</div>}
          <div className="integration-row"><span>Spendable (keeps the account open)</span><span>{cgt(snapshot.balance.spendable)} test CGT</span></div>
          <div className="integration-row"><span>Free</span><span>{cgt(snapshot.balance.free)} test CGT</span></div>
          <div className="integration-row"><span>Reserved</span><span>{cgt(snapshot.balance.reserved)} test CGT</span></div>
          {snapshot.balance.frozen !== '0' && <div className="integration-row"><span>Frozen</span><span>{cgt(snapshot.balance.frozen)} test CGT</span></div>}
          <div className="integration-row"><span>DRC-369 assets</span><span>{snapshot.assetCount}</span></div>
          {snapshot.assets.length > 0 && (
            <ul className="asset-list">
              {snapshot.assets.map((a) => (
                <li key={`${a.collection}:${a.item}`}>
                  <strong>{a.name || 'Untitled'}</strong>{' '}
                  <span className="muted">#{a.collection}:{a.item}{a.derivedFrom ? ` · remix of #${a.derivedFrom[0]}:${a.derivedFrom[1]}` : ''}{a.revisable ? '' : ' · permanent'}</span>
                </li>
              ))}
            </ul>
          )}
          {snapshot.assetCount > snapshot.assets.length && <p className="muted">Showing {snapshot.assets.length} of {snapshot.assetCount}.</p>}
          <p className="muted">Finalized block #{snapshot.finalized.toLocaleString()} · read {new Date(snapshot.observedAt).toLocaleTimeString()} · test CGT has no cash value.</p>
        </div>
      )}
    </div>
  );
}
