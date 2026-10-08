/**
 * QFX layer one: the living backdrop.
 *
 * A WebGL2 canvas behind the whole interface, drawing a slow colour drift taken
 * from the active theme's own tokens, disturbed where the pointer is.
 *
 * # What this file is allowed to do, and what it is not
 *
 * The design system governs the chrome. QFX governs the canvas. Until ADR-080
 * (6 October 2026) `scripts/check-design.mjs` forbade canvases, frame loops and
 * pointer-reactive effects everywhere else in `src/` and exempted `src/qfx/`;
 * effects are now allowed everywhere. The obligations below stand: this
 * directory carries *stricter* duties than the chrome does, not looser ones.
 *
 * Three of them, all enforced rather than intended:
 *
 * 1. **Reduced motion means still.** Not "slower", not "gentler". One frame,
 *    then nothing. The setting is read from `data-ambience`, which `applyA11y`
 *    computes after reduced motion has had its say, so this file cannot get it
 *    wrong on its own.
 * 2. **A frame-time budget.** If drawing costs more than it is worth, the canvas
 *    stops animating and stays on its last frame. A theme cannot make the
 *    launcher unusable, because the launcher stops listening before it does.
 * 3. **The contrast guarantee is not here.** It is the chrome's scrim, in
 *    `qor.css`, and nothing this file draws can weaken it. See `contrast.ts`.
 *
 * Off means nothing is drawn and nothing is scheduled, and the stylesheet hides
 * the canvas as well, because Off also removes the scrim: a frame left on screen
 * with no scrim over it would sit outside the contrast guarantee. That was the
 * first version's defect, and `scripts/check-accessibility.mjs` now pins it.
 *
 * # Why WebGL2 and not WebGPU
 *
 * Tauri uses the system webview, so the answer differs per platform and the
 * baseline has to be the one that is everywhere. WebGL2 is. WebGPU is an
 * enhancement to reach for later, per platform, with evidence — not a baseline.
 * A canvas that fails to initialise is not an error here: the backdrop simply
 * does not appear, and the interface is unchanged.
 */

import { useEffect, useRef } from 'react';

import type { Ambience } from '../lib/a11y';

// ─── Backdrop ID type ────────────────────────────────────────────────────────

export type BackdropId = 'drift' | 'aurora' | 'nebula' | 'lattice' | 'starfield' | 'liquid';

const BACKDROP_STORAGE_KEY = 'qor.backdrop';
const DEFAULT_BACKDROP: BackdropId = 'drift';

/** Read the stored backdrop preference, or return the default. */
export function currentBackdrop(): BackdropId {
  try {
    const stored = localStorage.getItem(BACKDROP_STORAGE_KEY);
    if (stored && isBackdropId(stored)) return stored;
  } catch {
    /* fall through */
  }
  return DEFAULT_BACKDROP;
}

/**
 * Apply a backdrop: sets `data-backdrop` on `<html>` and saves to localStorage.
 * The QfxCanvas mutation observer picks up the attribute change automatically.
 */
export function applyBackdrop(id: BackdropId): void {
  document.documentElement.setAttribute('data-backdrop', id);
  try {
    localStorage.setItem(BACKDROP_STORAGE_KEY, id);
  } catch {
    /* A preference is not worth failing over. */
  }
}

function isBackdropId(value: string): value is BackdropId {
  return (
    value === 'drift' ||
    value === 'aurora' ||
    value === 'nebula' ||
    value === 'lattice' ||
    value === 'starfield' ||
    value === 'liquid'
  );
}

// ─── Backdrop presets ────────────────────────────────────────────────────────

export interface BackdropPreset {
  id: BackdropId;
  name: string;
  description: string;
}

