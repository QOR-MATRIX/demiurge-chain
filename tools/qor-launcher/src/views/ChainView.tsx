/**
 * Chain: where the launcher points, and whether that endpoint is healthy.
 */

import { useState } from 'react';
import { Check, Loader2, RefreshCw } from 'lucide-react';

import { chain, explain } from '../lib/ipc';
import { useQor } from '../state/store';
import { Field, Panel, Stat, ViewHeader } from './parts';

// WebSocket addresses: the chain client speaks WebSocket and nothing else
// (ADR-040).
const PRESETS = [
  { label: 'Devnet', url: 'wss://rpc.qorsync.dev', note: 'The public test network' },
  { label: 'Local node', url: 'ws://127.0.0.1:9944', note: 'A node running on this machine' },
];

export function ChainView() {
  const chainStatus = useQor((s) => s.chainStatus);
  const refreshChain = useQor((s) => s.refreshChain);
  const notify = useQor((s) => s.notify);

  const [endpoint, setEndpoint] = useState(chainStatus?.endpoint ?? PRESETS[0]!.url);
  const [applying, setApplying] = useState(false);

  const apply = async (url: string) => {
    setApplying(true);
    try {
      await chain.setEndpoint(url);
      setEndpoint(url);
      await refreshChain();
      notify('ok', `Now pointing at ${url}.`);
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setApplying(false);
    }
  };

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        eyebrow="Network"
        title="Chain"
        body="The launcher talks to a Demiurge node over standard Substrate RPC, and builds every transaction from the runtime metadata that node serves. Transfers are signed by your vault, once you have approved them, and carry the chain's genesis hash and your account's nonce, so a captured transaction cannot be replayed here or anywhere else."
        action={
          <button type="button" className="btn" onClick={() => void refreshChain()}>
            <RefreshCw size={13} />
            Probe
          </button>
        }
      />

      <div className="grid grid-cols-4 gap-4 p-8 pb-4">
        <Stat
          label="Chain"
          value={chainStatus?.chain_name ?? '—'}
          tone={chainStatus?.reachable ? 'ok' : 'bad'}
          detail={chainStatus?.endpoint}
        />
        <Stat
          label="Best block"
          value={chainStatus?.block_number?.toLocaleString() ?? '—'}
          detail={chainStatus?.reachable ? 'Height reported by the node' : 'No response'}
        />
        <Stat
          label="Finalised"
          value={chainStatus?.finalized_number?.toLocaleString() ?? '—'}
          detail={chainStatus?.reachable ? 'Agreed by the validator set' : 'No response'}
        />
        <Stat
          label="Round trip"
          value={chainStatus?.latency_ms != null ? String(chainStatus.latency_ms) : '—'}
          unit="ms"
          detail="Time to answer a block-header query"
        />
      </div>

      {chainStatus?.detail && !chainStatus.reachable && (
        <div className="px-8 pb-4">
          <Panel className="border-l-2 border-bad p-5">
            <p className="eyebrow mb-1.5 text-bad">Why it is not responding</p>
            <p className="selectable text-ui text-ink-body">{chainStatus.detail}</p>
          </Panel>
        </div>
      )}

      <div className="px-8 pb-8">
        <Panel className="p-6">
          <p className="eyebrow mb-5">Endpoint</p>

          <div className="mb-5 grid grid-cols-2 gap-3">
            {PRESETS.map((preset) => {
              const active = chainStatus?.endpoint === preset.url;
              return (
                <button
                  key={preset.url}
                  type="button"
                  onClick={() => void apply(preset.url)}
                  className={`flex items-start gap-3 border p-4 text-left transition-colors duration-150 ${
                    active
                      ? 'border-accent bg-raised'
                      : 'border-edge hover:border-accent-dim'
                  }`}
                >
                  <Check
                    size={14}
                    className={`mt-0.5 flex-none ${active ? 'text-accent' : 'text-transparent'}`}
                  />
                  <span>
                    <span className="block text-ui font-semibold text-ink">
                      {preset.label}
                    </span>
                    <span className="numeric block text-micro text-ink-muted">
                      {preset.url}
                    </span>
                    <span className="mt-1 block text-caption text-ink-faint">{preset.note}</span>
                  </span>
                </button>
              );
            })}
          </div>

          <Field label="Custom endpoint" hint="Must begin with ws:// or wss://">
            <div className="flex gap-2">
              <input
                className="field numeric"
                value={endpoint}
                spellCheck={false}
                onChange={(e) => setEndpoint(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && void apply(endpoint)}
              />
              <button
                type="button"
                className="btn btn-primary flex-none"
                disabled={applying}
                onClick={() => void apply(endpoint)}
              >
                {applying ? <Loader2 size={13} className="animate-spin" /> : null}
                Apply
              </button>
            </div>
          </Field>
        </Panel>
      </div>
    </div>
  );
}
