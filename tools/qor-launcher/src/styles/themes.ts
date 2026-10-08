/**
 * The theme engine.
 *
 * # What a theme is here
 *
 * A theme is a small set of colours, not a skin. Every surface, border and
 * accent in the app derives from these tokens, so adding a theme is adding
 * one entry to this file and nothing else. The launcher reads them as CSS custom
 * properties, which means a theme change is a single attribute swap on `<html>`
 * with no re-render and no flash.
 *
 * # The restraint rule
 *
 * Each theme declares exactly one `accent` and one `counter`. The accent carries
 * meaning: it marks what is live, what is selected, what you are about to press.
 * The counter is structural and appears in borders and dead states. Nothing else
 * is coloured. That single constraint is what keeps this from becoming the
 * rainbow-gradient cyberpunk pastiche that every AI-designed interface collapses
 * into, and it is why the accent still reads as *information* rather than
 * decoration when six panels are on screen at once.
 *
 * # On the ink and edge ramps
 *
 * The dim end of both ramps was raised after first-run feedback that the
 * interface read as flat. The insight is that a dark theme's legibility problem
 * is never the headline: it is the secondary text and the one-pixel borders that
 * carry all the structure while being the hardest things on screen to see.
 * Lifting those specifically adds vibrance without washing the hierarchy out,
 * and the accessibility contrast levels push the same values further still.
 */

export interface Theme {
  id: string;
  name: string;
  /** One line shown in the picker. */
  note: string;
  tokens: {
    /** Deepest ground, behind everything. */
    void: string;
    /** Application background. */
    base: string;
    /** Panel fill. */
    surface: string;
    /** Raised panel, menus, hover fills. */
    raised: string;
    /** Input wells, sunk below the surface. */
    well: string;

    /** The single meaningful colour. */
    accent: string;
    accentBright: string;
    accentDim: string;
    /** Structural second colour: borders, dead states, danger edges. */
    counter: string;

    /** Text, brightest to faintest. */
    ink: string;
    inkBody: string;
    inkMuted: string;
    inkFaint: string;

    /** Hairlines. */
    edge: string;
    edgeSoft: string;
  };
}

export const THEMES: Theme[] = [
  {
    id: 'architect',
    name: 'Architect',
    note: 'Ember on carbon. Industrial, warm, the original.',
    tokens: {
      void: '#06070A',
      base: '#0B0C10',
      surface: '#12151C',
      raised: '#1A1F29',
      well: '#0E1117',
      accent: '#FF6A00',
      accentBright: '#FF9142',
      accentDim: '#B44A00',
      counter: '#8B0000',
      ink: '#FFFFFF',
      inkBody: '#D7D8DA',
      inkMuted: '#93A0AE',
      inkFaint: '#808E9D',
      edge: '#333A47',
      edgeSoft: '#232936',
    },
  },
  {
    id: 'abyss',
    name: 'Abyss',
    note: 'Cold cyan under deep water. Quiet and clinical.',
    tokens: {
      void: '#03070D',
      base: '#050A12',
      surface: '#0B1420',
      raised: '#11202F',
      well: '#060D16',
      accent: '#22D3EE',
      accentBright: '#67E8F9',
      accentDim: '#0E7490',
      counter: '#1E3A5F',
      ink: '#F0F9FF',
      inkBody: '#CBE0EC',
      inkMuted: '#89A6BA',
      inkFaint: '#728E9F',
      edge: '#294254',
      edgeSoft: '#1B3040',
    },
  },
  {
    id: 'sanguine',
    name: 'Sanguine',
    note: 'Arterial red on near-black. Unsettling on purpose.',
    tokens: {
      void: '#070304',
      base: '#0C0608',
      surface: '#170B0F',
      raised: '#231116',
      well: '#10070A',
      accent: '#FF3D62',
      accentBright: '#FF6B87',
      accentDim: '#A3102C',
      counter: '#4A0D1A',
      ink: '#FFF5F7',
      inkBody: '#E5D2D6',
      inkMuted: '#B08E97',
      inkFaint: '#A18089',
      edge: '#4A272F',
      edgeSoft: '#351C23',
    },
  },
  {
    id: 'veridian',
    name: 'Veridian',
    note: 'Phosphor green. The terminal you were warned about.',
    tokens: {
      void: '#030705',
      base: '#050A07',
      surface: '#0A160F',
      raised: '#102218',
      well: '#06100A',
      accent: '#3DFF88',
      accentBright: '#86FFB4',
      accentDim: '#12A24F',
      counter: '#14532D',
      ink: '#F0FFF6',
      inkBody: '#CDE6D6',
      inkMuted: '#88AC96',
      inkFaint: '#70917C',
      edge: '#27452F',
      edgeSoft: '#1A3122',
    },
  },
  {
    id: 'numen',
    name: 'Numen',
    note: 'Violet light through smoked glass. The Pleroma default.',
    tokens: {
      void: '#06030C',
      base: '#0A0612',
      surface: '#140C22',
      raised: '#1F1333',
      well: '#0C0718',
      accent: '#B166F8',
      accentBright: '#C99BFF',
      accentDim: '#6B21A8',
      counter: '#3B1D63',
      ink: '#FAF5FF',
      inkBody: '#DCD2E8',
      inkMuted: '#A596BA',
      inkFaint: '#9183A6',
      edge: '#3D2B56',
      edgeSoft: '#2A1E3E',
    },
  },
];