export const BACKDROP_PRESETS: BackdropPreset[] = [
  {
    id: 'drift',
    name: 'Drift',
    description: 'Slow sinusoidal colour fields that drift past each other. Pointer lifts the field nearby.',
  },
  {
    id: 'aurora',
    name: 'Aurora',
    description: 'Vertical curtain waves with shimmer bands. Pointer creates a small aurora ripple.',
  },
  {
    id: 'nebula',
    name: 'Nebula',
    description: 'Radial clusters of soft luminance that breathe and drift. Pointer attracts nearby wisps.',
  },
  {
    id: 'lattice',
    name: 'Lattice',
    description: 'Procedural glowing grid lines that breathe and distort. Pointer warps the mesh locally.',
  },
  {
    id: 'starfield',
    name: 'Starfield',
    description: 'Two-layer parallax star field with slow drift. Pointer brightens the nearest cluster.',
  },
  {
    id: 'liquid',
    name: 'Liquid',
    description: 'Fluid Voronoi cells that divide and merge. Pointer creates a surface-tension ripple.',
  },
];

// ─── Shared vertex shader ─────────────────────────────────────────────────────

const VERTEX = `#version 300 es
in vec2 a_position;
void main() { gl_Position = vec4(a_position, 0.0, 1.0); }`;

// ─── Drift ────────────────────────────────────────────────────────────────────
// Two wide, slow gradients that drift past each other, plus a soft lift near
// the pointer. No detail, no edges, nothing that reads as content.

const FRAGMENT_DRIFT = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;
uniform float u_level;

out vec4 outColour;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  float complexity = 1.0 + clamp(u_level / 50.0, 0.0, 1.0) * 0.6;

  // Two drifts at different speeds so the pattern never visibly repeats.
  float a = sin(uv.x * 2.2 * complexity + u_time * 0.05) * 0.5 + 0.5;
  float b = cos(uv.y * 1.7 * complexity - u_time * 0.037) * 0.5 + 0.5;
  float field = (a * 0.6 + b * 0.4);

  // Level 26+ adds a third slow drift layer for richer blending.
  if (u_level > 25.0) {
    float lv = clamp((u_level - 25.0) / 25.0, 0.0, 1.0);
    float c = sin(uv.x * 1.1 + uv.y * 0.9 + u_time * 0.028) * 0.5 + 0.5;
    field = mix(field, (field * 0.7 + c * 0.3), lv * 0.4);
  }

  // Pointer lifts the field nearby.
  float d = distance(uv, u_pointer);
  float touch = exp(-d * d * 18.0) * 0.5;

  float mixed = clamp((field + touch) * u_amplitude, 0.0, 1.0);
  outColour = vec4(mix(u_base, u_accent, mixed), 1.0);
}`;

// ─── Aurora ───────────────────────────────────────────────────────────────────
// Vertical curtain waves that ripple with slow phase shifts, plus horizontal
// shimmer bands. Pointer injects a small localised ripple.

const FRAGMENT_AURORA = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;
uniform float u_level;

out vec4 outColour;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  float complexity = 1.0 + clamp(u_level / 50.0, 0.0, 1.0) * 0.5;

  // Curtain: multiple vertical sine waves with slow phase drift.
  float t = u_time * 0.12;
  float curtain  = sin(uv.x * 5.0 * complexity + t * 0.7) * 0.35;
  curtain       += sin(uv.x * 9.0 * complexity - t * 0.4 + 1.3) * 0.18;
  curtain       += sin(uv.x * 3.0 + t * 0.2) * 0.12;

  // Curtains are tallest near top, fade toward bottom.
  float vMask = smoothstep(0.0, 0.55, uv.y) * smoothstep(1.0, 0.45, uv.y);
  float band = smoothstep(-0.05, 0.35, uv.y + curtain) * vMask;

  // Shimmer: faint horizontal bands with independent drift.
  float shimmer = sin(uv.y * 18.0 * complexity + u_time * 0.6) * 0.12 + 0.12;
  shimmer *= smoothstep(1.0, 0.3, uv.y);

  // Pointer ripple: spreads outward from pointer position.
  float dp = distance(uv, u_pointer);
  float ripple = sin(dp * 28.0 - u_time * 3.0) * exp(-dp * 9.0) * 0.35;

  // Level 26+ adds a second, slower curtain layer.
  if (u_level > 25.0) {
    float lv = clamp((u_level - 25.0) / 25.0, 0.0, 1.0);
    float band2 = sin(uv.x * 4.5 * complexity + t * 0.35 + 2.1) * 0.22 * lv;
    band = mix(band, clamp(band + band2, 0.0, 1.0), 0.4);
  }

  float field = clamp((band + shimmer + ripple) * u_amplitude, 0.0, 1.0);
  outColour = vec4(mix(u_base, u_accent, field), 1.0);
}`;

