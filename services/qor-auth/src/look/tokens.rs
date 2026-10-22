// Generated from docs/design/tokens.json by docs/design/tokens.mjs (ADR-108). Do not edit: edit the source,
// run `node docs/design/tokens.mjs`, and commit what it writes.

/// Colours, as `#rrggbb`.
pub const VOID: &str = "#06070a";
pub const BASE: &str = "#0b0e14";
pub const WELL: &str = "#0e121a";
pub const SURFACE: &str = "#131821";
pub const RAISED: &str = "#1b212d";
pub const EDGE: &str = "#2a3140";
pub const EDGE_SOFT: &str = "#1f2530";
pub const INK: &str = "#f3ede1";
pub const INK_BODY: &str = "#d6d1c6";
pub const INK_MUTED: &str = "#9ba2ad";
pub const INK_FAINT: &str = "#868f9d";
pub const ACCENT: &str = "#ff6a00";
pub const ACCENT_BRIGHT: &str = "#ff9142";
pub const ACCENT_DIM: &str = "#b44a00";
pub const ON_ACCENT: &str = "#06070a";
pub const GILT: &str = "#c9a45c";
pub const GILT_BRIGHT: &str = "#ead39a";
pub const GILT_DIM: &str = "#7a6236";
pub const OK: &str = "#22c55e";
pub const WARN: &str = "#e6a817";
pub const BAD: &str = "#d57889";

/// Faces: each family and its fallbacks, as a CSS `font-family` value.
pub const FONT_DISPLAY: &str = "'Cinzel', 'Trajan Pro', 'Times New Roman', Georgia, serif";
pub const FONT_INTERFACE: &str = "'Barlow Semi Condensed', 'Arial Narrow', 'Segoe UI', system-ui, sans-serif";
pub const FONT_BODY: &str = "'Barlow', 'Segoe UI', system-ui, -apple-system, sans-serif";
pub const FONT_MONO: &str = "'IBM Plex Mono', ui-monospace, 'Cascadia Mono', Consolas, monospace";

/// The type scale, in pixels; tracking, in ems.
pub const SIZE_MICRO: u32 = 10;
pub const SIZE_CAPTION: u32 = 11;
pub const SIZE_UI: u32 = 12;
pub const SIZE_BODY: u32 = 13;
pub const SIZE_TITLE: u32 = 15;
pub const SIZE_HEADING: u32 = 18;
pub const SIZE_DISPLAY: u32 = 24;
pub const SIZE_FIGURE: u32 = 38;
pub const TRACKING_HEADING: f32 = 0.06;
pub const TRACKING_LABEL: f32 = 0.1;
pub const TRACKING_EYEBROW: f32 = 0.18;

/// Corners, in pixels.
pub const RADIUS_CONTROL: u32 = 8;
pub const RADIUS_PANEL: u32 = 12;

/// The stylesheet a page carries first: every value as a custom property (`--qor-…`), and the faces.
pub const CSS: &str = r##"/*
  Generated from docs/design/tokens.json by docs/design/tokens.mjs (ADR-108). Do not edit: edit the source,
  run `node docs/design/tokens.mjs`, and commit what it writes.
*/

