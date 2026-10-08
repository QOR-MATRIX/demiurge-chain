/**
 * ARQADE from the launcher (ADR-078 decision 5, DIRECTION P7.18).
 *
 * ARQADE opens in a window of its own, made by the host, never inside this one:
 * the launcher's content security policy refuses frames, and a page from the
 * network must not share a webview with the interface that draws the host's
 * approvals. That window has no capability, so ARQADE's page cannot call the
 * host. The host decides what it may load (`src-tauri/src/arqade.rs`): ARQADE
 * and QOR ID's sign-in stay in it, a `qor://pay` link goes straight to the host
 * dialog, and any other address opens in the browser.
 *
 * Not yet done (P7.18 stays open): ARQADE signs in through its own QOR ID
 * sign-in in that window, not with the launcher's session.
 */

import { useState } from 'react';
import { ExternalLink, Gamepad2, Loader2 } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';

import { arqade, explain } from '../lib/ipc';
import { Panel, ViewHeader } from './parts';

const ARQADE_URL = 'https://qor-arqade-tau.vercel.app';

export function ArqadeView() {
  const [opening, setOpening] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const open = () => {
    setOpening(true);
    setError(null);
    arqade
      .open()
      .catch((e) => setError(explain(e)))
      .finally(() => setOpening(false));
  };

  return (
    <div className="flex h-full flex-col">
      <ViewHeader
        eyebrow="Play"
        title="ARQADE"
        body="Games on the Demiurge devnet, with rankings, collectibles and tips. It opens in a window of its own; a tip asked for there is approved here, in the launcher's own dialog."
      />
      <div className="min-h-0 flex-1 overflow-y-auto px-8 py-6">
        <Panel className="flex max-w-2xl flex-col gap-4 p-6">
          <div className="flex items-center gap-3">
            <Gamepad2 size={18} strokeWidth={1.75} className="flex-none text-accent" aria-hidden="true" />
            <p className="text-ui text-ink">
              Play, climb the rankings and tip the people who made the games you like.
            </p>
          </div>
          <p className="text-caption leading-snug text-ink-muted">
            Sign in there with your QOR ID. When a game asks for a tip, the launcher shows you what is
            asked and nothing is paid until you approve it.
          </p>
          <div className="flex flex-wrap items-center gap-2">
            <button type="button" className="btn btn-primary" onClick={open} disabled={opening}>
              {opening ? <Loader2 size={13} className="motion-safe:animate-spin" aria-hidden="true" /> : <Gamepad2 size={13} aria-hidden="true" />}
              Open ARQADE
            </button>
            <button type="button" className="btn btn-ghost" onClick={() => void openUrl(ARQADE_URL)}>
              <ExternalLink size={13} aria-hidden="true" />
              Open in your browser
            </button>
          </div>
          {error && (
            <p role="alert" className="text-caption text-bad">
              {error}
            </p>
          )}
        </Panel>
      </div>
    </div>
  );
}