// ─── Nebula ───────────────────────────────────────────────────────────────────
// Radial clusters of luminance that breathe. Hash-seeded positions, soft
// Gaussian falloffs. Pointer attracts wisps toward it.

const FRAGMENT_NEBULA = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;
uniform float u_level;

out vec4 outColour;

// Deterministic hash for procedural cluster positions.
vec2 hash2(float n) {
  return fract(vec2(sin(n * 127.1) * 43758.5453,
                    cos(n * 311.7) * 31415.9265));
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float t = u_time * 0.07;

  int numClouds = 5;
  if (u_level > 25.0) numClouds = 7;
  if (u_level > 50.0) numClouds = 9;

  float glow = 0.0;
  for (int i = 0; i < 9; i++) {
    if (i >= numClouds) break;
    float fi = float(i);
    vec2 seed = hash2(fi * 3.71 + 1.0);
    // Cluster centre drifts slowly on a Lissajous-like path.
    vec2 centre = seed * 0.8 + 0.1;
    centre.x += sin(t * (0.4 + seed.x * 0.5) + fi) * 0.12;
    centre.y += cos(t * (0.3 + seed.y * 0.4) + fi * 1.7) * 0.10;

    // Pointer attraction: wisps drift toward the pointer slightly.
    vec2 toPointer = u_pointer - centre;
    centre += toPointer * 0.12 * exp(-length(toPointer) * 4.0);

    float dist = distance(uv, centre);
    // Breathing: radius oscillates gently.
    float radius = 0.18 + 0.06 * sin(t * 1.3 + fi * 2.1);
    float brightness = 0.5 + 0.5 * seed.y;
    glow += brightness * exp(-dist * dist / (radius * radius));
  }

  glow = clamp(glow * u_amplitude * 0.55, 0.0, 1.0);
  outColour = vec4(mix(u_base, u_accent, glow), 1.0);
}`;

// ─── Lattice ──────────────────────────────────────────────────────────────────
// Thin glowing grid lines using SDF geometry. Pointer locally warps the mesh.
// Hash-based jitter adds organic imperfection. Level raises line count + glow.

const FRAGMENT_LATTICE = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;
uniform float u_level;

out vec4 outColour;

// Fast hash for jitter offsets.
float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// SDF: distance to the nearest grid line in the warped UV space.
float gridSDF(vec2 p, float scale) {
  vec2 g = abs(fract(p * scale) - 0.5);
  return min(g.x, g.y);
}

void main() {
  // Square cells at any aspect ratio: x is measured in heights, as y is.
  float aspect = u_resolution.x / u_resolution.y;
  vec2 uv = gl_FragCoord.xy / u_resolution;
  uv.x *= aspect;
  vec2 pointer = vec2(u_pointer.x * aspect, u_pointer.y);

  float gridScale = 6.0 + clamp(u_level / 25.0, 0.0, 3.0) * 2.0;
  float t = u_time * 0.08;

  // Organic breathing: low-frequency warp of the UV domain.
  float warpAmt = 0.03 + clamp(u_level / 100.0, 0.0, 1.0) * 0.02;
  vec2 warp;
  warp.x = sin(uv.y * 3.1 + t * 0.9) * warpAmt;
  warp.y = cos(uv.x * 2.7 - t * 0.7) * warpAmt;
  vec2 warped = uv + warp;

  // The pointer draws the mesh toward it, like a lens. The pull is proportional
  // to the distance, so it is zero at the pointer and smooth everywhere, and
  // never strong enough to fold a line over itself (a pull along a unit vector
  // tied the lines into a knot under the pointer).
  vec2 toPointer = pointer - uv;
  warped += toPointer * 0.45 * exp(-dot(toPointer, toPointer) * 10.0);

  // SDF distance to nearest grid line.
  float d = gridSDF(warped, gridScale);

  // Thin glow profile: bright at centre, fast exponential falloff.
  float lineWidth = 0.018 + clamp(u_level / 100.0, 0.0, 1.0) * 0.006;
  float glow = exp(-d * d / (lineWidth * lineWidth));

  // Level 26+ adds a diagonal lattice layer.
  if (u_level > 25.0) {
    float lv = clamp((u_level - 25.0) / 25.0, 0.0, 1.0);
    float diagScale = gridScale * 0.75;
    vec2 rotUV = vec2(warped.x + warped.y, warped.x - warped.y) * 0.707;
    float d2 = gridSDF(rotUV, diagScale);
    glow += lv * 0.5 * exp(-d2 * d2 / (lineWidth * lineWidth * 1.5));
  }

  // Jitter brightness per cell for organic feel.
  vec2 cell = floor(warped * gridScale);
  float cellBright = 0.75 + 0.25 * hash(cell + vec2(t * 0.05, 0.0));

  float field = clamp(glow * cellBright * u_amplitude, 0.0, 1.0);
  outColour = vec4(mix(u_base, u_accent, field), 1.0);
}`;

