/*
  qorsync.dev: the living background and the page's responses to the pointer.

  The aether is drawn on one canvas behind the page (ADR-123): an aurora of ember and gilt ribbons across the top, QOR OS
  Lumen's signature; a hexagonal lattice (the "flower of life" grid) that lights up around the pointer; the lens, QOR
  OS's mark at the scale of the sky, its rings and ticks turning slowly and the Q's tail burning on its ring; and embers
  rising. It is decoration only: every word on the page is HTML. With reduced motion asked for, one still frame is drawn
  and nothing moves; while the tab is hidden, nothing is drawn.
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
  const ctx = canvas && canvas.getContext('2d');
  if (!ctx) return;

  const EMBER = [255, 106, 0];
  const EMBER_DIM = [180, 74, 0];
  const GILT = [201, 164, 92];
  const rgba = (c, a) => `rgba(${c[0]},${c[1]},${c[2]},${a})`;

  // The aurora is drawn at an eighth of the size and scaled up, which softens its ribbons for almost nothing.
  const AURORA_SCALE = 8;
  const aurora = document.createElement('canvas');
  const actx = aurora.getContext('2d');
  const RIBBONS = [
    { c: EMBER, a: 0.11, y: 0.17, amp: 0.07, f: 1.3, s: 0.05, p: 0.0, thick: 0.13 },
    { c: GILT, a: 0.08, y: 0.27, amp: 0.06, f: 0.9, s: -0.035, p: 2.1, thick: 0.11 },
    { c: EMBER_DIM, a: 0.09, y: 0.09, amp: 0.05, f: 1.8, s: 0.03, p: 4.2, thick: 0.08 },
  ];

  let w = 0;
  let h = 0;
  let dpr = 1;
  let nodes = [];
  let edges = [];
  let embers = [];
  let lattice = null; // the lattice at rest, drawn once per size
  const pointer = { x: -9999, y: -9999, tx: -9999, ty: -9999, seen: false };

  const SPACING = 74;
  const REACH = 220; // how far the pointer's light reaches

  function size() {
    dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    w = window.innerWidth;
    h = window.innerHeight;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    // A triangular lattice: every node has six neighbours, as in the flower of life.
    nodes = [];
    edges = [];
    const rowH = SPACING * Math.sin(Math.PI / 3);
    const cols = Math.ceil(w / SPACING) + 2;
    const rows = Math.ceil(h / rowH) + 2;
    const index = (r, c) => r * cols + c;
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        nodes.push({ x: (c - 0.5) * SPACING + (r % 2 ? SPACING / 2 : 0), y: (r - 0.5) * rowH, glow: 0 });
      }
    }
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const i = index(r, c);
        if (c + 1 < cols) edges.push([i, index(r, c + 1)]);
        if (r + 1 < rows) {
          const shift = r % 2 ? 0 : -1;
          if (c + shift >= 0) edges.push([i, index(r + 1, c + shift)]);
          if (c + shift + 1 < cols) edges.push([i, index(r + 1, c + shift + 1)]);
        }
      }
    }

    lattice = document.createElement('canvas');
    lattice.width = canvas.width;
    lattice.height = canvas.height;
    const l = lattice.getContext('2d');
    l.setTransform(dpr, 0, 0, dpr, 0, 0);
    l.strokeStyle = rgba(GILT, 0.045);
    l.lineWidth = 1;
    l.beginPath();
    for (const [a, b] of edges) {
      l.moveTo(nodes[a].x, nodes[a].y);
      l.lineTo(nodes[b].x, nodes[b].y);
    }
    l.stroke();
    l.fillStyle = rgba(GILT, 0.12);
    for (const n of nodes) l.fillRect(n.x - 0.75, n.y - 0.75, 1.5, 1.5);

    const count = Math.round(Math.min(90, (w * h) / 18000));
    embers = Array.from({ length: count }, () => ember(true));

    aurora.width = Math.max(1, Math.ceil(w / AURORA_SCALE));
    aurora.height = Math.max(1, Math.ceil((h * 0.6) / AURORA_SCALE));
  }

  /* The aurora: slow ribbons whose edges sway on two waves each, drawn small and laid over the top of the page, fading
     as the page scrolls past the first screen. */
  function sky(t, scroll) {
    const fade = Math.max(0, 1 - scroll / (h * 1.2));
    if (fade <= 0) return;
    const aw = aurora.width;
    const ah = aurora.height;
    const H = ah / 0.6; // the viewport's height at the aurora's scale
    actx.clearRect(0, 0, aw, ah);
    for (const r of RIBBONS) {
      const base = H * r.y;
      const amp = H * r.amp;
      const thick = H * r.thick;
      const edge = (u, side) =>
        base +
        Math.sin(u * Math.PI * 2 * r.f + t * r.s * 6 + r.p) * amp +
        Math.sin(u * 7.3 - t * r.s * 4 + r.p * 1.7) * amp * 0.35 +
        side * thick * (0.5 + 0.18 * Math.sin(u * 5.1 + t * r.s * 3 + r.p));
      actx.beginPath();
      for (let i = 0; i <= 48; i++) actx.lineTo((i / 48) * aw, edge(i / 48, -1));
      for (let i = 48; i >= 0; i--) actx.lineTo((i / 48) * aw, edge(i / 48, 1));
      actx.closePath();
      const g = actx.createLinearGradient(0, base - amp - thick, 0, base + amp + thick);
      g.addColorStop(0, rgba(r.c, 0));
      g.addColorStop(0.5, rgba(r.c, r.a));
      g.addColorStop(1, rgba(r.c, 0));
      actx.fillStyle = g;
      actx.fill();
    }
    ctx.save();
    ctx.globalCompositeOperation = 'lighter';
    ctx.globalAlpha = fade;
    ctx.imageSmoothingEnabled = true;
    ctx.drawImage(aurora, 0, -scroll * 0.2, w, h * 0.6);
    ctx.restore();
  }

  function ember(anywhere) {
    return {
      x: Math.random() * w,
      y: anywhere ? Math.random() * h : h + 10,
      r: 0.6 + Math.random() * 1.8,
      vy: 0.12 + Math.random() * 0.42,
      vx: (Math.random() - 0.5) * 0.18,
      life: Math.random() * Math.PI * 2,
      hue: Math.random() < 0.7 ? EMBER : GILT,
    };
  }

  /* The lens: QOR OS's mark at the scale of the sky. Rings at different speeds, a ring of ticks, twelve stations, the
     broad ring of the Q, and its tail burning on that ring, swaying about where the mark puts it. */
  function lens(t, scroll) {
    const big = w > 760;
    const cx = big ? w * 0.74 : w * 0.5;
    const cy = (big ? h * 0.42 : h * 0.3) - scroll * 0.12;
    const R = Math.min(w, h) * (big ? 0.42 : 0.46);
    const near = Math.hypot(pointer.x - cx, pointer.y - cy) < R * 1.1;
    const lift = near ? 1.35 : 1;

    ctx.save();
    ctx.translate(cx, cy);
    ctx.lineWidth = 1;

    const rings = [
      { r: R, a: 0.10, spin: 0.012 },
      { r: R * 0.86, a: 0.07, spin: -0.02 },
      { r: R * 0.62, a: 0.09, spin: 0.03 },
      { r: R * 0.34, a: 0.11, spin: -0.045 },
    ];
    for (const ring of rings) {
      ctx.strokeStyle = rgba(GILT, ring.a * lift);
      ctx.beginPath();
      ctx.arc(0, 0, ring.r, 0, Math.PI * 2);
      ctx.stroke();
    }

    // 72 ticks, every sixth long, turning with the outer ring.
    ctx.save();
    ctx.rotate(t * rings[0].spin);
    ctx.strokeStyle = rgba(GILT, 0.16 * lift);
    ctx.beginPath();
    for (let i = 0; i < 72; i++) {
      const a = (i / 72) * Math.PI * 2;
      const len = i % 6 === 0 ? 16 : 6;
      ctx.moveTo(Math.cos(a) * R, Math.sin(a) * R);
      ctx.lineTo(Math.cos(a) * (R - len), Math.sin(a) * (R - len));
    }
    ctx.stroke();
    ctx.restore();

    // Twelve stations on the second ring, turning the other way.
    ctx.save();
    ctx.rotate(t * rings[1].spin);
    ctx.fillStyle = rgba(GILT, 0.32 * lift);
    for (let i = 0; i < 12; i++) {
      const a = (i / 12) * Math.PI * 2;
      ctx.beginPath();
      ctx.arc(Math.cos(a) * rings[1].r, Math.sin(a) * rings[1].r, 1.8, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();

    // The broad ring of the Q on the third ring, with hairline edges, as the mark engraves it.
    const q = rings[2].r;
    const band = Math.max(6, R * 0.07);
    ctx.lineWidth = band;
    ctx.strokeStyle = rgba(GILT, 0.05 * lift);
    ctx.beginPath();
    ctx.arc(0, 0, q, 0, Math.PI * 2);
    ctx.stroke();
    ctx.lineWidth = 1;
    ctx.strokeStyle = rgba(GILT, 0.14 * lift);
    for (const edge of [q - band / 2, q + band / 2]) {
      ctx.beginPath();
      ctx.arc(0, 0, edge, 0, Math.PI * 2);
      ctx.stroke();
    }

    // Its tail, an ember just outside the ring at the lower right, swaying a little either side of where the mark
    // puts it, and breathing.
    const a = Math.PI / 4 + Math.sin(t * 0.25) * 0.3;
    const d = q + band * 0.9;
    const x = Math.cos(a) * d;
    const y = Math.sin(a) * d;
    const core = Math.max(4, R * 0.03) * (1 + Math.sin(t * 1.3) * 0.08);
    const glow = core * 7;
    const g = ctx.createRadialGradient(x, y, 0, x, y, glow);
    g.addColorStop(0, rgba(EMBER, 0.4));
    g.addColorStop(1, rgba(EMBER, 0));
    ctx.fillStyle = g;
    ctx.fillRect(x - glow, y - glow, glow * 2, glow * 2);
    ctx.fillStyle = rgba(EMBER, 0.9);
    ctx.beginPath();
    ctx.arc(x, y, core, 0, Math.PI * 2);
    ctx.fill();

    // Arcs on the inner ring, like the rete of an astrolabe.
    ctx.save();
    ctx.rotate(t * rings[3].spin);
    ctx.strokeStyle = rgba(EMBER, 0.22 * lift);
    ctx.lineWidth = 1.5;
    for (let i = 0; i < 3; i++) {
      ctx.beginPath();
      ctx.arc(0, 0, rings[3].r, (i / 3) * Math.PI * 2, (i / 3) * Math.PI * 2 + 0.7);
      ctx.stroke();
    }
    ctx.restore();

    ctx.restore();
  }

  let last = performance.now();
  let t = 0;
  let running = true;

  function frame(now) {
    const dt = Math.min(48, now - last) / 16.67;
    last = now;
    t += dt / 60;

    // The pointer's light eases toward it rather than jumping.
    pointer.x += (pointer.tx - pointer.x) * 0.12 * dt;
    pointer.y += (pointer.ty - pointer.y) * 0.12 * dt;

    ctx.clearRect(0, 0, w, h);
    sky(t, window.scrollY);
    ctx.drawImage(lattice, 0, 0, w, h);

    if (pointer.seen) {
      // The lattice lights up near the pointer.
      ctx.lineWidth = 1;
      for (const [a, b] of edges) {
        const A = nodes[a];
        const B = nodes[b];
        const mx = (A.x + B.x) / 2;
        const my = (A.y + B.y) / 2;
        const d = Math.hypot(mx - pointer.x, my - pointer.y);
        if (d < REACH) {
          const k = 1 - d / REACH;
          ctx.strokeStyle = rgba(k > 0.6 ? EMBER : GILT, 0.05 + k * k * 0.45);
          ctx.beginPath();
          ctx.moveTo(A.x, A.y);
          ctx.lineTo(B.x, B.y);
          ctx.stroke();
        }
      }
      for (const n of nodes) {
        const d = Math.hypot(n.x - pointer.x, n.y - pointer.y);
        if (d < REACH) {
          const k = 1 - d / REACH;
          ctx.fillStyle = rgba(k > 0.55 ? EMBER : GILT, 0.15 + k * 0.7);
          ctx.beginPath();
          ctx.arc(n.x, n.y, 1 + k * 2.2, 0, Math.PI * 2);
          ctx.fill();
        }
      }
      const g = ctx.createRadialGradient(pointer.x, pointer.y, 0, pointer.x, pointer.y, REACH * 1.6);
      g.addColorStop(0, rgba(EMBER, 0.08));
      g.addColorStop(1, rgba(EMBER, 0));
      ctx.fillStyle = g;
      ctx.fillRect(pointer.x - REACH * 1.6, pointer.y - REACH * 1.6, REACH * 3.2, REACH * 3.2);
    }

    lens(t, window.scrollY);

    // Embers rise, flicker, and lean toward the pointer.
    for (let i = 0; i < embers.length; i++) {
      const e = embers[i];
      e.life += 0.04 * dt;
      e.y -= e.vy * dt;
      e.x += (e.vx + Math.sin(e.life) * 0.12) * dt;
      if (pointer.seen) {
        const dx = pointer.x - e.x;
        const dy = pointer.y - e.y;
        const d = Math.hypot(dx, dy);
        if (d < REACH * 1.4 && d > 1) {
          e.x += (dx / d) * 0.22 * dt;
          e.y += (dy / d) * 0.22 * dt;
        }
      }
      if (e.y < -12 || e.x < -12 || e.x > w + 12) embers[i] = ember(false);
      const a = 0.35 + Math.sin(e.life * 2.3) * 0.25;
      ctx.fillStyle = rgba(e.hue, a);
      ctx.beginPath();
      ctx.arc(e.x, e.y, e.r, 0, Math.PI * 2);
      ctx.fill();
    }

    if (running && !still) requestAnimationFrame(frame);
  }

  window.addEventListener('pointermove', (e) => {
    pointer.tx = e.clientX;
    pointer.ty = e.clientY;
    if (!pointer.seen) {
      pointer.x = e.clientX;
      pointer.y = e.clientY;
      pointer.seen = true;
    }
  }, { passive: true });
  document.addEventListener('pointerleave', () => { pointer.seen = false; });

  let resizing = 0;
  window.addEventListener('resize', () => {
    window.clearTimeout(resizing);
    resizing = window.setTimeout(() => {
      size();
      if (still) frame(performance.now());
    }, 120);
  });

  document.addEventListener('visibilitychange', () => {
    const was = running;
    running = !document.hidden;
    if (running && !was && !still) {
      last = performance.now();
      requestAnimationFrame(frame);
    }
  });

  size();
  if (still) frame(performance.now());
  else requestAnimationFrame(frame);
})();
