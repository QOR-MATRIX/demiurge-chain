/** Shared view primitives. Small, local, and not worth a package. */

import type { ReactNode } from 'react';

import { InfoTip } from '../components/ui/InfoTip';
import { Surface } from '../components/ui/Surface';

/**
 * A panel: a `Surface` with corner marks, and a button when it can be clicked.
 */
export function Panel({
  as = 'div',
  className = '',
  children,
  onClick,
}: {
  as?: 'div' | 'button';
  className?: string;
  children: ReactNode;
  onClick?: () => void;
}) {
  return (
    <Surface as={as} cut interactive={as === 'button'} onClick={onClick} className={className}>
      {children}
    </Surface>
  );
}

export function Stat({
  label,
  value,
  unit,
  detail,
  tone,
  mono,
  onClick,
}: {
  label: string;
  value: string;
  unit?: string;
  detail?: string;
  tone?: 'ok' | 'bad';
  mono?: boolean;
  onClick?: () => void;
}) {
  return (
    <Panel
      as={onClick ? 'button' : 'div'}
      onClick={onClick}
      className="p-5 text-left hover:border-accent-dim"
    >
      <p className="eyebrow mb-3">{label}</p>
      <p className="mb-1.5 flex items-baseline gap-1.5">
        <span
          className={`${mono ? 'numeric text-title' : 'numeric text-display'} ${
            tone === 'bad' ? 'text-bad' : 'text-ink'
          }`}
        >
          {value}
        </span>
        {unit && <span className="eyebrow text-ink-muted">{unit}</span>}
      </p>
      {detail && <p className="truncate text-caption text-ink-muted">{detail}</p>}
    </Panel>
  );
}

export function ViewHeader({
  eyebrow,
  title,
  body,
  action,
}: {
  eyebrow: string;
  title: string;
  body?: string;
  action?: ReactNode;
}) {
  return (
    <header className="view-header flex flex-none items-start gap-6 border-b border-edge-soft px-8 py-6">
      <div className="min-w-0 flex-1">
        <p className="eyebrow mb-2 text-accent">{eyebrow}</p>
        <div className="flex items-center gap-2">
          <h1 className="view-title">{title}</h1>
          {body && <InfoTip text={body} />}
        </div>
      </div>
      {action && <div className="flex flex-none items-center gap-2">{action}</div>}
    </header>
  );
}

export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <span className="eyebrow mb-2 flex items-center gap-1.5">
        {label}
        {hint && <InfoTip text={hint} />}
      </span>
      {children}
    </label>
  );
}
