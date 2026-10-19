/**
 * ARQADE from the launcher (ADR-102, DIRECTION P7.19; before it ADR-078 decision 5, P7.18).
 *
 * **ARQADE's app, docked.** ARQADE is a Qt app the host starts under its product host (`src-tauri/src/arqade/app.rs`)
 * and docks over this screen, as QOR Media docks over the Library: its window becomes a child of the launcher's and sits
 * exactly where this screen's content is. This side tells it where that is, the launcher's colours, who is playing
 * (their QOR ID, level and Sparks, written out here), and whether the person wants less motion. ARQADE holds no key
 * and no token: it signs in to its server by asking the host, which asks QOR ID with the launcher's own session.
 *
 * **The website's window, otherwise.** When the app is not installed, or after it ends, this screen offers ARQADE's
 * website in a window of its own, made by the host, never inside this one: the launcher's content security policy
 * refuses frames, and a page from the network must not share a webview with the interface that draws the host's
 * approvals. That window has no capability, so ARQADE's page cannot call the host. The host decides what it may load
 * (`src-tauri/src/arqade.rs`): ARQADE and QOR ID's sign-in stay in it, a `qor://pay` link goes straight to the host
 * dialog, and any other address opens in the browser. Signed in here, the window opens signed in.
 */

import { useEffect, useRef, useState } from 'react';
import { ExternalLink, Gamepad2, Loader2, RotateCcw } from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';

import { arqade, arqadeApp, explain } from '../lib/ipc';
import { dockOver } from '../lib/dock';
import { wantsLessMotion } from '../lib/a11y';
import { useProgress } from '../components/chrome/Level';
import { useQor } from '../state/store';
import { Panel, ViewHeader } from './parts';

const ARQADE_URL = 'https://qor-arqade-tau.vercel.app';

/** Whether ARQADE's app is here, docked, or has ended while this screen showed. */
type AppState = 'starting' | 'docked' | 'absent' | 'ended';

export function ArqadeView() {
  const [app, setApp] = useState<AppState>('starting');
  const [startError, setStartError] = useState<string | null>(null);

  const start = () => {
    setApp('starting');
    setStartError(null);
    arqadeApp
      .open()
      .then((here) => setApp(here ? 'docked' : 'absent'))
      .catch((e) => {
        setStartError(explain(e));
        setApp('absent');
      });
  };

  useEffect(start, []);
  useEffect(() => arqadeApp.onExit(() => setApp((now) => (now === 'docked' ? 'ended' : now))), []);

  if (app === 'docked') return <DockedArqade />;
  if (app === 'starting') {
    return (
      <div className="flex h-full items-center justify-center bg-void" role="status" aria-label="Opening ARQADE">
        <Loader2 size={20} className="text-accent motion-safe:animate-spin" aria-hidden="true" />
      </div>
    );
  }
  return <WebArqade ended={app === 'ended'} startError={startError} onStart={start} />;
}

/** The area ARQADE's app sits over, and what it is told while it does. */
function DockedArqade() {
  const area = useRef<HTMLDivElement>(null);
  const session = useQor((s) => s.session);
  const a11y = useQor((s) => s.a11y);
  const balance = useQor((s) => (s.activeAddress ? s.balances[s.activeAddress] : undefined));
  const progress = useProgress(Boolean(session));

  useEffect(() => (area.current ? dockOver(area.current, arqadeApp) : undefined), []);

  const signedIn = Boolean(session);
  const name = session?.username ?? null;
  const level = progress?.level ?? null;
  const sparks = balance?.in_sparks ?? null;
  const cgt = balance?.cgt ?? null;
  useEffect(() => {
    void arqadeApp.player({ signed_in: signedIn, name, level, sparks, cgt }).catch(() => undefined);
  }, [signedIn, name, level, sparks, cgt]);

  const lessMotion = wantsLessMotion(a11y);
  useEffect(() => {
    void arqadeApp.prefs({ reduced_motion: lessMotion }).catch(() => undefined);
  }, [lessMotion]);

  return <div ref={area} className="h-full w-full bg-void" role="region" aria-label="ARQADE" data-arqade-dock="" />;
}

/** ARQADE's website, in a window of its own: where the app is not installed, or after it ended. */
function WebArqade({ ended, startError, onStart }: { ended: boolean; startError: string | null; onStart: () => void }) {
  const session = useQor((s) => s.session);
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
          {ended && (
            <div className="flex flex-wrap items-center gap-3" data-arqade-ended="">
              <p className="text-ui text-ink">ARQADE closed.</p>
              <button type="button" className="btn btn-ghost" onClick={onStart}>
                <RotateCcw size={13} aria-hidden="true" />
                Open ARQADE again
              </button>
            </div>
          )}
          <div className="flex items-center gap-3">
            <Gamepad2 size={18} strokeWidth={1.75} className="flex-none text-accent" aria-hidden="true" />
            <p className="text-ui text-ink">
              Play, climb the rankings and tip the people who made the games you like.
            </p>
          </div>
          <p className="text-caption leading-snug text-ink-muted">
            {session
              ? `You arrive signed in as ${session.username}, with the QOR ID this launcher uses.`
              : 'Sign in there with your QOR ID, or sign in here first and arrive signed in.'}{' '}
            When a game asks for a tip, the launcher shows you what is asked, and nothing is paid until you
            approve it.
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
          {(startError || error) && (
            <p role="alert" className="text-caption text-bad">
              {startError ? `ARQADE's app did not start: ${startError}` : error}
            </p>
          )}
        </Panel>
      </div>
    </div>
  );
}
