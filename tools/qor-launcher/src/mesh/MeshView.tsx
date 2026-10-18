/**
 * The Mesh (ADR-095, ADR-097; docs/blueprints/mesh.md section 2; L10.1 to L10.4): seeding on or off with the person's
 * own limits, what this PC holds and has served, what it was paid for that, a ticket for each file to hand to someone,
 * fetching a file by its ticket or its fingerprint, and paying seeders to hold one's own work.
 *
 * Seeding is off until the person turns it on, and turning it on needs both limits. The page says plainly what seeding
 * shows of them (their network address, to the peers they serve), and that paid work is signed by the Mesh for their
 * account only once they allow it, in the host's own dialog, and can never send their CGT. Every price in an escrow is
 * the person's own: the form suggests none.
 */

import { useCallback, useEffect, useState } from 'react';
import { Coins, Copy, Download, Fingerprint, Loader2, Share2 } from 'lucide-react';

import { useQor } from '../state/store';
import { Panel, Stat, ViewHeader } from '../views/parts';
import { bytes, DISK_CHOICES, mesh, rate, short, toSparks, UPLOAD_CHOICES, type Held, type MeshStatus } from './model';

function explain(e: unknown): string {
  return e && typeof e === 'object' && 'message' in e ? String((e as { message: unknown }).message) : String(e);
}

