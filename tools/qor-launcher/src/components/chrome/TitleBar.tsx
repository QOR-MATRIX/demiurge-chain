/**
 * The window chrome.
 *
 * The window is frameless (`decorations: false`), so this bar provides the drag
 * region and the minimise / maximise / close controls. It also carries the one
 * piece of status that must be visible from every surface: whether the chain is
 * reachable. The vault has no idle lock since ADR-056, so there is no countdown.
 */

import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from 'lucide-react';

import { useQor } from '../../state/store';

export function TitleBar() {
  const chainStatus = useQor((s) => s.chainStatus);
  const version = useQor((s) => s.version);
  const refreshVault = useQor((s) => s.refreshVault);
  const refreshChain = useQor((s) => s.refreshChain);

  const [maximized, setMaximized] = useState(false);

  // Poll for chain liveness and the vault's state. Ten seconds is
  // frequent enough that the indicators are not misleading, and slow enough not
  // to hammer a remote RPC endpoint from an idle window.
  useEffect(() => {
    const id = setInterval(() => {
      void refreshChain();
      void refreshVault();
    }, 10_000);
    return () => clearInterval(id);
  }, [refreshChain, refreshVault]);

  useEffect(() => {
    void getCurrentWindow().isMaximized().then(setMaximized);
  }, []);

  const win = getCurrentWindow();

  const toggleMaximize = async () => {
    await win.toggleMaximize();
    setMaximized(await win.isMaximized());
  };

  const chainTone = !chainStatus
    ? 'dot-warn'
    : chainStatus.reachable
      ? 'dot-ok'
      : 'dot-bad';

  const chainLabel = !chainStatus
    ? 'Connecting'
    : chainStatus.reachable
      ? `Block ${chainStatus.block_number?.toLocaleString() ?? '—'}`
      : 'Offline';

  return (
    <header className="drag-region flex h-10 flex-none items-center gap-4 border-b border-edge bg-void pl-4">
      <div className="flex items-center gap-2.5">
        <ApertureGlyph />
        <span className="font-wordmark text-ui font-semibold tracking-eyebrow text-gilt-bright">QOR</span>
        <span className="eyebrow text-ink-faint">Launcher {version}</span>
      </div>

      <div className="ml-auto flex items-center gap-5 pr-2">
        <span
          className="flex items-center gap-2 text-ink-muted"
          title={chainStatus?.detail ?? chainStatus?.endpoint ?? 'Contacting the chain'}
        >
          <span className={`dot ${chainTone}`} />
          <span className="numeric text-caption">{chainLabel}</span>
        </span>

        <div className="no-drag flex items-center">
          <WindowButton label="Minimise" onClick={() => void win.minimize()}>
            <Minus size={14} strokeWidth={2} />
          </WindowButton>
          <WindowButton
            label={maximized ? 'Restore' : 'Maximise'}
            onClick={() => void toggleMaximize()}
          >
            <Square size={11} strokeWidth={2} />
          </WindowButton>
          <WindowButton label="Close" danger onClick={() => void win.close()}>
            <X size={15} strokeWidth={2} />
          </WindowButton>
        </div>
      </div>
    </header>
  );
}

function WindowButton({
  children,
  label,
  onClick,
  danger,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      // Closing ends something: under the pointer it takes the status red with dark ink on it (6.6:1), in every theme,
      // as QQ Studio's danger buttons do (ADR-108).
      className={`flex h-10 w-11 items-center justify-center text-ink-muted transition-colors duration-150 ${
        danger ? 'hover:bg-bad hover:text-on-accent' : 'hover:bg-raised hover:text-ink'
      }`}
    >
      {children}
    </button>
  );
}

/** The aperture mark, inline so the chrome needs no image request. */
function ApertureGlyph({ size = 15 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M12 4a8 8 0 1 0 5.66 13.66"
        fill="none"
        stroke="var(--accent)"
        strokeWidth="2.5"
        strokeLinecap="butt"
      />
      <path
        d="M14.5 14.5 L20 20"
        fill="none"
        stroke="var(--accent)"
        strokeWidth="2.5"
        strokeLinecap="butt"
      />
      <circle cx="12" cy="12" r="1.6" fill="var(--accent-bright)" />
    </svg>
  );
}
