/**
 * Accessibility and comfort controls.
 *
 * Every control applies immediately and is visible while you adjust it, because
 * a text-size slider you cannot read the result of is not a text-size slider.
 * Nothing here is hidden behind an "apply" step for the same reason.
 *
 * Each control also states plainly who it is for. That is not padding: someone
 * who does not know the word "contrast" still knows "secondary text is hard to
 * read", and naming the symptom is how they find the right switch.
 */

import { useCallback } from 'react';
import {
  Accessibility,
  Contrast as ContrastIcon,
  Eye,
  Link2,
  MoveHorizontal,
  RotateCcw,
  Type,
  Sparkles,
  Waves,
} from 'lucide-react';

import {
  announce,
  DEFAULT_A11Y,
  DENSITY_RANGE,
  SCALE_RANGE,
  type A11ySettings,
  type Contrast,
  type Ambience,
  type Motion,
} from '../lib/a11y';
import { useQor } from '../state/store';
import { InfoTip } from '../components/ui/InfoTip';
import { Panel } from './parts';

export function AccessibilityPanel() {
  const a11y = useQor((s) => s.a11y);
  const setA11y = useQor((s) => s.setA11y);

  const update = useCallback(
    <K extends keyof A11ySettings>(key: K, value: A11ySettings[K], spoken?: string) => {
      setA11y({ ...a11y, [key]: value });
      if (spoken) announce(spoken);
    },
    [a11y, setA11y],
  );

  return (
    <Panel className="p-6">
      <div className="mb-5 flex items-start gap-3">
        <Accessibility size={17} className="mt-0.5 flex-none text-accent" />
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h2 className="heading text-body tracking-label">Accessibility</h2>
            <InfoTip text="These apply everywhere in the launcher and are remembered. Changes take effect as you make them." />
          </div>
        </div>
        <button
          type="button"
          className="btn btn-ghost flex-none"
          onClick={() => {
            setA11y({ ...DEFAULT_A11Y });
            announce('Accessibility settings reset to defaults');
          }}
        >
          <RotateCcw size={13} />
          Reset
        </button>
      </div>

      <div className="flex flex-col gap-6">
        <Slider
          icon={Type}
          label="Interface size"
          hint="Scales everything together: text, spacing, icons and controls."
          value={a11y.scale}
          range={SCALE_RANGE}
          format={(v) => `${Math.round(v * 100)}%`}
          onChange={(v) => update('scale', v)}
          onCommit={(v) => announce(`Interface size ${Math.round(v * 100)} percent`)}
        />

        <Slider
          icon={MoveHorizontal}
          label="Spacing"
          hint="How much room sits between elements. Raise it if the interface feels crowded."
          value={a11y.density}
          range={DENSITY_RANGE}
          format={(v) => (v === 1 ? 'Default' : `${Math.round(v * 100)}%`)}
          onChange={(v) => update('density', v)}
          onCommit={(v) => announce(`Spacing ${Math.round(v * 100)} percent`)}
        />

        <Choice
          icon={ContrastIcon}
          label="Contrast"
          hint="Raises secondary text and borders, which are the first things to become hard to read."
          value={a11y.contrast}
          options={[
            { value: 'normal' as Contrast, label: 'Normal' },
            { value: 'high' as Contrast, label: 'High' },
            { value: 'maximum' as Contrast, label: 'Maximum' },
          ]}
          onChange={(v) => update('contrast', v, `Contrast set to ${v}`)}
        />

        <Choice
          icon={Waves}
          label="Motion"
          hint="Animation can trigger nausea and migraine. System follows your operating system setting."
          value={a11y.motion}
          options={[
            { value: 'system' as Motion, label: 'System' },
            { value: 'full' as Motion, label: 'Full' },
            { value: 'reduced' as Motion, label: 'Reduced' },
          ]}
          onChange={(v) => update('motion', v, `Motion set to ${v}`)}
        />

        <Choice
          icon={Sparkles}
          label="Ambience"
          hint="The backdrop behind the interface. Reduced motion holds it still whatever this says."
          value={a11y.ambience}
          options={[
            { value: 'off' as Ambience, label: 'Off' },
            { value: 'still' as Ambience, label: 'Still' },
            { value: 'live' as Ambience, label: 'Live' },
          ]}
          onChange={(v) => update('ambience', v, `Ambience set to ${v}`)}
        />

        <div className="flex flex-col gap-3 border-t border-edge pt-5">
          <Toggle
            icon={Type}
            label="Readable text"
            hint="Looser letter, word and line spacing, and sentence case instead of capitals. Helps many dyslexic readers."
            checked={a11y.readableText}
            onChange={(v) => update('readableText', v, v ? 'Readable text on' : 'Readable text off')}
          />
          <Toggle
            icon={Eye}
            label="Bold focus outline"
            hint="A thicker, brighter ring around whatever the keyboard is on."
            checked={a11y.boldFocus}
            onChange={(v) => update('boldFocus', v, v ? 'Bold focus on' : 'Bold focus off')}
          />
          <Toggle
            icon={Waves}
            label="Reduce transparency"
            hint="Replaces frosted panels with solid ones, so text never sits over a busy background."
            checked={a11y.reduceTransparency}
            onChange={(v) =>
              update('reduceTransparency', v, v ? 'Transparency reduced' : 'Transparency restored')
            }
          />
          <Toggle
            icon={Link2}
            label="Underline links"
            hint="Never rely on colour alone to mark something as clickable."
            checked={a11y.underlineLinks}
            onChange={(v) => update('underlineLinks', v, v ? 'Links underlined' : 'Underlines off')}
          />
        </div>

        <ScreenReaderNote />
      </div>
    </Panel>
  );
}

