/**
 * Launcher state.
 *
 * One store, because the launcher genuinely has one connected state: which
 * surface you are on depends on whether the vault is unlocked, which depends on
 * whether a vault exists, and the chain indicator is visible from everywhere.
 * Splitting that across several contexts (as apps/hub does, with six nested
 * providers) buys separation that the data does not actually have.
 *
 * Secrets are never stored here. The store holds addresses, balances and status.
 * The recovery phrase exists in this process only for as long as the setup
 * screen is displaying it for transcription, and lives in that component's own
 * state so it dies with the component.
 */

import { useMemo } from 'react';
import { create } from 'zustand';

import { applyTheme, storedTheme } from '../styles/themes';
import { announce, applyA11y, loadA11y, type A11ySettings } from '../lib/a11y';
import { computeAscent, type AscentState } from '../lib/ascent';
import {
  asQorError,
  chain,
  identity,
  shell,
  vault,
  type AccountView,
  type Balance,
  type ChainStatus,
  type LauncherState,
  type Session,
  type TokenInfo,
  type VaultStatus,
} from '../lib/ipc';

/** Which top-level surface is showing. */
export type Surface =
  | 'nexus'
  | 'vault'
  | 'inventory'
  | 'market'
  | 'projects'
  | 'library'
  | 'social'
  | 'mesh'
  | 'chain'
  | 'gates'
  | 'settings';

interface State {
  /** False until the first `launcher_state` call resolves. */
  ready: boolean;
  version: string;
  token: TokenInfo | null;

  vaultStatus: VaultStatus;
  accounts: AccountView[];
  activeAddress: string | null;
  balances: Record<string, Balance>;

  session: Session | null;
  chainStatus: ChainStatus | null;

  surface: Surface;
  /** Transient banner, cleared by the next successful action. */
  notice: { tone: 'ok' | 'bad'; text: string } | null;

  /** Active theme id. Applied to the document, not held in React state alone. */
  theme: string;
  /** Request nonce of the active account; non-zero means it has spent. */
  activeNonce: number | null;
  a11y: A11ySettings;

  bootstrap: () => Promise<void>;
  setTheme: (id: string) => void;
  setA11y: (settings: A11ySettings) => void;
  refreshNonce: () => Promise<void>;
  refreshVault: () => Promise<void>;
  refreshChain: () => Promise<void>;
  refreshBalance: (address: string) => Promise<void>;
  refreshAllBalances: () => Promise<void>;

  setAccounts: (accounts: AccountView[]) => void;
  setActiveAddress: (address: string) => void;
  setSession: (session: Session | null) => void;
  go: (surface: Surface) => void;
  notify: (tone: 'ok' | 'bad', text: string) => void;
  clearNotice: () => void;
  signOut: () => Promise<void>;
  lock: () => Promise<void>;
}