// ─── Starfield ────────────────────────────────────────────────────────────────
// Three depth planes of stars drifting at different speeds, twinkling, brighter
// near the pointer. One possible star per grid cell, and each pixel looks at its
// own cell only, so the cost does not grow with the number of stars: the first
// version looped over every star for every pixel, and CI's software renderer
// drew it at 11 frames a second. Round stars at any aspect ratio; the level
// fills the sky.

const FRAGMENT_STARFIELD = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;
uniform float u_level;

out vec4 outColour;

// A hash with no sin: the same on every GPU, and cheap in software.
vec2 hash22(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.xx + p3.yz) * p3.zy);
}

// One plane of stars: at most one star in each cell of a grid, kept to the
// middle of its cell so its light never reaches a neighbour. A pixel therefore
// looks at its own cell only: two hashes and one sine, whatever the star count.
float starLayer(vec2 p, float density, float sizeScale, float seed) {
  vec2 cell = floor(p);
  vec2 h = hash22(cell + seed);
  if (h.x > density) return 0.0;
  vec2 star = 0.25 + 0.5 * hash22(cell + seed + 17.0);
  vec2 d = fract(p) - star;
  float size = (0.03 + 0.045 * h.y) * sizeScale;
  float twinkle = 0.65 + 0.35 * sin(u_time * (1.2 + 2.0 * h.y) + h.x * 40.0);
  float falloff = max(0.0, 1.0 - dot(d, d) / (9.0 * size * size));
  return twinkle * falloff * falloff * falloff;
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float aspect = u_resolution.x / u_resolution.y;
  vec2 p = vec2(uv.x * aspect, uv.y);

  // The share of cells that hold a star: the sky fills as the level rises.
  float density = 0.55 + 0.3 * clamp(u_level / 50.0, 0.0, 1.0);

  vec2 drift = vec2(u_time * 0.006, u_time * 0.002);
  float near = starLayer((p + drift) * 9.0, density, 1.0, 0.0);
  float mid = starLayer((p + drift * 0.6) * 15.0, density, 0.9, 53.0) * 0.7;
  float far = starLayer((p + drift * 0.35) * 26.0, density, 0.8, 91.0) * 0.45;

  // The pointer brightens the stars around it.
  vec2 pointer = vec2(u_pointer.x * aspect, u_pointer.y);
  float dp = distance(p, pointer);
  float boost = exp(-dp * dp * 14.0);

  float stars = (near + mid + far) * (1.0 + boost * 1.5) + boost * 0.25;
  stars = clamp(stars * u_amplitude, 0.0, 1.0);
  outColour = vec4(mix(u_base, u_accent, stars), 1.0);
}`;

// ─── Liquid ───────────────────────────────────────────────────────────────────
// Voronoi cells that animate: each cell centre drifts on its own Lissajous
// path. Iridescent edge glow from proximity to nearest two centres.
// Pointer creates a surface-tension ripple.

const FRAGMENT_LIQUID = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;
uniform float u_level;

out vec4 outColour;

// A hash with no sin: the same on every GPU, and cheap in software.
vec2 hash22(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.xx + p3.yz) * p3.zy);
}

// A smooth back-and-forth from -1 to 1 with no sin: a triangle wave, eased.
vec2 sway(vec2 x) {
  vec2 t = abs(fract(x) - 0.5) * 2.0;
  return (t * t * (3.0 - 2.0 * t)) * 2.0 - 1.0;
}

// Where a cell's centre is now: inside its own cell, drifting on a slow path.
vec2 centre(vec2 cellId) {
  vec2 seed = hash22(cellId);
  return 0.2 + 0.6 * seed + 0.18 * sway(seed + u_time * 0.02 * (0.6 + seed * 0.5));
}

// Animated Voronoi: (distance to the nearest centre, distance to the border
// between the two nearest cells). The border is measured to the bisector of
// the two nearest centres, so it has the same width everywhere (the difference
// of the two distances, used first, swelled into bright wedges), and in one
// pass over the nine cells.
vec2 voronoi(vec2 uv, float scale) {
  vec2 p = uv * scale;
  vec2 i = floor(p);
  vec2 f = fract(p);

  vec2 r1 = vec2(9.9);
  vec2 r2 = vec2(9.9);
  float d1 = 99.0;
  float d2 = 99.0;
  for (int y = -1; y <= 1; y++) {
    for (int x = -1; x <= 1; x++) {
      vec2 o = vec2(float(x), float(y));
      vec2 r = o + centre(i + o) - f;
      float d = dot(r, r);
      if (d < d1) { d2 = d1; r2 = r1; d1 = d; r1 = r; }
      else if (d < d2) { d2 = d; r2 = r; }
    }
  }
  float border = dot(0.5 * (r1 + r2), normalize(r2 - r1));
  return vec2(sqrt(d1), border);
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  float cellScale = 3.5 + clamp(u_level / 33.0, 0.0, 2.0);

  // Pointer surface-tension ripple: a radial wave from the pointer. Its push is
  // proportional to the distance, so it is smooth through the pointer itself
  // (a push along a unit vector tore the cells there).
  vec2 fromPointer = uv - u_pointer;
  float dp = length(fromPointer);
  vec2 rippleUV = uv + fromPointer * sin(dp * 22.0 - u_time * 4.0) * exp(-dp * 7.0) * 0.18;

  vec2 vd = voronoi(rippleUV, cellScale);
  float nearest = vd.x;
  float border = vd.y;

  // Edge glow: bright along cell borders, the same width everywhere.
  float edge = 1.0 - smoothstep(0.0, 0.05, border);
  // Interior shimmer: faint variation across the cell face.
  float interior = smoothstep(0.4, 0.0, nearest) * 0.35;
  // Iridescent shift based on cell distance (creates colour variance feel).
  float iridescence = sin(nearest * 12.0 + u_time * 0.4) * 0.12 + 0.12;

  float field = clamp((edge * 0.7 + interior + iridescence) * u_amplitude, 0.0, 1.0);

  // Level 51+: wider, more luminous edges.
  if (u_level > 50.0) {
    float lv = clamp((u_level - 50.0) / 50.0, 0.0, 1.0);
    float wideEdge = 1.0 - smoothstep(0.0, 0.1, border);
    field = clamp(mix(field, field + wideEdge * 0.3, lv * 0.5), 0.0, 1.0);
  }

  outColour = vec4(mix(u_base, u_accent, field), 1.0);
}`;