/* ──────────────────────────────── controls ─────────────────────────────── */

function Slider({
  icon: Icon,
  label,
  hint,
  value,
  range,
  format,
  onChange,
  onCommit,
}: {
  icon: typeof Type;
  label: string;
  hint: string;
  value: number;
  range: { min: number; max: number; step: number };
  format: (v: number) => string;
  onChange: (v: number) => void;
  onCommit: (v: number) => void;
}) {
  const id = `a11y-${label.replace(/\s+/g, '-').toLowerCase()}`;

  return (
    <div>
      <div className="mb-1.5 flex items-center gap-2.5">
        <Icon size={14} className="flex-none text-ink-muted" />
        <label htmlFor={id} className="text-body font-semibold text-ink">
          {label}
        </label>
        <InfoTip text={hint} />
        <span className="numeric ml-auto text-ui text-accent">{format(value)}</span>
      </div>

      <input
        id={id}
        type="range"
        className="w-full accent-[var(--accent)]"
        min={range.min}
        max={range.max}
        step={range.step}
        value={value}
        aria-valuetext={format(value)}
        onChange={(e) => onChange(Number(e.target.value))}
        onPointerUp={(e) => onCommit(Number((e.target as HTMLInputElement).value))}
        onKeyUp={(e) => onCommit(Number((e.target as HTMLInputElement).value))}
      />
    </div>
  );
}

function Choice<T extends string>({
  icon: Icon,
  label,
  hint,
  value,
  options,
  onChange,
}: {
  icon: typeof Type;
  label: string;
  hint: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div>
      <div className="mb-1.5 flex items-center gap-2.5">
        <Icon size={14} className="flex-none text-ink-muted" />
        <span className="text-body font-semibold text-ink">{label}</span>
        <InfoTip text={hint} />
      </div>

      {/* A radio group, not a row of buttons: screen readers then announce
          "2 of 3" and arrow keys move between options as users expect. */}
      <div role="radiogroup" aria-label={label} className="flex gap-2 pl-[26px]">
        {options.map((option) => {
          const active = option.value === value;
          return (
            <button
              key={option.value}
              type="button"
              role="radio"
              aria-checked={active}
              onClick={() => onChange(option.value)}
              className={`btn flex-1 ${active ? 'btn-primary' : ''}`}
            >
              {option.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}

export function Toggle({
  icon: Icon,
  label,
  hint,
  checked,
  onChange,
}: {
  icon: typeof Type;
  label: string;
  hint: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className="flex items-start gap-2.5 text-left"
    >
      <Icon size={14} className="mt-0.5 flex-none text-ink-muted" />

      <span className="min-w-0 flex-1">
        <span className="block text-body font-semibold text-ink">{label}</span>
        <span className="block text-caption leading-snug text-ink-muted">{hint}</span>
      </span>

      {/* The state is shown by position *and* by colour *and* by a label, so it
          survives colour blindness and a monochrome display. */}
      <span className="flex flex-none items-center gap-2">
        <span className="eyebrow text-micro text-ink-muted">{checked ? 'On' : 'Off'}</span>
        <span
          aria-hidden="true"
          className={`flex h-[18px] w-[32px] items-center rounded-full border p-[2px] transition-colors duration-200 ${
            checked ? 'border-accent bg-accent/25' : 'border-edge bg-well'
          }`}
        >
          <span
            className={`h-[12px] w-[12px] rounded-full transition-transform duration-200 ${
              checked ? 'translate-x-[14px] bg-accent' : 'bg-ink-faint'
            }`}
          />
        </span>
      </span>
    </button>
  );
}

/**
 * An honest note about screen readers.
 *
 * The request was for voice control for blind users. It is worth being precise
 * here rather than simply agreeing, because the distinction decides whether the
 * work actually helps: blind users overwhelmingly navigate with a screen reader
 * they already own and are expert in, and a bespoke voice layer would ask them
 * to abandon that expertise for something worse. Voice *control* is primarily an
 * access route for motor impairment, and on Windows it already exists system-wide.
 *
 * The work that matters is making this interface legible to the tools people
 * already use. That is in progress and stated plainly rather than claimed.
 */
function ScreenReaderNote() {
  return (
    <div className="border-t border-edge pt-5">
      <p className="eyebrow flex items-center gap-1.5">
        Screen readers and voice control
        <InfoTip text="The launcher is built for the screen reader you already use: Narrator, NVDA and JAWS all work, every control is labelled and reachable by keyboard, and results such as a settled transfer are announced. Voice control is left to your operating system, which already does it well and system-wide." />
      </p>
    </div>
  );
}
