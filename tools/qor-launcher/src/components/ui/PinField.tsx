/**
 * A profile's PIN (ADR-099): 6 to 12 digits, typed masked, never kept anywhere but this field until it is sent to the
 * host. `confirm` asks for it twice, for choosing a new one.
 */

import { useId } from 'react';

import { validPin } from '../../lib/ipc';

export function PinField({
  value,
  onChange,
  label = 'PIN',
  autoFocus,
  onEnter,
  name,
}: {
  value: string;
  onChange: (pin: string) => void;
  label?: string;
  autoFocus?: boolean;
  onEnter?: () => void;
  name?: string;
}) {
  const id = useId();
  return (
    <label htmlFor={id} className="block">
      <span className="mb-1 block text-caption text-ink-muted">{label}</span>
      <input
        id={id}
        className="field numeric tracking-eyebrow"
        type="password"
        inputMode="numeric"
        autoComplete="off"
        maxLength={12}
        autoFocus={autoFocus}
        value={value}
        data-pin={name ?? 'pin'}
        onChange={(e) => onChange(e.target.value.replace(/[^0-9]/g, ''))}
        onKeyDown={(e) => e.key === 'Enter' && onEnter?.()}
      />
    </label>
  );
}

/** A new PIN, typed twice. `ready` is true once both match and are a PIN. */
export function NewPin({
  pin,
  again,
  onPin,
  onAgain,
  onEnter,
}: {
  pin: string;
  again: string;
  onPin: (pin: string) => void;
  onAgain: (pin: string) => void;
  onEnter?: () => void;
}) {
  const mismatch = again.length >= 6 && pin !== again;
  return (
    <div className="grid gap-3 sm:grid-cols-2">
      <PinField label="New PIN, 6 to 12 digits" value={pin} onChange={onPin} autoFocus name="new" />
      <PinField label="The same PIN again" value={again} onChange={onAgain} onEnter={onEnter} name="again" />
      {mismatch && (
        <p role="alert" className="text-caption text-bad sm:col-span-2">
          The two PINs are not the same.
        </p>
      )}
    </div>
  );
}

export const newPinReady = (pin: string, again: string) => validPin(pin) && pin === again;
