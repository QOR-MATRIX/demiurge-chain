/**
 * Updates (L6.2, ADR-111): what the launcher's update channel found, and the one way to install it.
 *
 * The host checks a signed manifest, fetches what it names, and verifies it against the launcher's own key, twice,
 * before anything runs. This panel only asks: to check, to install what was found, and whether to check by itself.
 */

import { useState } from 'react';
import { Download, Loader2, RefreshCw, ShieldCheck } from 'lucide-react';

import { explain, updates, type UpdateStatus } from '../lib/ipc';
import { useQor } from '../state/store';
import { Toggle } from './AccessibilityPanel';
import { Panel } from './parts';

export function UpdatesPanel() {
  const update = useQor((s) => s.update);
  const setUpdate = useQor((s) => s.setUpdate);
  const notify = useQor((s) => s.notify);
  const [asking, setAsking] = useState(false);

  // A host without an update channel says nothing, and there is nothing to show.
  if (!update) return null;
  const { status, current } = update;
  const busy =
    asking || status.state === 'checking' || status.state === 'downloading' || status.state === 'installing';

  const check = async () => {
    setAsking(true);
    try {
      setUpdate(await updates.check());
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setAsking(false);
    }
  };

  const install = async () => {
    setAsking(true);
    try {
      // On Windows the launcher closes here and the installer opens; this returns only if something stopped it.
      await updates.install();
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setAsking(false);
    }
  };

  const setAutomatic = async (on: boolean) => {
    try {
      setUpdate(await updates.setAutomatic(on));
    } catch (e) {
      notify('bad', explain(e));
    }
  };

  return (
    <Panel className="p-6">
      <div data-updates data-update-state={status.state}>
        <p className="eyebrow mb-4">Updates</p>

        <p className="text-body text-ink" data-update-line aria-live="polite">
          {describe(status, current, update.checked_at)}
        </p>

        {status.state === 'available' && (
          <div className="mt-4">
            {status.notes && (
              <p
                className="selectable mb-4 whitespace-pre-line border-l border-gilt-dim pl-3 text-ui text-ink-body"
                data-update-notes
              >
                {status.notes}
              </p>
            )}
            <button
              type="button"
              className="btn btn-primary"
              disabled={busy}
              onClick={() => void install()}
              data-update-install
            >
              <Download size={14} />
              Update and restart
            </button>
            <p className="mt-2 text-caption text-ink-muted">
              It is checked against the launcher's own signature before anything installs. Windows asks once for
              permission; the launcher closes and opens again on the new version.
            </p>
          </div>
        )}

        {status.state === 'downloading' && (
          <div
            className="mt-4 h-1.5 overflow-hidden rounded-full bg-well"
            role="progressbar"
            aria-label={`Fetching QOR Launcher ${status.version}`}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={percent(status) ?? undefined}
            data-update-progress
          >
            <div className="h-full bg-accent" style={{ width: `${percent(status) ?? 0}%` }} />
          </div>
        )}

        <div className="mt-5 flex flex-wrap items-center gap-3">
          <button
            type="button"
            className="btn btn-ghost"
            disabled={busy}
            onClick={() => void check()}
            data-update-check
          >
            {status.state === 'checking' ? <Loader2 size={14} className="animate-spin" /> : <RefreshCw size={14} />}
            Check now
          </button>
        </div>

        <div className="mt-5" data-update-automatic>
          <Toggle
            icon={ShieldCheck}
            label="Check for updates by itself"
            hint="Every six hours, and soon after the launcher opens. It never installs anything until you say."
            checked={update.automatic}
            onChange={(on) => void setAutomatic(on)}
          />
        </div>
      </div>
    </Panel>
  );
}

function percent(status: Extract<UpdateStatus, { state: 'downloading' }>): number | null {
  return status.total ? Math.min(100, Math.round((status.received / status.total) * 100)) : null;
}

function describe(status: UpdateStatus, current: string, checkedAt: number | null): string {
  const when = checkedAt
    ? new Date(checkedAt * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    : null;
  switch (status.state) {
    case 'unchecked':
      return `QOR Launcher ${current}. Not checked for updates yet.`;
    case 'checking':
      return 'Checking for a newer version…';
    case 'up_to_date':
      return `QOR Launcher ${current} is the newest.${when ? ` Checked at ${when}.` : ''}`;
    case 'available':
      return `QOR Launcher ${status.version} is ready. You have ${current}.`;
    case 'downloading': {
      const done = percent(status);
      return `Fetching QOR Launcher ${status.version}${done === null ? '…' : `: ${done}%`}`;
    }
    case 'installing':
      return `Installing QOR Launcher ${status.version}. The launcher closes and opens again.`;
    case 'failed':
      return status.message;
  }
}
