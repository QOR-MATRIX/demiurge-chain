/**
 * A profile's banner and a room's ambience (docs/blueprints/social.md section 5): one of the six QFX backdrops, in the
 * person's own hue and intensity. The launcher's QFX layer is one WebGL canvas behind everything; a banner is drawn in
 * CSS so any number of cards can show one at no cost to the frame budget, and it holds still when motion is reduced
 * (the stylesheet's reduced-motion rule, and the in-app Motion setting).
 */

import type { CSSProperties } from 'react';

import type { Backdrop, BackdropName } from './model';

export const BACKDROPS: { id: BackdropName; name: string }[] = [
  { id: 'drift', name: 'Drift' },
  { id: 'aurora', name: 'Aurora' },
  { id: 'nebula', name: 'Nebula' },
  { id: 'lattice', name: 'Lattice' },
  { id: 'starfield', name: 'Starfield' },
  { id: 'liquid', name: 'Liquid' },
];

export function Banner({
  backdrop,
  className = '',
  label,
}: {
  backdrop: Backdrop | null | undefined;
  className?: string;
  /** Spoken description; the art itself is decoration. */
  label?: string;
}) {
  const preset = backdrop?.preset ?? 'drift';
  const style = {
    '--b-hue': String(Math.round(backdrop?.hue ?? 250)),
    '--b-int': String(backdrop?.intensity ?? 0.6),
  } as CSSProperties;
  return (
    <div
      className={`social-banner social-banner-${preset} ${backdrop?.preset ? '' : 'social-banner-plain'} ${className}`}
      style={style}
      role={label ? 'img' : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
    >
      <span className="social-banner-a" />
      <span className="social-banner-b" />
    </div>
  );
}
