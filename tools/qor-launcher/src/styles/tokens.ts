/**
 * Generated from docs/design/tokens.json by docs/design/tokens.mjs (ADR-108). Do not edit: edit the source,
 * run `node docs/design/tokens.mjs`, and commit what it writes.
 */

export const TOKENS = {
  color: {
    void: '#06070a',
    base: '#0b0e14',
    well: '#0e121a',
    surface: '#131821',
    raised: '#1b212d',
    edge: '#2a3140',
    edgeSoft: '#1f2530',
    ink: '#f3ede1',
    inkBody: '#d6d1c6',
    inkMuted: '#9ba2ad',
    inkFaint: '#868f9d',
    accent: '#ff6a00',
    accentBright: '#ff9142',
    accentDim: '#b44a00',
    onAccent: '#06070a',
    gilt: '#c9a45c',
    giltBright: '#ead39a',
    giltDim: '#7a6236',
    ok: '#22c55e',
    warn: '#e6a817',
    bad: '#d57889'
  },
  font: {
    display: 'Cinzel',
    interface: 'Barlow Semi Condensed',
    body: 'Barlow',
    mono: 'IBM Plex Mono'
  },
  size: {
    micro: 10,
    caption: 11,
    ui: 12,
    body: 13,
    title: 15,
    heading: 18,
    display: 24,
    figure: 38
  },
  tracking: {
    heading: '0.06em',
    label: '0.1em',
    eyebrow: '0.18em'
  },
  radius: {
    control: 8,
    panel: 12
  },
  motion: {
    ease: 'cubic-bezier(0.16, 1, 0.3, 1)',
    quick: 150,
    standard: 260,
    enter: 420,
    sheen: 760
  },
  focus: {
    width: 2,
    halo: 5,
    color: 'accentBright'
  }
} as const;