export const DEFAULT_THEME = 'architect';

const STORAGE_KEY = 'qor.theme';

/** Map a theme's tokens onto the CSS custom properties the stylesheet reads. */
function cssVariables(theme: Theme): Record<string, string> {
  const t = theme.tokens;
  return {
    '--void': t.void,
    '--base': t.base,
    '--surface': t.surface,
    '--raised': t.raised,
    '--well': t.well,
    '--accent': t.accent,
    '--accent-bright': t.accentBright,
    '--accent-dim': t.accentDim,
    '--counter': t.counter,
    '--ink': t.ink,
    '--ink-body': t.inkBody,
    '--ink-muted': t.inkMuted,
    '--ink-faint': t.inkFaint,
    '--edge': t.edge,
    '--edge-soft': t.edgeSoft,
  };
}

export function findTheme(id: string): Theme {
  return THEMES.find((t) => t.id === id) ?? THEMES[0]!;
}

const STYLE_ID = 'qor-theme';

/**
 * Apply a theme to the document.
 *
 * Writes the tokens as one stylesheet rule on `:root[data-theme]`, still a
 * repaint with no layout pass and no unstyled frame. They are deliberately not
 * inline properties on `<html>`: an inline property outranks every stylesheet
 * rule, and that is why the accessibility contrast levels never changed a
 * colour. As a rule, the theme sits where a more specific contrast rule in
 * `qor.css` can raise it (roadmap L1.2).
 */
export function applyTheme(id: string): Theme {
  const theme = findTheme(id);
  const root = document.documentElement;

  let style = document.getElementById(STYLE_ID) as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement('style');
    style.id = STYLE_ID;
    document.head.appendChild(style);
  }

  const declarations = Object.entries(cssVariables(theme))
    .map(([name, value]) => `  ${name}: ${value};`)
    .join('\n');
  style.textContent = `:root[data-theme] {\n${declarations}\n}`;
  root.setAttribute('data-theme', theme.id);

  try {
    localStorage.setItem(STORAGE_KEY, theme.id);
  } catch {
    // A theme preference is not worth failing a launch over.
  }

  return theme;
}

/** The stored preference, or the default. */
export function storedTheme(): string {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && THEMES.some((t) => t.id === saved)) return saved;
  } catch {
    /* fall through */
  }
  return DEFAULT_THEME;
}

/**
 * QQ's starter palette (ADR-082). The colours a new QQ scene and a newly added
 * entity begin with, before the creator chooses their own. They are the game's,
 * not the interface's: a scene keeps its colours whatever theme the launcher
 * wears, so these are fixed values rather than theme variables.
 */
export const QQ_PALETTE = {
  background: '#07080c',
  ember: '#ff6a00',
  spark: '#ffb15c',
  glint: '#ffd27a',
  ice: '#5ad1ff',
  slate: '#2b3140',
} as const;
