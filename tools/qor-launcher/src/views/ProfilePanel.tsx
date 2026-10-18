/**
 * This QOR ID on this PC (ADR-099): signing it out, its PIN, and the accounts an older wallet's phrase used.
 *
 * Each QOR ID on a PC is a profile with its own wallet and its own data. Signing out closes it completely: nothing of
 * it stays on screen, and it opens again only with its PIN. A profile with no PIN yet chooses one as it signs out, so a
 * first run never asks for one.
 */

import { useState } from 'react';
import { KeyRound, Loader2, LogOut, Search, UserPlus } from 'lucide-react';

import { explain, profiles, shortAddress, type OtherAccount } from '../lib/ipc';
import { useQor } from '../state/store';
import { NewPin, newPinReady } from '../components/ui/PinField';
import { Panel } from './parts';

type Mode = 'idle' | 'sign-out' | 'pin' | { split: OtherAccount };

export function ProfilePanel() {
  const all = useQor((s) => s.profiles);
  const session = useQor((s) => s.session);
  const signOut = useQor((s) => s.signOut);
  const notify = useQor((s) => s.notify);
  const setSession = useQor((s) => s.setSession);
  const refreshVault = useQor((s) => s.refreshVault);
  const refreshProfiles = useQor((s) => s.refreshProfiles);

  const me = all.find((p) => p.signed_in) ?? null;
  const name = session?.qor_id ?? me?.name ?? 'This QOR ID';
  const hasPin = me?.has_pin ?? false;

  const [mode, setMode] = useState<Mode>('idle');
  const [pin, setPin] = useState('');
  const [again, setAgain] = useState('');
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [others, setOthers] = useState<OtherAccount[] | null>(null);

  const reset = () => {
    setMode('idle');
    setPin('');
    setAgain('');
    setProblem(null);
  };

  const attempt = async (work: () => Promise<void>) => {
    setBusy(true);
    setProblem(null);
    try {
      await work();
    } catch (e) {
      setProblem(explain(e));
    } finally {
      setBusy(false);
    }
  };

  // A PIN is asked here only when the profile has none: it is chosen as the profile is first left.
  const needsNewPin = !hasPin;
  const pinReady = !needsNewPin || newPinReady(pin, again);

  const leave = () =>
    attempt(async () => {
      await signOut(needsNewPin ? pin : undefined);
    });

  const changePin = () =>
    attempt(async () => {
      await profiles.setPin(pin);
      await refreshProfiles();
      notify('ok', 'PIN changed.');
      reset();
    });

  const look = () =>
    attempt(async () => {
      setOthers(await profiles.otherAccounts());
    });

  const split = (account: OtherAccount) =>
    attempt(async () => {
      const arrival = await profiles.split(account.index, needsNewPin ? pin : undefined);
      setSession(arrival.session);
      await refreshVault();
      await refreshProfiles();
      setOthers(null);
      reset();
      notify('ok', arrival.session ? `Signed in as ${arrival.session.qor_id}.` : 'Switched to that account.');
    });

  return (
    <Panel className="p-6">
      <p className="eyebrow mb-2" data-profile-panel>
        This QOR ID on this PC
      </p>
      <p className="mb-5 text-ui leading-relaxed text-ink-muted">
        {name} has its own wallet and its own data here. Signing out closes it completely: its balance and everything
        else of it leave the screen, and it opens again only with its PIN.
      </p>

      {mode === 'idle' && (
        <div className="flex flex-wrap gap-2">
          <button type="button" className="btn btn-danger" onClick={() => setMode('sign-out')} data-profile-sign-out>
            <LogOut size={13} />
            Sign out
          </button>
          {hasPin && (
            <button type="button" className="btn" onClick={() => setMode('pin')} data-profile-change-pin>
              <KeyRound size={13} />
              Change PIN
            </button>
          )}
        </div>
      )}

      {mode === 'sign-out' && (
        <div className="space-y-4" data-profile-leaving>
          <p className="text-ui text-ink">
            {needsNewPin
              ? `Choose a PIN for ${name}. Opening it again on this PC will take this PIN.`
              : `Sign out of ${name}? Opening it again will take its PIN.`}
          </p>
          {needsNewPin && <NewPin pin={pin} again={again} onPin={setPin} onAgain={setAgain} onEnter={() => pinReady && void leave()} />}
          <div className="flex gap-2">
            <button type="button" className="btn btn-ghost" onClick={reset}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-danger"
              disabled={!pinReady || busy}
              onClick={() => void leave()}
              data-profile-confirm-sign-out
            >
              {busy ? <Loader2 size={13} className="animate-spin" /> : <LogOut size={13} />}
              {needsNewPin ? 'Choose PIN and sign out' : 'Sign out'}
            </button>
          </div>
        </div>
      )}

      {mode === 'pin' && (
        <div className="space-y-4">
          <NewPin pin={pin} again={again} onPin={setPin} onAgain={setAgain} onEnter={() => newPinReady(pin, again) && void changePin()} />
          <div className="flex gap-2">
            <button type="button" className="btn btn-ghost" onClick={reset}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              disabled={!newPinReady(pin, again) || busy}
              onClick={() => void changePin()}
            >
              Save PIN
            </button>
          </div>
        </div>
      )}

      {typeof mode === 'object' && (
        <div className="space-y-4" data-profile-splitting>
          <p className="text-ui text-ink">
            {needsNewPin ? `First choose a PIN for ${name}, which signs out. ` : `${name} signs out. `}
            Then <span className="numeric">{shortAddress(mode.split.address, 10, 6)}</span> opens as a QOR ID of its own,
            with its {mode.split.balance}.
          </p>
          {needsNewPin && <NewPin pin={pin} again={again} onPin={setPin} onAgain={setAgain} />}
          <div className="flex gap-2">
            <button type="button" className="btn btn-ghost" onClick={reset}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              disabled={!pinReady || busy}
              onClick={() => void split(mode.split)}
              data-profile-confirm-split
            >
              {busy ? <Loader2 size={13} className="animate-spin" /> : <UserPlus size={13} />}
              Switch to it
            </button>
          </div>
        </div>
      )}

      <div className="mt-6 border-t border-edge-soft pt-5">
        <p className="mb-1 text-ui font-semibold text-ink">Other accounts from this wallet&apos;s recovery phrase</p>
        <p className="mb-3 text-caption leading-relaxed text-ink-muted">
          A wallet made before launcher 0.1.13 could hold several accounts. Each one that was used can become a QOR ID of
          its own, with its own balance.
        </p>
        {others === null ? (
          <button type="button" className="btn" disabled={busy} onClick={() => void look()} data-profile-look>
            {busy ? <Loader2 size={13} className="animate-spin" /> : <Search size={13} />}
            Look for them
          </button>
        ) : others.length === 0 ? (
          <p className="text-ui text-ink-muted" data-profile-none>
            None: this wallet&apos;s phrase has no other account in use.
          </p>
        ) : (
          <ul className="grid gap-2" data-profile-others>
            {others.map((o) => (
              <li key={o.address} className="flex flex-wrap items-center gap-3 rounded-qor border border-edge-soft px-4 py-3">
                <span className="min-w-0 flex-1">
                  <span className="numeric block truncate text-ui text-ink">{shortAddress(o.address, 10, 6)}</span>
                  <span className="numeric block text-caption text-accent">{o.balance}</span>
                </span>
                <button
                  type="button"
                  className="btn"
                  onClick={() => {
                    setPin('');
                    setAgain('');
                    setProblem(null);
                    setMode({ split: o });
                  }}
                  data-profile-split={o.index}
                >
                  <UserPlus size={13} />
                  Make it its own QOR ID
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      {problem && (
        <p role="alert" className="mt-4 text-ui text-bad">
          {problem}
        </p>
      )}
    </Panel>
  );
}
