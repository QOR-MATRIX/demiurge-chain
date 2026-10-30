/*
  qorsync.dev: the living background and the page's responses to touch and pointer.

  The aether (ADR-127) is one WebGL shader behind the page: QOR OS's mark as a lens at the scale of the sky. Its ring is
  the edge of a dark mass that bends the light around it; a nebula of ember and gilt drifts behind, folded on itself,
  with stars, all of it seen through the lens; a tilted disc of burning matter circles the mass, its far side bent up
  over the top; and the mark's tail, an ember, orbits just outside the ring. A finger or the pointer carries a small
  mass of its own that bends the sky where it goes, and a tap sends a ring of light through it.

  It is decoration only: every word on the page is HTML. With reduced motion asked for, one still frame is drawn; while
  the tab is hidden nothing is drawn; on a phone left alone it draws at half the rate; and it is drawn at a fraction of
  the screen's resolution that it lowers by itself if frames come late. Without WebGL the page keeps its painted ground.
*/
(() => {
  'use strict';

  const root = document.documentElement;
  root.classList.add('js');
  const still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const fine = window.matchMedia('(pointer: fine)').matches;

  /* ── the bar darkens once the page moves ─────────────────────────────── */
  const bar = document.querySelector('.bar');
  const onScroll = () => bar && bar.classList.toggle('scrolled', window.scrollY > 24);
  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();

  /* ── the menu, on narrow screens ─────────────────────────────────────── */
  const toggle = document.querySelector('.menu-toggle');
  const menu = document.getElementById('menu');
  if (toggle && menu) {
    const setOpen = (open) => {
      toggle.setAttribute('aria-expanded', String(open));
      toggle.setAttribute('aria-label', open ? 'Close the menu' : 'Open the menu');
      root.classList.toggle('menu-open', open);
      if (open) menu.querySelector('a')?.focus({ preventScroll: true });
    };
    toggle.addEventListener('click', () => setOpen(toggle.getAttribute('aria-expanded') !== 'true'));
    menu.addEventListener('click', (e) => { if (e.target.closest('a')) setOpen(false); });
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && root.classList.contains('menu-open')) {
        setOpen(false);
        toggle.focus();
      }
    });
    window.matchMedia('(min-width: 861px)').addEventListener('change', (m) => { if (m.matches) setOpen(false); });
  }

  /* ── the download for this device comes first ────────────────────────── */
  const ua = navigator.userAgent;
  const platform = /Android/i.test(ua) ? 'android' : /Windows/i.test(ua) ? 'windows' : '';
  if (platform) {
    document.querySelectorAll(`[data-platform="${platform}"]`).forEach((el) => el.classList.add('for-you'));
    root.dataset.platform = platform;
  }

  /* ── sections ease in as they arrive; everything shows within a moment regardless ── */
  const reveals = [...document.querySelectorAll('.reveal')];
  const show = (el) => el.classList.add('in');
  if (still || !('IntersectionObserver' in window)) {
    reveals.forEach(show);
  } else {
    const seen = new IntersectionObserver(
      (entries) => entries.forEach((e) => {
        if (e.isIntersecting) {
          const siblings = [...e.target.parentElement.children].filter((n) => n.classList.contains('reveal'));
          e.target.style.transitionDelay = `${Math.min(siblings.indexOf(e.target), 5) * 70}ms`;
          show(e.target);
          seen.unobserve(e.target);
        }
      }),
      { rootMargin: '0px 0px -8% 0px', threshold: 0.08 },
    );
    reveals.forEach((el) => seen.observe(el));
    window.setTimeout(() => reveals.forEach(show), 2400);
  }

  /* ── cards tilt toward the pointer, and carry a light where it is ──────── */
  if (fine && !still) {
    document.querySelectorAll('[data-tilt]').forEach((card) => {
      card.addEventListener('pointermove', (e) => {
        const r = card.getBoundingClientRect();
        const x = (e.clientX - r.left) / r.width;
        const y = (e.clientY - r.top) / r.height;
        card.style.setProperty('--ry', `${(x - 0.5) * 9}deg`);
        card.style.setProperty('--rx', `${(0.5 - y) * 9}deg`);
        card.style.setProperty('--mx', `${x * 100}%`);
        card.style.setProperty('--my', `${y * 100}%`);
      });
      card.addEventListener('pointerleave', () => {
        card.style.setProperty('--rx', '0deg');
        card.style.setProperty('--ry', '0deg');
      });
    });

    /* buttons lean a little toward the pointer */
    document.querySelectorAll('.btn').forEach((btn) => {
      btn.addEventListener('pointermove', (e) => {
        const r = btn.getBoundingClientRect();
        btn.style.setProperty('--tx', `${((e.clientX - r.left) / r.width - 0.5) * 6}px`);
        btn.style.setProperty('--ty', `${((e.clientY - r.top) / r.height - 0.5) * 6}px`);
      });
      btn.addEventListener('pointerleave', () => {
        btn.style.setProperty('--tx', '0px');
        btn.style.setProperty('--ty', '0px');
      });
    });
  }

  /* ── the aether ──────────────────────────────────────────────────────── */
  const canvas = document.getElementById('aether');
  if (!canvas) return;

  const VERTEX = `
    attribute vec2 a;
    void main() { gl_Position = vec4(a, 0.0, 1.0); }
  `;

  // Scene units: the viewport is one unit high, y up, the origin at its centre.
  const FRAGMENT = `
    precision PRECISION float;
    uniform vec2 uRes;
    uniform float uTime;
    uniform float uScroll;   // viewports scrolled
    uniform vec3 uLens;      // the lens: x, y (0 to 1 from the top left) and its ring's radius, in viewport heights
    uniform vec3 uPtr;       // the pointer's mass: x, y, and how much there is of it (0 to 1)
    uniform vec4 uRip[4];    // rings from taps: x, y, age in seconds, strength
    uniform float uGlow;     // 1 on the first screen, less further down

    const vec3 OBSIDIAN = vec3(0.024, 0.027, 0.039);
    const vec3 EMBER = vec3(1.0, 0.416, 0.0);
    const vec3 GILT = vec3(0.788, 0.643, 0.361);
    const vec3 GILT_BRIGHT = vec3(0.918, 0.827, 0.604);

    float sq(float x) { return x * x; }
    float hash(vec2 p) {
      p = fract(p * vec2(123.34, 456.21));
      p += dot(p, p + 45.32);
      return fract(p.x * p.y);
    }
    float noise(vec2 p) {
      vec2 i = floor(p);
      vec2 f = fract(p);
      vec2 u = f * f * (3.0 - 2.0 * f);
      return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
    }
    float fbm(vec2 p) {
      float v = 0.0;
      float a = 0.5;
      for (int i = 0; i < 5; i++) {
        v += a * noise(p);
        p = mat2(1.6, 1.2, -1.2, 1.6) * p + 7.3;
        a *= 0.5;
      }
      return v;
    }
    vec2 scene(vec2 uv) {
      return vec2((uv.x - 0.5) * uRes.x / uRes.y, 0.5 - uv.y);
    }
    // Light passing a mass at c bends toward it: a point near it is seen from further in, which draws the sky into a
    // ring about the mass.
    vec2 bend(vec2 p, vec2 c, float rs) {
      vec2 d = p - c;
      float r = max(length(d), 1e-4);
      return p - d / r * (rs * rs / max(r, 0.6 * rs));
    }
    // The disc, in the mass's frame: tilted, seen nearly edge on, turning faster inside, hotter inside, and brighter on
    // the side that comes toward us.
    vec3 disc(vec2 p, vec2 c, float rs, float t) {
      vec2 d = p - c;
      d = mat2(0.949, -0.315, 0.315, 0.949) * d;
      vec2 e = vec2(d.x, d.y / 0.24);
      float r = length(e) / rs;
      float a = atan(e.y, e.x);
      float band = smoothstep(1.9, 2.5, r) * (1.0 - smoothstep(4.2, 6.0, r));
      if (band <= 0.0) return vec3(0.0);
      float spin = a + t * 0.9 / pow(max(r, 1.0), 1.5);
      float turb = fbm(vec2(spin * 2.6, r * 1.9 - t * 0.15));
      float streaks = 0.6 + 0.4 * sin(spin * 7.0 + turb * 7.0 + r * 2.4);
      float heat = pow(1.9 / max(r, 1.9), 1.6);
      float toward = 1.0 + 0.7 * cos(a);
      vec3 col = mix(EMBER, GILT_BRIGHT, smoothstep(0.35, 1.0, heat));
      col = mix(col, vec3(1.0, 0.93, 0.8), smoothstep(0.75, 1.0, heat) * 0.6);
      return col * band * heat * (0.35 + 0.65 * turb) * streaks * toward;
    }

    void main() {
      vec2 uv = gl_FragCoord.xy / uRes;
      uv.y = 1.0 - uv.y;
      vec2 p = scene(uv);
      float t = uTime;

      // Rings from taps: space is pushed outward as each passes, and lit where it is.
      float flash = 0.0;
      for (int i = 0; i < 4; i++) {
        vec4 R = uRip[i];
        if (R.w > 0.0) {
          vec2 d = p - scene(R.xy);
          float r = length(d);
          float ring = exp(-sq((r - R.z * 0.75) * 18.0)) * R.w * exp(-R.z * 1.3);
          p -= d / max(r, 1e-3) * ring * 0.03;
          flash += ring;
        }
      }

      // The pointer's own mass.
      vec2 P = scene(uPtr.xy);
      float ps = 0.045 * uPtr.z;
      float pr = length(p - P);
      vec2 q = uPtr.z > 0.001 ? bend(p, P, ps) : p;

      // The lens.
      vec2 L = scene(uLens.xy);
      float rs = uLens.z;
      float r = length(q - L);
      vec2 w = bend(q, L, rs * 1.15);

      // The sky behind it: a nebula folded on itself, drifting, seen through both masses.
      vec2 s = w * 1.7 + vec2(t * 0.012, uScroll * 0.45);
      vec2 warp = vec2(fbm(s * 1.2 + vec2(0.0, t * 0.035)), fbm(s * 1.2 + vec2(5.2, -t * 0.03)));
      float n = fbm(s + 2.4 * warp);
      float m = fbm(s * 2.3 - 1.7 * warp + 9.1);
      vec3 col = OBSIDIAN;
      col += EMBER * pow(n, 3.0) * 0.6;
      col += GILT * pow(m, 3.6) * 0.38;
      col += vec3(0.30, 0.10, 0.03) * pow(warp.y, 3.0) * 0.4;

      // Stars, in two layers, bent like the rest.
      for (int l = 0; l < 2; l++) {
        float k = l == 0 ? 70.0 : 140.0;
        vec2 g = w * k + float(l) * 13.1;
        vec2 id = floor(g);
        vec2 f = fract(g) - 0.5;
        float h = hash(id);
        vec2 off = vec2(hash(id + 3.1), hash(id + 7.7)) - 0.5;
        float star = smoothstep(0.12, 0.0, length(f - off * 0.6)) * step(0.975, h);
        star *= 0.55 + 0.45 * sin(t * (1.5 + h * 3.0) + h * 40.0);
        col += mix(GILT_BRIGHT, vec3(1.0), 0.5) * star * (l == 0 ? 1.0 : 0.6);
      }

      // Nothing comes out of the ring. Outside it the disc is seen through the lens, so its far side shows bent over
      // the top; across it, the near side of the disc passes in front.
      float shadow = smoothstep(rs * 0.92, rs * 1.08, r);
      col *= shadow;
      col += disc(w, L, rs, t) * shadow * 0.9;
      vec2 dq = mat2(0.949, -0.315, 0.315, 0.949) * (q - L);
      col += disc(q, L, rs, t) * (1.0 - shadow) * smoothstep(0.0, -0.01, dq.y) * 1.1;

      // The far side of the disc, bent up and over the ring into a crown, brightest on top and on the side that
      // comes toward us, turning with the disc.
      float phi = atan(q.y - L.y, q.x - L.x);
      float cr = (r - rs * 1.12) / (rs * 0.55);
      float crown = smoothstep(0.0, 0.12, cr) * (1.0 - smoothstep(0.3, 1.0, cr));
      float over = 0.2 + 0.8 * smoothstep(-0.25, 0.85, sin(phi));
      float swirl = fbm(vec2(phi * 3.0 - t * 0.7, cr * 2.5 + t * 0.1));
      col += mix(EMBER, GILT_BRIGHT, 0.6 + 0.4 * (1.0 - cr)) * crown * over * (0.3 + 0.95 * swirl) * (1.0 + 0.5 * cos(phi)) * 0.6;

      // The ring itself, the mark's, in gilt, with an ember haze.
      col += GILT_BRIGHT * exp(-sq((r - rs * 1.1) / (rs * 0.05))) * 0.9;
      col += EMBER * exp(-sq((r - rs * 1.1) / (rs * 0.22))) * 0.15;

      // The mark's tail: an ember orbiting just outside the ring, swaying about the lower right where the mark puts it.
      float ta = -0.785 + sin(t * 0.25) * 0.35;
      vec2 T = L + vec2(cos(ta), sin(ta)) * rs * 1.75;
      float td = length(q - T);
      col += EMBER * exp(-td / (rs * 0.16)) * 0.9 + vec3(1.0, 0.75, 0.45) * exp(-td / (rs * 0.05)) * 0.8;

      // The pointer's mass: a dark core in a thin gilt ring.
      col *= mix(1.0, smoothstep(ps * 0.55, ps * 1.05, pr), uPtr.z);
      col += GILT_BRIGHT * exp(-sq((pr - ps * 1.2) / max(ps * 0.12, 1e-4))) * uPtr.z * 0.55;

      col += mix(EMBER, GILT_BRIGHT, 0.4) * flash * 0.35;

      // Quieter below the first screen, darker toward the edges, and kept within range.
      col *= mix(0.45, 1.0, uGlow);
      col *= 1.0 - 0.45 * pow(length((uv - 0.5) * vec2(0.9, 1.1)), 2.0);
      col = vec3(1.0) - exp(-col * 1.35);
      gl_FragColor = vec4(col, 1.0);
    }
  `;

  let gl = null;
  let program = null;
  let u = {};

  function compile(type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.warn('aether:', gl.getShaderInfoLog(shader));
      return null;
    }
    return shader;
  }

  function init() {
    gl = canvas.getContext('webgl', { alpha: false, antialias: false, depth: false, stencil: false, powerPreference: 'default' });
    if (!gl) return false;
    const high = gl.getShaderPrecisionFormat(gl.FRAGMENT_SHADER, gl.HIGH_FLOAT);
    const vs = compile(gl.VERTEX_SHADER, VERTEX);
    const fs = compile(gl.FRAGMENT_SHADER, FRAGMENT.replace('PRECISION', high && high.precision > 0 ? 'highp' : 'mediump'));
    if (!vs || !fs) return false;
    program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return false;
    gl.useProgram(program);
    // One triangle that covers the screen.
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    const a = gl.getAttribLocation(program, 'a');
    gl.enableVertexAttribArray(a);
    gl.vertexAttribPointer(a, 2, gl.FLOAT, false, 0, 0);
    for (const name of ['uRes', 'uTime', 'uScroll', 'uLens', 'uPtr', 'uRip', 'uGlow']) u[name] = gl.getUniformLocation(program, name);
    return true;
  }

  // The fraction of the screen's pixels drawn, lowered when frames come late and raised again when there is room.
  const coarse = !fine;
  let quality = coarse ? 0.66 : 0.62;
  const QUALITY_MAX = coarse ? 0.85 : 0.8;
  const QUALITY_MIN = 0.28;

  function size() {
    const w = canvas.clientWidth || window.innerWidth;
    const h = canvas.clientHeight || window.innerHeight;
    canvas.width = Math.max(1, Math.round(w * quality));
    canvas.height = Math.max(1, Math.round(h * quality));
    if (gl) gl.viewport(0, 0, canvas.width, canvas.height);
  }

  /* where the lens sits: high on the right of a wide screen, above the words on a narrow one */
  function lens() {
    const w = window.innerWidth;
    const h = window.innerHeight;
    const wide = w > 860 && w / h > 1.05;
    const drift = (window.scrollY / h) * 0.35;
    return wide ? [0.74, 0.42 - drift, 0.068] : [0.6, 0.155 - drift, w / h < 0.7 ? 0.042 : 0.05];
  }

  const pointer = { x: 0.5, y: 0.5, tx: 0.5, ty: 0.5, mass: 0, target: 0, down: false, lastMove: -1e9 };
  const rings = [];
  let lastInput = -1e9;

  const at = (e) => [e.clientX / window.innerWidth, e.clientY / window.innerHeight];
  window.addEventListener('pointermove', (e) => {
    [pointer.tx, pointer.ty] = at(e);
    if (pointer.mass < 0.01) { pointer.x = pointer.tx; pointer.y = pointer.ty; }
    pointer.lastMove = performance.now();
    lastInput = pointer.lastMove;
    if (e.pointerType !== 'mouse') pointer.down = e.buttons > 0 || e.pressure > 0;
  }, { passive: true });
  window.addEventListener('pointerdown', (e) => {
    [pointer.tx, pointer.ty] = at(e);
    if (pointer.mass < 0.01) { pointer.x = pointer.tx; pointer.y = pointer.ty; }
    pointer.down = e.pointerType !== 'mouse';
    pointer.lastMove = performance.now();
    lastInput = pointer.lastMove;
    if (still) return;
    rings.push({ x: pointer.tx, y: pointer.ty, born: clock });
    if (rings.length > 4) rings.shift();
  }, { passive: true });
  const lift = () => { pointer.down = false; };
  window.addEventListener('pointerup', lift, { passive: true });
  window.addEventListener('pointercancel', lift, { passive: true });
  document.addEventListener('pointerleave', () => { pointer.lastMove = -1e9; });
  window.addEventListener('scroll', () => { lastInput = performance.now(); }, { passive: true });

  let clock = 20; // seconds of the scene's own time
  let last = performance.now();
  let running = !document.hidden;
  let frames = 0;
  let spent = 0;
  let skipped = 0;

  function draw() {
    if (!gl) return;
    const [lx, ly, lr] = lens();
    const now = performance.now();
    // A mouse's mass stays while it moves and a few seconds after; a finger's while it touches, and a moment after.
    pointer.target = still ? 0 : pointer.down || (fine && now - pointer.lastMove < 2500) ? 1 : 0;
    gl.uniform2f(u.uRes, canvas.width, canvas.height);
    gl.uniform1f(u.uTime, clock % 3600);
    gl.uniform1f(u.uScroll, still ? 0 : window.scrollY / window.innerHeight);
    gl.uniform3f(u.uLens, lx, ly, lr);
    gl.uniform3f(u.uPtr, pointer.x, pointer.y, pointer.mass);
    const rip = new Float32Array(16);
    rings.forEach((r, i) => {
      const age = clock - r.born;
      if (age < 3) rip.set([r.x, r.y, age, 1], i * 4);
    });
    gl.uniform4fv(u.uRip, rip);
    gl.uniform1f(u.uGlow, Math.max(0, 1 - window.scrollY / (window.innerHeight * 1.1)));
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }

  function frame(now) {
    if (!running) return;
    requestAnimationFrame(frame);
    const dt = Math.min(64, now - last);
    // A phone nobody is touching draws at half the rate.
    if (coarse && now - lastInput > 1500 && skipped++ % 2 === 0) return;
    last = now;
    clock += dt / 1000;
    const k = 1 - Math.pow(0.82, dt / 16.67);
    pointer.x += (pointer.tx - pointer.x) * k;
    pointer.y += (pointer.ty - pointer.y) * k;
    pointer.mass += (pointer.target - pointer.mass) * (1 - Math.pow(0.9, dt / 16.67));
    draw();

    // Frames that come late lower the resolution; a run of quick ones raises it again.
    frames += 1;
    spent += dt;
    if (frames === 45) {
      const average = spent / frames;
      const interval = coarse && now - lastInput > 1500 ? 34 : 17.5;
      if (average > interval * 1.3 && quality > QUALITY_MIN) {
        quality = Math.max(QUALITY_MIN, quality * 0.85);
        size();
      } else if (average < interval * 1.05 && quality < QUALITY_MAX) {
        quality = Math.min(QUALITY_MAX, quality * 1.06);
        size();
      }
      frames = 0;
      spent = 0;
    }
  }

  function start() {
    if (!init()) {
      root.dataset.aether = 'none';
      canvas.remove();
      return;
    }
    root.dataset.aether = 'webgl';
    size();
    if (still) {
      draw();
    } else if (running) {
      last = performance.now();
      requestAnimationFrame(frame);
    }
  }

  canvas.addEventListener('webglcontextlost', (e) => {
    e.preventDefault();
    gl = null;
  });
  canvas.addEventListener('webglcontextrestored', start);

  let resizing = 0;
  window.addEventListener('resize', () => {
    window.clearTimeout(resizing);
    resizing = window.setTimeout(() => {
      size();
      if (still) draw();
    }, 150);
  });

  document.addEventListener('visibilitychange', () => {
    const was = running;
    running = !document.hidden;
    if (running && !was && !still && gl) {
      last = performance.now();
      requestAnimationFrame(frame);
    }
  });

  start();
})();
