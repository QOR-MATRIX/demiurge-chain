/**
 * The selected entity's components, as fields (QQ, P3.1).
 *
 * Every number is kept to its range and step as it is typed (`keep`), so what
 * the inspector shows is exactly what is saved. A field being typed into keeps
 * its own text until it parses, so "-" or "0." on the way to a number is not
 * thrown away.
 */

import { useEffect, useState } from 'react';
import { Minus, Plus } from 'lucide-react';

import {
  COMPONENTS,
  LIMITS,
  colourOf,
  defaultComponent,
  keep,
  type ComponentKey,
  type Entity,
  type NumberField,
} from '../runtime/scene';

const TITLES: Record<ComponentKey, string> = {
  shape: 'Shape',
  motion: 'Motion',
  follow: 'Follow the pointer',
  emitter: 'Particles',
};

const LABELS: Partial<Record<NumberField, string>> = {
  x: 'X',
  y: 'Y',
  rotation: 'Rotation',
  scale: 'Scale',
  w: 'Width',
  h: 'Height',
  glow: 'Glow',
  vx: 'Speed X',
  vy: 'Speed Y',
  spin: 'Spin (° / s)',
  strength: 'Strength',
  rate: 'Per second',
  life: 'Life (s)',
  speed: 'Speed',
  spread: 'Spread (°)',
  size: 'Size',
};

function NumberInput({
  field,
  value,
  disabled,
  onChange,
}: {
  field: NumberField;
  value: number;
  disabled: boolean;
  onChange: (value: number) => void;
}) {
  const [draft, setDraft] = useState(String(value));
  const [editing, setEditing] = useState(false);
  useEffect(() => {
    if (!editing) setDraft(String(value));
  }, [value, editing]);
  const { min, max, step } = LIMITS[field];
  return (
    <label className="flex flex-col gap-1">
      <span className="text-caption text-ink-muted">{LABELS[field] ?? field}</span>
      <input
        className="field numeric"
        type="number"
        inputMode="decimal"
        min={min}
        max={max}
        step={step}
        value={draft}
        disabled={disabled}
        data-qq-field={field}
        onFocus={() => setEditing(true)}
        onBlur={() => {
          setEditing(false);
          setDraft(String(value));
        }}
        onChange={(e) => {
          setDraft(e.target.value);
          const n = Number(e.target.value);
          if (e.target.value.trim() !== '' && Number.isFinite(n)) onChange(keep(field, n));
        }}
      />
    </label>
  );
}

function ColourInput({ label, value, disabled, onChange }: { label: string; value: string; disabled: boolean; onChange: (v: string) => void }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-caption text-ink-muted">{label}</span>
      <input
        className="field h-9 p-1"
        type="color"
        value={value}
        disabled={disabled}
        data-qq-field="colour"
        onChange={(e) => {
          const c = colourOf(e.target.value);
          if (c) onChange(c);
        }}
      />
    </label>
  );
}

export function Inspector({
  entity,
  disabled,
  onChange,
}: {
  entity: Entity;
  disabled: boolean;
  onChange: (change: (e: Entity) => Entity) => void;
}) {
  const set = <K extends keyof Entity>(key: K, value: Entity[K]) => onChange((e) => ({ ...e, [key]: value }));
  const t = entity.transform;
  const numbers = <K extends ComponentKey>(key: K, fields: NumberField[]) => {
    const component = entity[key] as Record<string, unknown> | undefined;
    if (!component) return null;
    return (
      <div className="grid grid-cols-2 gap-2">
        {fields.map((f) => (
          <NumberInput
            key={f}
            field={f}
            value={component[f] as number}
            disabled={disabled}
            onChange={(v) => set(key, { ...(entity[key] as object), [f]: v } as unknown as Entity[K])}
          />
        ))}
      </div>
    );
  };

  return (
    <fieldset className="flex flex-col gap-4" disabled={disabled} aria-label={`Components of ${entity.name || entity.id}`}>
      <label className="flex flex-col gap-1">
        <span className="text-caption text-ink-muted">Name</span>
        <input
          className="field"
          value={entity.name}
          maxLength={64}
          onChange={(e) => set('name', e.target.value)}
          data-qq-field="name"
        />
      </label>

      <section className="flex flex-col gap-2" aria-label="Transform">
        <p className="eyebrow">Transform</p>
        <div className="grid grid-cols-2 gap-2">
          {(['x', 'y', 'rotation', 'scale'] as const).map((f) => (
            <NumberInput key={f} field={f} value={t[f]} disabled={disabled} onChange={(v) => set('transform', { ...t, [f]: v })} />
          ))}
        </div>
      </section>

      {COMPONENTS.map((key) => {
        const present = entity[key] !== undefined;
        return (
          <section key={key} className="flex flex-col gap-2 border-t border-edge pt-3" aria-label={TITLES[key]}>
            <div className="flex items-center gap-2">
              <p className="eyebrow flex-1">{TITLES[key]}</p>
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => set(key, present ? undefined : defaultComponent(key))}
                aria-label={present ? `Remove ${TITLES[key]}` : `Add ${TITLES[key]}`}
              >
                {present ? <Minus size={12} aria-hidden="true" /> : <Plus size={12} aria-hidden="true" />}
                {present ? 'Remove' : 'Add'}
              </button>
            </div>
            {key === 'shape' && entity.shape && (
              <>
                <div className="grid grid-cols-2 gap-2">
                  <label className="flex flex-col gap-1">
                    <span className="text-caption text-ink-muted">Kind</span>
                    <select
                      className="field"
                      value={entity.shape.kind}
                      data-qq-field="kind"
                      onChange={(e) => set('shape', { ...entity.shape!, kind: e.target.value === 'rect' ? 'rect' : 'circle' })}
                    >
                      <option value="circle">Circle</option>
                      <option value="rect">Rectangle</option>
                    </select>
                  </label>
                  <ColourInput label="Colour" value={entity.shape.colour} disabled={disabled} onChange={(c) => set('shape', { ...entity.shape!, colour: c })} />
                </div>
                {numbers('shape', ['w', 'h', 'glow'])}
              </>
            )}
            {key === 'motion' && numbers('motion', ['vx', 'vy', 'spin'])}
            {key === 'follow' && numbers('follow', ['strength'])}
            {key === 'emitter' && entity.emitter && (
              <>
                {numbers('emitter', ['rate', 'life', 'speed', 'spread', 'size'])}
                <ColourInput label="Colour" value={entity.emitter.colour} disabled={disabled} onChange={(c) => set('emitter', { ...entity.emitter!, colour: c })} />
              </>
            )}
          </section>
        );
      })}
    </fieldset>
  );
}
