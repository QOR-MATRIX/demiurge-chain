/**
 * First-run orientation.
 *
 * # The problem this solves
 *
 * First-run feedback was: "I was unsure what was actually going on within my
 * dashboard." That is the most serious kind of interface failure, and it was
 * earned. The Nexus opened onto a standing title, a progression track, a
 * holdings figure and six abstract category names, none of which mean anything
 * to someone who has never used the platform. Every one of those is useful *once
 * you know what it is*, which is precisely the knowledge a first-time user does
 * not have.
 *
 * The fix is not more chrome. It is answering, in plain words and before
 * anything else, the three questions a newcomer actually has:
 *
 *   1. What is this thing I am looking at?
 *   2. Is any of this going to cost me something or move my money?
 *   3. What should I do first?
 *
 * The third is the one that converts confusion into momentum, so it is a button,
 * not a sentence, and it points at the next verifiable rite rather than a generic
 * tour.
 *
 * It disappears once dismissed or once the basics are done, and can be summoned
 * again from the Nexus header, because an orientation you cannot get back is a
 * trap for anyone who dismissed it too early.
 */

import { useState } from 'react';
import { ArrowRight, Compass, X } from 'lucide-react';

import { announce } from '../../lib/a11y';
import { useAscent, useQor, type Surface as SurfaceId } from '../../state/store';
import { Surface } from '../ui/Surface';

const DISMISS_KEY = 'qor.orientation.dismissed';

/** Where each rite wants to send you. */
const RITE_DESTINATION: Record<string, SurfaceId> = {
  vault: 'vault',
  name: 'nexus',
  connected: 'chain',
  bound: 'settings',
  funded: 'vault',
  spent: 'vault',
  sovereign: 'chain',
};

export function useOrientation() {
  const [dismissed, setDismissed] = useState(() => {
    try {
      return localStorage.getItem(DISMISS_KEY) === 'true';
    } catch {
      return false;
    }
  });

  const dismiss = () => {
    try {
      localStorage.setItem(DISMISS_KEY, 'true');
    } catch {
      /* a preference is not worth failing over */
    }
    setDismissed(true);
  };

  const restore = () => {
    try {
      localStorage.removeItem(DISMISS_KEY);
    } catch {
      /* ignore */
    }
    setDismissed(false);
  };

  return { dismissed, dismiss, restore };
}

export function Orientation({ onDismiss }: { onDismiss: () => void }) {
  const ascent = useAscent();
  const go = useQor((s) => s.go);
  const token = useQor((s) => s.token);

  const next = ascent.next;
  const destination = next ? (RITE_DESTINATION[next.id] ?? 'nexus') : 'nexus';

  return (
    <Surface
      as="section"
      cut
      className="mb-8 p-6"
      aria-labelledby="orientation-heading"
    >
      <div className="mb-5 flex items-start gap-4">
        <Compass size={20} className="mt-0.5 flex-none text-accent" />

        <div className="min-w-0 flex-1">
          <h2 id="orientation-heading" className="heading mb-2 text-title">
            What you are looking at
          </h2>

          <p className="mb-3 max-w-[70ch] text-body leading-relaxed text-ink-body">
            This is the launcher for Demiurge. Everything below is a{' '}
            <strong className="font-semibold text-ink">system</strong> you can open:
            your wallet, the chain itself, the creative tools, the community. They
            are grouped by what you would be doing, not by what built them.
          </p>

          <p className="max-w-[70ch] text-body leading-relaxed text-ink-muted">
            Nothing here spends anything on its own. {token?.symbol ?? 'CGT'} only
            moves when you sign a transfer yourself, and the launcher always shows
            you the exact amount and recipient before you do.
          </p>
        </div>

        <button
          type="button"
          aria-label="Dismiss orientation"
          title="Dismiss. You can bring this back from the header."
          onClick={() => {
            onDismiss();
            announce('Orientation dismissed. Reopen it from the Nexus header.');
          }}
          className="btn-ghost -mr-2 -mt-1 flex h-7 w-7 flex-none items-center justify-center rounded-qor"
        >
          <X size={14} />
        </button>
      </div>

      <dl className="mb-6 grid grid-cols-[repeat(auto-fit,minmax(220px,1fr))] gap-5 border-t border-edge pt-5">
        <Explain term="Your standing">
          A title that reflects how much of the ecosystem you have actually set up.
          It is not a score and nothing is spent to raise it.
        </Explain>
        <Explain term="The Ascent">
          Seven setup steps. Each one is checked against reality rather than
          awarded, so it always tells you the truth about your account.
        </Explain>
        <Explain term={`Holdings (${token?.symbol ?? 'CGT'})`}>
          The {token?.name ?? 'Creator God Token'} you hold. New accounts start
          empty, and there is no starter grant yet: on the devnet, test CGT is sent to
          testers by hand.
        </Explain>
        <Explain term="Ready, Connect, Forming">
          Whether a system is usable now, needs a server running, or is still being
          built. Anything unfinished says what it is waiting on.
        </Explain>
      </dl>

      {next ? (
        <div className="flex items-center gap-4">
          <div className="min-w-0 flex-1">
            <p className="eyebrow mb-1 text-accent">Start here</p>
            <p className="text-body text-ink">{next.name}</p>
            <p className="text-ui text-ink-muted">{next.how}</p>
          </div>
          <button
            type="button"
            className="btn btn-primary flex-none"
            onClick={() => go(destination)}
          >
            Take me there
            <ArrowRight size={13} />
          </button>
        </div>
      ) : (
        <p className="text-body text-accent">
          Your account is fully set up. Everything below is open to you.
        </p>
      )}
    </Surface>
  );
}

function Explain({ term, children }: { term: string; children: React.ReactNode }) {
  return (
    <div>
      <dt className="mb-1 text-ui font-semibold text-ink">{term}</dt>
      <dd className="text-caption leading-relaxed text-ink-muted">{children}</dd>
    </div>
  );
}