// ─── Fragment map ─────────────────────────────────────────────────────────────

const FRAGMENTS: Record<BackdropId, string> = {
  drift: FRAGMENT_DRIFT,
  aurora: FRAGMENT_AURORA,
  nebula: FRAGMENT_NEBULA,
  lattice: FRAGMENT_LATTICE,
  starfield: FRAGMENT_STARFIELD,
  liquid: FRAGMENT_LIQUID,
};

// ─── Constants ────────────────────────────────────────────────────────────────

/** Restrained on purpose. The dreamlike look is a creator's choice, not the default. */
const DEFAULT_AMPLITUDE = 0.35;

/** Over this, for this many frames in a row, and the canvas stops animating. */
const FRAME_BUDGET_MS = 24;
const BUDGET_STRIKES = 30;

// ─── Token reader ─────────────────────────────────────────────────────────────

function readToken(name: string, fallback: [number, number, number]): [number, number, number] {
  const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const m = /^#?([0-9a-f]{6})$/i.exec(raw) ?? /rgba?\(([^)]+)\)/i.exec(raw);
  if (!m) return fallback;
  if (m[1] && /^[0-9a-f]{6}$/i.test(m[1])) {
    const hex = m[1];
    return [
      parseInt(hex.slice(0, 2), 16) / 255,
      parseInt(hex.slice(2, 4), 16) / 255,
      parseInt(hex.slice(4, 6), 16) / 255,
    ];
  }
  const parts = m[1]!.split(/[,\s/]+/).filter(Boolean).map(Number);
  if (parts.length < 3) return fallback;
  return [parts[0]! / 255, parts[1]! / 255, parts[2]! / 255];
}

