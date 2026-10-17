/**
 * The Mesh (ADR-095; docs/blueprints/mesh.md section 2; L10.1): seeding on or off with the person's own limits, what
 * this PC holds and has served, a ticket for each file to hand to someone, and fetching a file by its ticket.
 *
 * Seeding is off until the person turns it on, and turning it on needs both limits. The page says plainly what seeding
 * shows of them (their network address, to the peers they serve) and what it does not do yet: no CGT is paid until the
 * proofs (L10.3) and the hosting escrow (L10.4) exist.
 */

import { useCallback, useEffect, useState } from 'react';
import { Copy, Download, Loader2, Share2 } from 'lucide-react';

import { useQor } from '../state/store';
import { Panel, Stat, ViewHeader } from '../views/parts';
import { bytes, DISK_CHOICES, mesh, rate, short, UPLOAD_CHOICES, type Held, type MeshStatus } from './model';

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

  const seeding = status?.settings.seeding ?? false;
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
                  No CGT is paid yet: payment for seeding comes with the proofs and the hosting escrow, still being
                  built. Today the Mesh holds your own published work.
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
                ? 'Nothing yet. Work you publish from Projects or QQ Studio is taken in within a minute.'
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
                  {seeding && (
                    <button type="button" className="btn btn-ghost px-2" title="Copy a ticket for this file" aria-label="Copy a ticket for this file" onClick={() => void copyTicket(h.root)} data-mesh-ticket>
                      <Copy size={14} />
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}
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
        </Panel>
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