export const useQor = create<State>((set, get) => ({
  ready: false,
  version: '',
  token: null,

  vaultStatus: { state: 'absent' },
  accounts: [],
  activeAddress: null,
  balances: {},

  session: null,
  chainStatus: null,

  surface: 'nexus',
  notice: null,
  theme: storedTheme(),
  activeNonce: null,
  a11y: loadA11y(),

  setTheme(id) {
    applyTheme(id);
    set({ theme: id });
  },

  setA11y(settings) {
    applyA11y(settings);
    set({ a11y: settings });
  },

  async refreshNonce() {
    const address = get().activeAddress;
    if (!address) {
      set({ activeNonce: null });
      return;
    }
    try {
      set({ activeNonce: await chain.nonce(address) });
    } catch {
      // An unreachable node should leave the last known value rather than
      // implying the account has never transacted.
    }
  },

  async bootstrap() {
    applyTheme(get().theme);
    applyA11y(get().a11y);
    try {
      const state: LauncherState = await shell.state();

      // A keychain vault opens with nothing asked (ADR-056), before the first
      // frame, so there is no lock screen to see. Signing in to QOR ID is tried
      // as it opens; a service that is down stops nothing.
      if (state.vault.state === 'locked' && state.vault.sealed_with === 'keychain') {
        try {
          const arrival = await vault.unlock();
          state.vault = { state: 'unlocked', accounts: arrival.accounts };
          if (arrival.session) state.session = arrival.session;
        } catch {
          /* the Gate shows why, with the recovery phrase as the way back */
        }
      }

      set({
        ready: true,
        version: state.version,
        token: state.token,
        vaultStatus: state.vault,
        session: state.session,
      });

      if (state.vault.state === 'unlocked') {
        get().setAccounts(state.vault.accounts);
      }

      // A stored session may still be good from a previous run. Failing here is
      // routine (no tokens yet, or they expired) and must not block startup.
      if (!state.session) {
        try {
          set({ session: await identity.restore() });
        } catch {
          /* not signed in; the Gate will handle it */
        }
      }

      void get().refreshChain();
      void get().refreshNonce();
    } catch (error) {
      set({ ready: true });
      get().notify('bad', asQorError(error).message);
    }
  },

  async refreshVault() {
    const status = await vault.status();
    set({ vaultStatus: status });

    if (status.state === 'unlocked') {
      get().setAccounts(status.accounts);
    } else {
      set({ accounts: [], activeAddress: null, balances: {} });
    }
  },

  async refreshChain() {
    try {
      set({ chainStatus: await chain.status() });
    } catch (error) {
      set({
        chainStatus: {
          endpoint: get().chainStatus?.endpoint ?? '',
          reachable: false,
          chain_name: null,
          block_number: null,
          finalized_number: null,
          latency_ms: null,
          detail: asQorError(error).message,
        },
      });
    }
  },

  async refreshBalance(address) {
    try {
      const balance = await chain.balance(address);
      set((s) => ({ balances: { ...s.balances, [address]: balance } }));
    } catch {
      // An unreachable node should leave the last known balance on screen
      // rather than flashing a zero, which would read as "your money is gone".
    }
  },

  async refreshAllBalances() {
    await Promise.all(get().accounts.map((a) => get().refreshBalance(a.address)));
  },

  setAccounts(accounts) {
    const current = get().activeAddress;
    const stillPresent = accounts.some((a) => a.address === current);

    set({
      accounts,
      activeAddress: stillPresent ? current : (accounts[0]?.address ?? null),
    });

    void get().refreshAllBalances();
    void get().refreshNonce();
  },

  setActiveAddress(address) {
    set({ activeAddress: address });
    void get().refreshBalance(address);
    void get().refreshNonce();
  },

  setSession(session) {
    set({ session });
  },

  go(surface) {
    set({ surface, notice: null });
    // Navigation in a single-window app produces no page load, so nothing is
    // spoken unless we say it.
    announce(`${surface} opened`);
  },

  notify(tone, text) {
    set({ notice: { tone, text } });
    // A banner is invisible to a screen reader. Errors interrupt; successes wait
    // for a natural pause.
    announce(text, tone === 'bad' ? 'assertive' : 'polite');
  },

  clearNotice() {
    set({ notice: null });
  },

  async signOut() {
    await identity.logout();
    set({ session: null, surface: 'nexus' });
  },

  async lock() {
    await vault.lock();
    await get().refreshVault();
  },
}));

/* ─────────────────────────────── selectors ──────────────────────────────── */

export const selectActiveAccount = (s: State): AccountView | null =>
  s.accounts.find((a) => a.address === s.activeAddress) ?? null;

export const selectActiveBalance = (s: State): Balance | null =>
  s.activeAddress ? (s.balances[s.activeAddress] ?? null) : null;

/** Total holdings across every account in the vault, in Sparks. */
export const selectTotalSparks = (s: State): bigint =>
  Object.values(s.balances).reduce((sum, b) => sum + BigInt(b.sparks), 0n);

/**
 * The Ascent, computed from the store.
 *
 * This is a hook rather than a store method for a specific reason. Zustand 5
 * reads through `useSyncExternalStore`, which requires the selector to return a
 * **cached** value: it calls the selector repeatedly and compares results, so a
 * selector that builds a fresh object each time looks like state that never
 * stops changing. React then throws, and because the throw happens during
 * render of the shell, the whole tree unmounts and the window goes black.
 *
 * That is exactly what `useQor((s) => s.ascent())` did. Each call produced a new
 * object with a new `rites` array, so the snapshot never compared equal.
 *
 * Selecting the individual slices is safe, because each is a stable reference
 * held by the store, and `useMemo` then caches the derived result.
 */
export function useAscent(): AscentState {
  const vaultStatus = useQor((s) => s.vaultStatus);
  const session = useQor((s) => s.session);
  const accounts = useQor((s) => s.accounts);
  const balances = useQor((s) => s.balances);
  const chainStatus = useQor((s) => s.chainStatus);
  const activeNonce = useQor((s) => s.activeNonce);
  const activeAddress = useQor((s) => s.activeAddress);

  return useMemo(
    () =>
      computeAscent({
        vaultStatus,
        session,
        accounts,
        balances,
        chainStatus,
        activeNonce,
        activeAddress,
      }),
    [vaultStatus, session, accounts, balances, chainStatus, activeNonce, activeAddress],
  );
}

export const selectVaultUnlocked = (s: State): boolean =>
  s.vaultStatus.state === 'unlocked';