// ─── Ambience helper ──────────────────────────────────────────────────────────

function currentAmbience(): Ambience {
  const value = document.documentElement.getAttribute('data-ambience');
  return value === 'off' || value === 'still' ? value : 'live';
}

// ─── React component ──────────────────────────────────────────────────────────

export function QfxCanvas() {
  const ref = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;

    // Created the first time the backdrop is wanted, not before: a launcher
    // that opens with ambience off never creates a GL context at all.
    let backdrop: Backdrop | null = null;
    let unavailable = false;
    let activeId: BackdropId = currentBackdrop();

    // Initialise the data-backdrop attribute from stored preference on mount,
    // so the mutation observer can track future changes.
    document.documentElement.setAttribute('data-backdrop', activeId);

    const follow = () => {
      const ambience = currentAmbience();
      // The attribute, not storage: storage can refuse a write, and the choice
      // must still take effect for this session.
      const attribute = document.documentElement.getAttribute('data-backdrop') ?? '';
      const wantedId = isBackdropId(attribute) ? attribute : activeId;

      // A different backdrop: tear down the old one, and build the new one if
      // ambience wants anything drawn. Taken even when nothing is built yet
      // (ambience off), so turning ambience on later builds the chosen one.
      if (wantedId !== activeId) {
        backdrop?.dispose();
        backdrop = null;
        unavailable = false;
        activeId = wantedId;
      }

      if (!backdrop && !unavailable && ambience !== 'off') {
        backdrop = createBackdrop(canvas, activeId);
        unavailable = backdrop === null;
      }
      backdrop?.show(ambience);
    };

    follow();

    // The setting can change while the launcher is open: ambience (movement),
    // backdrop (variant), or theme (colours).
    const observer = new MutationObserver(follow);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-ambience', 'data-backdrop'],
    });

    return () => {
      observer.disconnect();
      backdrop?.dispose();
    };
  }, []);

  return <canvas ref={ref} className="qfx-canvas" aria-hidden="true" />;
}

// ─── Backdrop interface ───────────────────────────────────────────────────────

interface Backdrop {
  /** Move to an ambience: live runs the loop, still draws one frame, off draws nothing. */
  show(ambience: Ambience): void;
  dispose(): void;
}

// ─── GL backdrop factory ──────────────────────────────────────────────────────

