/*
  qorsync.dev: the living background and the page's responses to the pointer.

  The aether is drawn on one canvas behind the page: a hexagonal lattice (the "flower of life" grid) that lights up
  around the pointer, an astrolabe of rings, ticks, and an inscribed triangle and square turning slowly, and embers rising. It is
  decoration only: every word on the page is HTML. With reduced motion asked for, one still frame is drawn and nothing
  moves; while the tab is hidden, nothing is drawn.
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
  const GILT = [201, 164, 92];
  const rgba = (c, a) => `rgba(${c[0]},${c[1]},${c[2]},${a})`;

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

  /* The astrolabe: rings at different speeds, a ring of ticks, twelve stations, and a hexagram. */
  function astrolabe(t, scroll) {
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

    // Twelve stations on the second ring, turning the other way, one of them burning.
    ctx.save();
    ctx.rotate(t * rings[1].spin);
    for (let i = 0; i < 12; i++) {
      const a = (i / 12) * Math.PI * 2;
      const x = Math.cos(a) * rings[1].r;
      const y = Math.sin(a) * rings[1].r;
      ctx.fillStyle = i === 0 ? rgba(EMBER, 0.9) : rgba(GILT, 0.32 * lift);
      ctx.beginPath();
      ctx.arc(x, y, i === 0 ? 3 : 1.8, 0, Math.PI * 2);
      ctx.fill();
      if (i === 0) {
        const g = ctx.createRadialGradient(x, y, 0, x, y, 26);
        g.addColorStop(0, rgba(EMBER, 0.35));
        g.addColorStop(1, rgba(EMBER, 0));
        ctx.fillStyle = g;
        ctx.fillRect(x - 26, y - 26, 52, 52);
      }
    }
    ctx.restore();

    // A triangle and a square inscribed in the third ring, turning against each other: the circle squared.
    ctx.strokeStyle = rgba(GILT, 0.12 * lift);
    for (const [sides, spin] of [[3, rings[2].spin], [4, -rings[2].spin * 0.6]]) {
      ctx.save();
      ctx.rotate(t * spin);
      ctx.beginPath();
      for (let k = 0; k <= sides; k++) {
        const a = -Math.PI / 2 + (k / sides) * Math.PI * 2;
        const x = Math.cos(a) * rings[2].r;
        const y = Math.sin(a) * rings[2].r;
        if (k === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();
      ctx.restore();
    }

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

    astrolabe(t, window.scrollY);

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
