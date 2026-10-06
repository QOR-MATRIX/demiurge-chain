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

const VERTEX = `#version 300 es
in vec2 a_position;
void main() { gl_Position = vec4(a_position, 0.0, 1.0); }`;

// Two wide, slow gradients that drift past each other, plus a soft lift near the
// pointer. No detail, no edges, nothing that reads as content: this sits behind
// text and its job is to be almost unnoticed.
const FRAGMENT = `#version 300 es
precision mediump float;

uniform vec2  u_resolution;
uniform float u_time;
uniform vec2  u_pointer;
uniform vec3  u_base;
uniform vec3  u_accent;
uniform float u_amplitude;

out vec4 outColour;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Two drifts at different speeds so the pattern never visibly repeats.
  float a = sin(uv.x * 2.2 + u_time * 0.05) * 0.5 + 0.5;
  float b = cos(uv.y * 1.7 - u_time * 0.037) * 0.5 + 0.5;
  float field = (a * 0.6 + b * 0.4);

  // The pointer lifts the field nearby. Falls off fast, so it reads as a
  // presence rather than a spotlight following the cursor.
  float d = distance(uv, u_pointer);
  float touch = exp(-d * d * 18.0) * 0.5;

  float mixed = clamp((field + touch) * u_amplitude, 0.0, 1.0);
  outColour = vec4(mix(u_base, u_accent, mixed), 1.0);
}`;

/** Restrained on purpose. The dreamlike look is a creator's choice, not the default. */
const DEFAULT_AMPLITUDE = 0.35;

/** Over this, for this many frames in a row, and the canvas stops animating. */
const FRAME_BUDGET_MS = 24;
const BUDGET_STRIKES = 30;

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

function currentAmbience(): Ambience {
  const value = document.documentElement.getAttribute('data-ambience');
  return value === 'off' || value === 'still' ? value : 'live';
}

export function QfxCanvas() {
  const ref = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;

    // Created the first time the backdrop is wanted, not before: a launcher that
    // opens with ambience off never creates a GL context at all. The observer is
    // attached either way, so the setting can be turned on later without a
    // restart.
    let backdrop: Backdrop | null = null;
    let unavailable = false;

    const follow = () => {
      const ambience = currentAmbience();
      if (!backdrop && !unavailable && ambience !== 'off') {
        backdrop = createBackdrop(canvas);
        unavailable = backdrop === null;
      }
      backdrop?.show(ambience);
    };

    follow();

    // The setting can change while the launcher is open, and the canvas has to
    // follow it immediately — especially into `still` and `off`.
    const observer = new MutationObserver(follow);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-ambience'],
    });

    return () => {
      observer.disconnect();
      backdrop?.dispose();
    };
  }, []);

  return <canvas ref={ref} className="qfx-canvas" aria-hidden="true" />;
}

interface Backdrop {
  /** Move to an ambience: live runs the loop, still draws one frame, off draws nothing. */
  show(ambience: Ambience): void;
  dispose(): void;
}

/** The GL side. `null` when WebGL2 is unavailable, which is not an error. */
function createBackdrop(canvas: HTMLCanvasElement): Backdrop | null {
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

  const vs = compile(gl.VERTEX_SHADER, VERTEX);
  const fs = compile(gl.FRAGMENT_SHADER, FRAGMENT);
  if (!vs || !fs) return null;

  const program = gl.createProgram()!;
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return null;
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
    },
  };
}