export function MeshView() {
  const notify = useQor((s) => s.notify);
  const [status, setStatus] = useState<MeshStatus | null>(null);
  const [held, setHeld] = useState<Held[]>([]);
  const [disk, setDisk] = useState(0);
  const [upload, setUpload] = useState(0);
  const [busy, setBusy] = useState(false);
  const [ticket, setTicket] = useState('');
  const [fetching, setFetching] = useState(false);
  const [root, setRoot] = useState('');
  const [escrowFor, setEscrowFor] = useState<Held | null>(null);

  const read = useCallback(async () => {
    try {
      const s = await mesh.status();
      setStatus(s);
      setHeld(await mesh.held());
      return s;
    } catch (e) {
      notify('bad', explain(e));
      return null;
    }
  }, [notify]);

  useEffect(() => {
    void read().then((s) => {
      if (s) {
        setDisk(s.settings.diskBytes);
        setUpload(s.settings.uploadBytesPerSecond);
      }
    });
    const id = window.setInterval(() => void read(), 3000);
    return () => window.clearInterval(id);
  }, [read]);

  const configure = async (seeding: boolean) => {
    setBusy(true);
    try {
      setStatus(await mesh.configure({ seeding, diskBytes: disk, uploadBytesPerSecond: upload }));
      setHeld(await mesh.held());
      notify('ok', seeding ? 'Seeding is on.' : 'Seeding is off. Nothing is served from this PC.');
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setBusy(false);
    }
  };

  const copyTicket = async (root: string) => {
    try {
      await navigator.clipboard.writeText(await mesh.ticket(root));
      notify('ok', 'Ticket copied. Whoever has it can fetch this file from this PC while it seeds.');
    } catch (e) {
      notify('bad', explain(e));
    }
  };

  const copyRoot = async (root: string) => {
    try {
      await navigator.clipboard.writeText(root);
      notify('ok', 'Fingerprint copied. Anyone can fetch this file by it once an escrow pays seeders to hold it.');
    } catch (e) {
      notify('bad', explain(e));
    }
  };

  const fetchTicket = async () => {
    setFetching(true);
    try {
      const got = await mesh.fetch(ticket);
      notify('ok', `Fetched and verified: ${bytes(got.size)}.`);
      setTicket('');
      await read();
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setFetching(false);
    }
  };

  const fetchRoot = async () => {
    setFetching(true);
    try {
      const got = await mesh.fetchRoot(root.trim());
      notify('ok', `Found through the chain, fetched and verified: ${bytes(got.size)}.`);
      setRoot('');
      await read();
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setFetching(false);
    }
  };

  const allowPaid = async () => {
    setBusy(true);
    try {
      await mesh.allowPaid();
      notify('ok', 'The Mesh now signs its own work for your account.');
      await read();
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setBusy(false);
    }
  };

  const seeding = status?.settings.seeding ?? false;
  const paid = status?.paid;
  const changed = status && (disk !== status.settings.diskBytes || upload !== status.settings.uploadBytesPerSecond);

  return (
    <div className="flex h-full flex-col overflow-y-auto" data-mesh>
      <ViewHeader
        eyebrow="Peer to peer"
        title="Mesh"
        body="Launchers holding and serving what Demiurge publishes, each file checked against its fingerprint on chain. Seeders are paid in CGT from the hosting escrow of whoever put the work there, for work they prove (ADR-095)."
      />

      <div className="grid gap-6 px-8 py-6">
        <Panel className="p-6">
          <div className="flex items-start gap-4">
            <span className="mt-0.5 text-accent" aria-hidden="true">
              <Share2 size={20} />
            </span>
            <div className="min-w-0 flex-1">
              <h2 className="heading text-title">{seeding ? 'Seeding' : 'Seeding is off'}</h2>
              <p className="mt-1 text-ui text-ink-body" data-mesh-state>
                {seeding
                  ? 'This PC holds and serves your published work within the limits below.'
                  : 'Turn seeding on to give disk and upload to the Mesh. Nothing is served from this PC until you do.'}
              </p>
              <ul className="mt-3 grid gap-1 text-caption text-ink-muted" data-mesh-terms>
                <li>Seeding shows your network address to the peers you serve.</li>
                <li>Every file is checked against its fingerprint; nothing unchecked is kept or served.</li>
                <li>
                  To be paid, the Mesh signs its own work for your account: proof that this PC holds what it took on,
                  claims for what it served, and receipts for what it downloads. You are asked once, when you turn
                  seeding on. It can never send your CGT, and turning seeding off ends it.
                </li>
              </ul>
            </div>
          </div>

          <div className="mt-6 grid gap-5">
            <Choice label="Disk the Mesh may use" values={DISK_CHOICES} chosen={disk} show={bytes} onChoose={setDisk} name="disk" />
            <Choice label="Upload to others, at most" values={UPLOAD_CHOICES} chosen={upload} show={rate} onChoose={setUpload} name="upload" />
          </div>

          <div className="mt-6 flex flex-wrap items-center gap-3">
            {seeding ? (
              <>
                <button type="button" className="btn btn-ghost" disabled={busy} onClick={() => void configure(false)} data-mesh-off>
                  Turn seeding off
                </button>
                {changed && (
                  <button type="button" className="btn btn-primary" disabled={busy} onClick={() => void configure(true)} data-mesh-apply>
                    Keep to these limits
                  </button>
                )}
              </>
            ) : (
              <button
                type="button"
                className="btn btn-primary"
                disabled={busy || disk === 0 || upload === 0}
                onClick={() => void configure(true)}
                data-mesh-on
              >
                {busy && <Loader2 size={14} className="animate-spin" />} Turn seeding on
              </button>
            )}
            {!seeding && (disk === 0 || upload === 0) && (
              <span className="text-caption text-ink-muted">Choose both limits first.</span>
            )}
          </div>
        </Panel>

        <div className="grid gap-4 sm:grid-cols-3" data-mesh-stats>
          <Stat
            label="Holding"
            value={bytes(status?.usedBytes ?? 0)}
            detail={status?.settings.diskBytes ? `of ${bytes(status.settings.diskBytes)}` : 'no disk given'}
          />
          <Stat label="Files" value={String(status?.held ?? 0)} detail="checked against their fingerprints" />
          <Stat label="Served" value={bytes(status?.servedBytes ?? 0)} detail="to others, since the launcher started" />
        </div>

        <Panel className="p-6">
          <h2 className="heading text-title">On this PC</h2>
          {held.length === 0 ? (
            <p className="mt-2 text-ui text-ink-muted" data-mesh-empty>
              {status?.node
                ? 'Nothing yet. Right-click a file in the Library and choose Publish to the Mesh, or mint a project in Projects; it appears here within a minute.'
                : 'The Mesh holds nothing while it is off.'}
            </p>
          ) : (
            <ul className="mt-3 grid gap-2" data-mesh-held>
              {held.map((h) => (
                <li key={`${h.kind}-${h.root}`} className="flex items-center gap-3 rounded border border-edge px-3 py-2" data-mesh-item={h.root}>
                  <span className="numeric selectable text-ui text-ink" title={h.root}>
                    {short(h.root)}
                  </span>
                  <span className="text-caption text-ink-muted">{h.kind === 'own' ? 'your work' : 'fetched'}</span>
                  <span className="numeric ml-auto text-caption text-ink-body">{bytes(h.size)}</span>
                  {seeding && h.kind === 'own' && (
                    <button type="button" className="btn btn-ghost px-2" title="Pay seeders to hold this file" aria-label="Pay seeders to hold this file" onClick={() => setEscrowFor(h)} data-mesh-escrow-open>
                      <Coins size={14} />
                    </button>
                  )}
                  <button type="button" className="btn btn-ghost px-2" title="Copy this file's fingerprint" aria-label="Copy this file's fingerprint" onClick={() => void copyRoot(h.root)} data-mesh-copy-root>
                    <Fingerprint size={14} />
                  </button>
                  {seeding && (
                    <button type="button" className="btn btn-ghost px-2" title="Copy a ticket for this file" aria-label="Copy a ticket for this file" onClick={() => void copyTicket(h.root)} data-mesh-ticket>
                      <Copy size={14} />
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}
          {escrowFor && (
            <EscrowForm
              file={escrowFor}
              onDone={async (opened) => {
                setEscrowFor(null);
                if (opened !== null) {
                  notify('ok', `Escrow ${opened} is open. Seeders can take this file on.`);
                  await read();
                }
              }}
              onError={(e) => notify('bad', explain(e))}
            />
          )}
        </Panel>

        <Panel className="p-6">
          <div data-mesh-paid>
            <h2 className="heading text-title">Paid for what this PC does</h2>
            {!paid?.account ? (
              <div className="mt-2 grid gap-3">
                <p className="text-ui text-ink-muted" data-mesh-paid-off>
                  The Mesh is not signing for any account, so this PC is paid nothing for what it holds or serves.
                </p>
                {seeding && (
                  <div>
                    <button type="button" className="btn btn-primary" disabled={busy} onClick={() => void allowPaid()} data-mesh-allow>
                      Let the Mesh sign its work
                    </button>
                  </div>
                )}
              </div>
            ) : (
              <div className="mt-3 grid gap-4">
                <p className="text-caption text-ink-muted">
                  Signing for <span className="numeric selectable text-ink-body" data-mesh-account>{short(paid.account)}</span>.
                </p>
                <div className="grid gap-4 sm:grid-cols-3">
                  <Stat label="Paid" value={paid.paidCgt} detail="for proven storage and serving" />
                  <Stat label="Held for others" value={String(paid.holdings.length)} detail="files an escrow pays for" />
                  <Stat label="Receipts waiting" value={String(paid.pendingReceipts)} detail="to be claimed" />
                </div>
                {paid.holdings.length > 0 && (
                  <ul className="grid gap-2" data-mesh-holdings>
                    {paid.holdings.map((h) => (
                      <li key={h.escrow} className="flex items-center gap-3 rounded border border-edge px-3 py-2" data-mesh-holding={h.escrow}>
                        <span className="text-caption text-ink-muted">Escrow {h.escrow}</span>
                        <span className="numeric text-ui text-ink" title={h.root}>{short(h.root)}</span>
                        <span className="text-caption text-ink-muted">
                          {h.answered === null ? 'not yet proven' : `proven in period ${h.answered}`}
                        </span>
                        <span className="numeric ml-auto text-caption text-ink-body">{bytes(h.size)}</span>
                      </li>
                    ))}
                  </ul>
                )}
                {paid.payments.length > 0 && (
                  <ul className="grid gap-1 text-caption" data-mesh-payments>
                    {paid.payments.map((p) => (
                      <li key={p.blockHash + p.kind + (p.escrow ?? '')} className="flex gap-3" data-mesh-payment>
                        <span className="text-ink-muted">{p.kind === 'storage' ? 'Holding proven' : 'Serving'}</span>
                        <span className="numeric ml-auto text-ink-body">{p.paidCgt}</span>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </div>
        </Panel>

        <Panel className="p-6">
          <h2 className="heading text-title">Fetch by ticket</h2>
          <p className="mt-1 text-caption text-ink-muted">A ticket names a file and the PC that has it. What arrives is checked before it is kept.</p>
          <div className="mt-3 flex gap-2">
            <input
              className="field numeric flex-1"
              placeholder="Paste a Mesh ticket"
              aria-label="Mesh ticket"
              value={ticket}
              onChange={(e) => setTicket(e.target.value)}
              data-mesh-fetch-input
            />
            <button type="button" className="btn btn-primary" disabled={fetching || ticket.trim().length === 0} onClick={() => void fetchTicket()} data-mesh-fetch>
              {fetching ? <Loader2 size={14} className="animate-spin" /> : <Download size={14} />} Fetch
            </button>
          </div>
          <h2 className="heading mt-6 text-title">Fetch by fingerprint</h2>
          <p className="mt-1 text-caption text-ink-muted">
            A file an escrow pays seeders to hold is found through the chain by its fingerprint alone. What arrives is
            checked before it is kept.
          </p>
          <div className="mt-3 flex gap-2">
            <input
              className="field numeric flex-1"
              placeholder="A file's fingerprint (64 hex characters)"
              aria-label="File fingerprint"
              value={root}
              onChange={(e) => setRoot(e.target.value)}
              data-mesh-root-input
            />
            <button type="button" className="btn btn-primary" disabled={fetching || !/^[0-9a-f]{64}$/i.test(root.trim())} onClick={() => void fetchRoot()} data-mesh-fetch-root>
              {fetching ? <Loader2 size={14} className="animate-spin" /> : <Download size={14} />} Fetch
            </button>
          </div>
        </Panel>
      </div>
    </div>
  );
}

/** Pay seeders to hold one's own work: every number is the person's, and none is suggested. */
function EscrowForm({
  file,
  onDone,
  onError,
}: {
  file: Held;
  onDone: (opened: number | null) => void | Promise<void>;
  onError: (e: unknown) => void;
}) {
  const [copies, setCopies] = useState('');
  const [storage, setStorage] = useState('');
  const [serving, setServing] = useState('');
  const [capMib, setCapMib] = useState('');
  const [deposit, setDeposit] = useState('');
  const [opening, setOpening] = useState(false);

  const terms = (() => {
    const n = Number(copies);
    const cap = Number(capMib);
    const [s, v, d] = [toSparks(storage), toSparks(serving), toSparks(deposit)];
    if (!Number.isInteger(n) || n < 1 || n > 16 || !Number.isInteger(cap) || cap < 0 || s === null || v === null || d === null) return null;
    return {
      root: file.root,
      size: file.size,
      copies: n,
      storagePriceSparks: s,
      servingPricePerMibSparks: v,
      servingCapPerPeriod: cap * 1024 * 1024,
      depositSparks: d,
    };
  })();

  const open = async () => {
    if (!terms) return;
    setOpening(true);
    try {
      await onDone(await mesh.openEscrow(terms));
    } catch (e) {
      onError(e);
    } finally {
      setOpening(false);
    }
  };

  const field = (label: string, value: string, set: (v: string) => void, name: string, unit: string) => (
    <label className="grid gap-1">
      <span className="eyebrow">{label}</span>
      <span className="flex items-center gap-2">
        <input className="field numeric flex-1" inputMode="decimal" value={value} onChange={(e) => set(e.target.value)} data-mesh-escrow-field={name} />
        <span className="text-caption text-ink-muted">{unit}</span>
      </span>
    </label>
  );

  return (
    <div className="mt-4 grid gap-4 rounded border border-edge p-4" data-mesh-escrow>
      <p className="text-ui text-ink-body">
        Pay seeders to hold <span className="numeric">{short(file.root)}</span> ({bytes(file.size)}). The prices are
        yours; the host asks you to approve the escrow before anything is sent.
      </p>
      <div className="grid gap-3 sm:grid-cols-2">
        {field('Copies held', copies, setCopies, 'copies', 'seeders, 1 to 16')}
        {field('For each proof of holding', storage, setStorage, 'storage', 'CGT')}
        {field('For each MiB served', serving, setServing, 'serving', 'CGT')}
        {field('Serving paid for, at most', capMib, setCapMib, 'cap', 'MiB a period')}
        {field('Put in the escrow', deposit, setDeposit, 'deposit', 'CGT')}
      </div>
      <p className="text-caption text-ink-muted" data-mesh-escrow-limit>
        Two people working together can draw on the escrow by downloading from each other, up to the most you pay for
        serving each period. What is left comes back to you when you close it.
      </p>
      <div className="flex gap-3">
        <button type="button" className="btn btn-primary" disabled={!terms || opening} onClick={() => void open()} data-mesh-escrow-submit>
          {opening && <Loader2 size={14} className="animate-spin" />} Open the escrow
        </button>
        <button type="button" className="btn btn-ghost" disabled={opening} onClick={() => void onDone(null)}>
          Cancel
        </button>
      </div>
    </div>
  );
}

function Choice({
  label,
  values,
  chosen,
  show,
  onChoose,
  name,
}: {
  label: string;
  values: number[];
  chosen: number;
  show: (n: number) => string;
  onChoose: (n: number) => void;
  name: string;
}) {
  return (
    <fieldset>
      <legend className="eyebrow mb-2">{label}</legend>
      <div className="flex flex-wrap gap-2" role="radiogroup" aria-label={label} data-mesh-choice={name}>
        {values.map((v) => (
          <button
            key={v}
            type="button"
            role="radio"
            aria-checked={chosen === v}
            className={chosen === v ? 'btn btn-primary' : 'btn btn-ghost'}
            onClick={() => onChoose(v)}
          >
            {show(v)}
          </button>
        ))}
      </div>
    </fieldset>
  );
}