@font-face {
  font-family: 'Barlow';
  font-style: normal;
  font-display: swap;
  font-weight: 400;
  src: url(/fonts/barlow-latin-400-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Barlow';
  font-style: normal;
  font-display: swap;
  font-weight: 400;
  src: url(/fonts/barlow-latin-ext-400-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Barlow';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/barlow-latin-500-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Barlow';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/barlow-latin-ext-500-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Barlow';
  font-style: normal;
  font-display: swap;
  font-weight: 600;
  src: url(/fonts/barlow-latin-600-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Barlow';
  font-style: normal;
  font-display: swap;
  font-weight: 600;
  src: url(/fonts/barlow-latin-ext-600-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Barlow Semi Condensed';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/barlow-semi-condensed-latin-500-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Barlow Semi Condensed';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/barlow-semi-condensed-latin-ext-500-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Barlow Semi Condensed';
  font-style: normal;
  font-display: swap;
  font-weight: 600;
  src: url(/fonts/barlow-semi-condensed-latin-600-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Barlow Semi Condensed';
  font-style: normal;
  font-display: swap;
  font-weight: 600;
  src: url(/fonts/barlow-semi-condensed-latin-ext-600-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Barlow Semi Condensed';
  font-style: normal;
  font-display: swap;
  font-weight: 700;
  src: url(/fonts/barlow-semi-condensed-latin-700-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Barlow Semi Condensed';
  font-style: normal;
  font-display: swap;
  font-weight: 700;
  src: url(/fonts/barlow-semi-condensed-latin-ext-700-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Cinzel';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/cinzel-latin-500-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Cinzel';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/cinzel-latin-ext-500-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Cinzel';
  font-style: normal;
  font-display: swap;
  font-weight: 600;
  src: url(/fonts/cinzel-latin-600-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Cinzel';
  font-style: normal;
  font-display: swap;
  font-weight: 600;
  src: url(/fonts/cinzel-latin-ext-600-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'Cinzel';
  font-style: normal;
  font-display: swap;
  font-weight: 700;
  src: url(/fonts/cinzel-latin-700-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'Cinzel';
  font-style: normal;
  font-display: swap;
  font-weight: 700;
  src: url(/fonts/cinzel-latin-ext-700-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'IBM Plex Mono';
  font-style: normal;
  font-display: swap;
  font-weight: 400;
  src: url(/fonts/ibm-plex-mono-latin-400-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'IBM Plex Mono';
  font-style: normal;
  font-display: swap;
  font-weight: 400;
  src: url(/fonts/ibm-plex-mono-latin-ext-400-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

@font-face {
  font-family: 'IBM Plex Mono';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/ibm-plex-mono-latin-500-normal.woff2) format('woff2');
  unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD;
}

@font-face {
  font-family: 'IBM Plex Mono';
  font-style: normal;
  font-display: swap;
  font-weight: 500;
  src: url(/fonts/ibm-plex-mono-latin-ext-500-normal.woff2) format('woff2');
  unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF;
}

:root {
  --qor-void: #06070a;
  --qor-base: #0b0e14;
  --qor-well: #0e121a;
  --qor-surface: #131821;
  --qor-raised: #1b212d;
  --qor-edge: #2a3140;
  --qor-edge-soft: #1f2530;
  --qor-ink: #f3ede1;
  --qor-ink-body: #d6d1c6;
  --qor-ink-muted: #9ba2ad;
  --qor-ink-faint: #868f9d;
  --qor-accent: #ff6a00;
  --qor-accent-bright: #ff9142;
  --qor-accent-dim: #b44a00;
  --qor-on-accent: #06070a;
  --qor-gilt: #c9a45c;
  --qor-gilt-bright: #ead39a;
  --qor-gilt-dim: #7a6236;
  --qor-ok: #22c55e;
  --qor-warn: #e6a817;
  --qor-bad: #d57889;
  --qor-font-display: "Cinzel", "Trajan Pro", "Times New Roman", Georgia, serif;
  --qor-font-interface: "Barlow Semi Condensed", "Arial Narrow", "Segoe UI", system-ui, sans-serif;
  --qor-font-body: "Barlow", "Segoe UI", system-ui, -apple-system, sans-serif;
  --qor-font-mono: "IBM Plex Mono", ui-monospace, "Cascadia Mono", Consolas, monospace;
  --qor-size-micro: 10px;
  --qor-size-caption: 11px;
  --qor-size-ui: 12px;
  --qor-size-body: 13px;
  --qor-size-title: 15px;
  --qor-size-heading: 18px;
  --qor-size-display: 24px;
  --qor-size-figure: 38px;
  --qor-tracking-heading: 0.06em;
  --qor-tracking-label: 0.1em;
  --qor-tracking-eyebrow: 0.18em;
  --qor-radius-control: 8px;
  --qor-radius-panel: 12px;
  --qor-ease: cubic-bezier(0.16, 1, 0.3, 1);
  --qor-duration-quick: 150ms;
  --qor-duration-standard: 260ms;
  --qor-duration-enter: 420ms;
  --qor-duration-sheen: 760ms;
  --qor-focus-width: 2px;
  --qor-focus-halo: 5px;
  --qor-focus: var(--qor-accent-bright);
}
"##;

/// The faces the stylesheet names, served at `/fonts/<file>`: each file's name and bytes.
pub const FACES: &[(&str, &[u8])] = &[
    ("barlow-latin-400-normal.woff2", include_bytes!("fonts/barlow-latin-400-normal.woff2")),
    ("barlow-latin-ext-400-normal.woff2", include_bytes!("fonts/barlow-latin-ext-400-normal.woff2")),
    ("barlow-latin-500-normal.woff2", include_bytes!("fonts/barlow-latin-500-normal.woff2")),
    ("barlow-latin-ext-500-normal.woff2", include_bytes!("fonts/barlow-latin-ext-500-normal.woff2")),
    ("barlow-latin-600-normal.woff2", include_bytes!("fonts/barlow-latin-600-normal.woff2")),
    ("barlow-latin-ext-600-normal.woff2", include_bytes!("fonts/barlow-latin-ext-600-normal.woff2")),
    ("barlow-semi-condensed-latin-500-normal.woff2", include_bytes!("fonts/barlow-semi-condensed-latin-500-normal.woff2")),
    ("barlow-semi-condensed-latin-ext-500-normal.woff2", include_bytes!("fonts/barlow-semi-condensed-latin-ext-500-normal.woff2")),
    ("barlow-semi-condensed-latin-600-normal.woff2", include_bytes!("fonts/barlow-semi-condensed-latin-600-normal.woff2")),
    ("barlow-semi-condensed-latin-ext-600-normal.woff2", include_bytes!("fonts/barlow-semi-condensed-latin-ext-600-normal.woff2")),
    ("barlow-semi-condensed-latin-700-normal.woff2", include_bytes!("fonts/barlow-semi-condensed-latin-700-normal.woff2")),
    ("barlow-semi-condensed-latin-ext-700-normal.woff2", include_bytes!("fonts/barlow-semi-condensed-latin-ext-700-normal.woff2")),
    ("cinzel-latin-500-normal.woff2", include_bytes!("fonts/cinzel-latin-500-normal.woff2")),
    ("cinzel-latin-ext-500-normal.woff2", include_bytes!("fonts/cinzel-latin-ext-500-normal.woff2")),
    ("cinzel-latin-600-normal.woff2", include_bytes!("fonts/cinzel-latin-600-normal.woff2")),
    ("cinzel-latin-ext-600-normal.woff2", include_bytes!("fonts/cinzel-latin-ext-600-normal.woff2")),
    ("cinzel-latin-700-normal.woff2", include_bytes!("fonts/cinzel-latin-700-normal.woff2")),
    ("cinzel-latin-ext-700-normal.woff2", include_bytes!("fonts/cinzel-latin-ext-700-normal.woff2")),
    ("ibm-plex-mono-latin-400-normal.woff2", include_bytes!("fonts/ibm-plex-mono-latin-400-normal.woff2")),
    ("ibm-plex-mono-latin-ext-400-normal.woff2", include_bytes!("fonts/ibm-plex-mono-latin-ext-400-normal.woff2")),
    ("ibm-plex-mono-latin-500-normal.woff2", include_bytes!("fonts/ibm-plex-mono-latin-500-normal.woff2")),
    ("ibm-plex-mono-latin-ext-500-normal.woff2", include_bytes!("fonts/ibm-plex-mono-latin-ext-500-normal.woff2")),
];
