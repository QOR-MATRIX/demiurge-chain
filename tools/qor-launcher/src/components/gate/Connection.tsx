/**
 * The connection chooser, shown at the Gate.
 *
 * # Why this is at the Gate and not in Settings
 *
 * Settings lives behind sign-in, and sign-in needs a reachable identity service.
 * If the configured endpoints are down, putting the only way to change them
 * behind them is a locked door with the key inside. That is precisely what
 * happened: the compiled-in production endpoints stopped answering, and the app
 * had no way to be pointed anywhere else without setting an environment
 * variable and relaunching from a shell.
 *
 * So this probes every known endpoint pair, reports honestly which answered, and
 * lets the choice be made before anyone has signed in to anything.
 *
 * (That was September 2026. Since ADR-056 Settings no longer waits on sign-in,
 * but the chooser stays at the Gate, where a fresh install meets it first.)
 */

import { useCallback, useEffect, useState } from 'react';
import { Check, Loader2, Plug, RefreshCw, X } from 'lucide-react';

import { explain, net, type Probe } from '../../lib/ipc';
import { useQor } from '../../state/store';
import { Surface } from '../ui/Surface';

export function Connection({ onClose }: { onClose: () => void }) {
  const refreshChain = useQor((s) => s.refreshChain);
  const notify = useQor((s) => s.notify);

  const [probes, setProbes] = useState<Probe[]>([]);
  const [busy, setBusy] = useState(true);
  const [custom, setCustom] = useState('');

  const probe = useCallback(async () => {
    setBusy(true);
    try {
      setProbes(await net.probe());
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setBusy(false);
    }
  }, [notify]);

  useEffect(() => {
    void probe();
  }, [probe]);

  const choose = async (rpcUrl: string, authUrl: string) => {
    try {
      await net.use(rpcUrl, authUrl);
      await refreshChain();
      notify('ok', `Now using ${rpcUrl}.`);
      onClose();
    } catch (e) {
      notify('bad', explain(e));
    }
  };

  return (
    <div className="animate-rise">
      <div className="mb-5 flex items-start gap-3">
        <Plug size={16} className="mt-0.5 flex-none text-accent" />
        <div className="min-w-0 flex-1">
          <h2 className="heading mb-1 text-title">Choose a network</h2>
          <p className="text-ui leading-relaxed text-ink-muted">
            The launcher needs a node for the chain and an identity service for your
            QOR ID. Your choice is remembered.
          </p>
        </div>
        <button
          type="button"
          aria-label="Close"
          onClick={onClose}
          className="btn-ghost -mr-2 -mt-1 flex h-7 w-7 flex-none items-center justify-center rounded-qor"
        >
          <X size={14} />
        </button>
      </div>

      <ul className="mb-4 flex flex-col gap-2">
        {busy && probes.length === 0 ? (
          <li className="flex items-center gap-2.5 py-6 text-ui text-ink-muted">
            <Loader2 size={14} className="animate-spin" />
            Checking what is reachable…
          </li>
        ) : (
          probes.map((p) => <ProbeRow key={p.rpc_url} probe={p} onChoose={choose} />)
        )}
      </ul>

      <div className="mb-4 flex gap-2">
        <input
          className="field numeric"
          placeholder="ws://127.0.0.1:9944"
          spellCheck={false}
          value={custom}
          onChange={(e) => setCustom(e.target.value)}
        />
        <button
          type="button"
          className="btn flex-none"
          disabled={!custom.trim().startsWith('ws')}
          onClick={() => {
            // A custom node is assumed to sit alongside its identity service on
            // the conventional port, which is true of the documented local stack.
            // The two are not the same kind of address: the chain speaks
            // WebSocket (ADR-040) and QOR ID speaks HTTP, so the scheme is
            // translated rather than carried across.
            const base = custom.trim().replace(/\/+$/, '');
            const host = base.replace(/:\d+$/, '');
            const auth = host.replace(/^wss:\/\//, 'https://').replace(/^ws:\/\//, 'http://');
            void choose(base, `${auth}:8080/api/v1`);
          }}
        >
          Use
        </button>
      </div>

      <button type="button" className="btn btn-ghost w-full" onClick={() => void probe()}>
        <RefreshCw size={13} className={busy ? 'animate-spin' : ''} />
        Check again
      </button>
    </div>
  );
}

function ProbeRow({
  probe,
  onChoose,
}: {
  probe: Probe;
  onChoose: (rpc: string, auth: string) => void;
}) {
  const both = probe.rpc_ok && probe.auth_ok;
  const partial = probe.rpc_ok !== probe.auth_ok;

  return (
    <Surface
      as="li"
      interactive={probe.rpc_ok}
      cut={both}
      onClick={() => probe.rpc_ok && onChoose(probe.rpc_url, probe.auth_url)}
      className={`flex items-center gap-3.5 px-4 py-3 ${probe.rpc_ok ? '' : 'opacity-55'}`}
    >
      <span
        className={`dot ${both ? 'dot-ok' : partial ? 'dot-warn' : 'dot-bad'}`}
      />

      <div className="min-w-0 flex-1">
        <p className="text-ui font-semibold text-ink">{probe.label}</p>
        <p className="numeric truncate text-micro text-ink-muted">{probe.rpc_url}</p>
      </div>

      <div className="flex flex-none items-center gap-3 text-micro">
        <Marker ok={probe.rpc_ok} label="chain" />
        <Marker ok={probe.auth_ok} label="identity" />
      </div>

      {both && <Check size={14} className="flex-none text-ok" />}
    </Surface>
  );
}

function Marker({ ok, label }: { ok: boolean; label: string }) {
  return (
    <span className={`eyebrow text-micro ${ok ? 'text-ok' : 'text-ink-faint'}`}>
      {label} {ok ? 'up' : 'down'}
    </span>
  );
}