/** The GL side. `null` when WebGL2 is unavailable, which is not an error. */
function createBackdrop(
  canvas: HTMLCanvasElement,
  backdropId: BackdropId = 'drift',
): Backdrop | null {
  const gl = canvas.getContext('webgl2', {
    alpha: false,
    antialias: false,
    powerPreference: 'low-power',
    preserveDrawingBuffer: false,
  });
  // No WebGL2: no backdrop. Not an error, and nothing else changes.
  if (!gl) return null;

  const compile = (type: number, source: string) => {
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      gl.deleteShader(shader);
      return null;
    }
    return shader;
  };

  const fragmentSource = FRAGMENTS[backdropId];
  const vs = compile(gl.VERTEX_SHADER, VERTEX);
  const fs = compile(gl.FRAGMENT_SHADER, fragmentSource);
  if (!vs || !fs) return null;

  const program = gl.createProgram()!;
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  // Linked or not, the shaders are not needed again.
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    gl.deleteProgram(program);
    return null;
  }
  gl.useProgram(program);

  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  const position = gl.getAttribLocation(program, 'a_position');
  gl.enableVertexAttribArray(position);
  gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);

  const u = {
    resolution: gl.getUniformLocation(program, 'u_resolution'),
    time: gl.getUniformLocation(program, 'u_time'),
    pointer: gl.getUniformLocation(program, 'u_pointer'),
    base: gl.getUniformLocation(program, 'u_base'),
    accent: gl.getUniformLocation(program, 'u_accent'),
    amplitude: gl.getUniformLocation(program, 'u_amplitude'),
    level: gl.getUniformLocation(program, 'u_level'),
  };

  const pointer = { x: 0.5, y: 0.5 };
  const onPointer = (event: PointerEvent) => {
    pointer.x = event.clientX / window.innerWidth;
    pointer.y = 1 - event.clientY / window.innerHeight;
  };

  const resize = () => {
    // Capped device pixel ratio: a 4K display at native resolution costs more
    // than a backdrop is worth, and nobody can see the difference in a blur.
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    canvas.width = Math.floor(window.innerWidth * dpr);
    canvas.height = Math.floor(window.innerHeight * dpr);
    gl.viewport(0, 0, canvas.width, canvas.height);
  };
  resize();

  /** Read the current level from a data attribute, defaulting to 0. */
  const readLevel = (): number => {
    const raw = document.documentElement.getAttribute('data-level');
    if (!raw) return 0;
    const n = parseFloat(raw);
    return Number.isFinite(n) ? Math.max(0, n) : 0;
  };

  let ambience: Ambience = 'off';
  let frame = 0;
  let strikes = 0;
  let running = false;
  let last = performance.now();
  const started = last;

  const draw = (now: number) => {
    const base = readToken('--base', [0.04, 0.05, 0.06]);
    const accent = readToken('--accent', [0.3, 0.35, 0.45]);

    gl.uniform2f(u.resolution, canvas.width, canvas.height);
    gl.uniform1f(u.time, (now - started) / 1000);
    gl.uniform2f(u.pointer, pointer.x, pointer.y);
    gl.uniform3f(u.base, base[0], base[1], base[2]);
    gl.uniform3f(u.accent, accent[0], accent[1], accent[2]);
    gl.uniform1f(u.amplitude, DEFAULT_AMPLITUDE);
    gl.uniform1f(u.level, readLevel());
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  };

  const halt = () => {
    running = false;
    cancelAnimationFrame(frame);
    window.removeEventListener('pointermove', onPointer);
  };

  const loop = (now: number) => {
    if (!running) return;

    const cost = now - last;
    last = now;

    // The budget. Persistent overrun means this machine cannot afford the
    // backdrop, so it keeps the last frame and stops. Better a still image
    // than an interface that stutters while someone is trying to work.
    if (cost > FRAME_BUDGET_MS) {
      strikes += 1;
      if (strikes >= BUDGET_STRIKES) {
        halt();
        document.documentElement.setAttribute('data-ambience-stopped', 'budget');
        return;
      }
    } else if (strikes > 0) {
      strikes -= 1;
    }

    draw(now);
    frame = requestAnimationFrame(loop);
  };

  const show = (next: Ambience) => {
    halt();
    ambience = next;
    strikes = 0;
    document.documentElement.removeAttribute('data-ambience-stopped');

    if (next === 'live') {
      running = true;
      window.addEventListener('pointermove', onPointer, { passive: true });
      last = performance.now();
      frame = requestAnimationFrame(loop);
    } else if (next === 'still') {
      // Still: one frame, then nothing. No listener, so the pointer does not
      // move it either — "still" has to mean still.
      pointer.x = 0.5;
      pointer.y = 0.5;
      draw(performance.now());
    }
    // Off: nothing drawn and nothing scheduled. The stylesheet hides the canvas.
  };

  const onResize = () => {
    resize();
    // A resize clears the drawing buffer. Redraw once unless the loop will.
    if (ambience !== 'off' && !running) draw(performance.now());
  };
  window.addEventListener('resize', onResize);

  return {
    show,
    dispose: () => {
      halt();
      window.removeEventListener('resize', onResize);
      // The context outlives this backdrop (a canvas has one), so the next
      // backdrop built on it must not inherit this one's program and buffer.
      gl.deleteBuffer(buffer);
      gl.deleteProgram(program);
    },
  };
}
