/**
 * QQ, the QOR Engine, as a launcher surface (ADR-081, ADR-082, DIRECTION P3.1).
 *
 * A viewport on the left, the project, the scene's entities and the selected
 * entity's components on the right. The editor holds the scene; the engine
 * draws it and reports clicks and drags through the `EditorBridge`. Play runs a
 * copy of the scene and Stop drops it, so nothing a game does while it plays
 * reaches what is saved.
 *
 * A scene is saved as `scenes/<name>.qq.json` in a Qontrol project, through the
 * host (the webview has no filesystem), in QQ's canonical form, so the change is
 * an ordinary, readable diff in Projects and is committed there. Nothing here is
 * published, minted or signed.
 */

import { useEffect, useMemo, useRef, useState } from 'react';
import { Circle, FolderOpen, Pause, Play, Plus, Save, Sparkles, Square, StepForward, Trash2 } from 'lucide-react';

import { explain, qontrol, qq, type QontrolProject } from '../../lib/ipc';
import { wantsLessMotion } from '../../lib/a11y';
import { useQor } from '../../state/store';
import { Panel, ViewHeader } from '../../views/parts';
import { Engine } from '../runtime/engine';
import {
  SceneError,
  keep,
  newEntity,
  parseScene,
  serialiseScene,
  starterScene,
  type Entity,
  type Scene,
} from '../runtime/scene';
import { Inspector } from './Inspector';

const PROJECT_KEY = 'qq.project';
const NAME = /^[a-z0-9][a-z0-9-]{0,62}$/;

function rememberedProject(): string | null {
  try {
    return localStorage.getItem(PROJECT_KEY);
  } catch {
    return null;
  }
}

function remember(path: string | null) {
  try {
    if (path) localStorage.setItem(PROJECT_KEY, path);
    else localStorage.removeItem(PROJECT_KEY);
  } catch {
    /* A remembered folder is a convenience, not worth failing over. */
  }
}

export function QQView() {
  const notify = useQor((s) => s.notify);
  const theme = useQor((s) => s.theme);

  const [scene, setScene] = useState<Scene>(() => starterScene());
  const [selected, setSelected] = useState<string | null>(null);
  const [playing, setPlaying] = useState(false);
  const [held, setHeld] = useState(false);
  const [stats, setStats] = useState({ particles: 0, fps: 0 });
  const [drawing, setDrawing] = useState(true);

  const [project, setProject] = useState<QontrolProject | null>(null);
  const [scenes, setScenes] = useState<string[]>([]);
  const [name, setName] = useState('first-light');
  // The canonical text last saved or opened. The scene is unsaved while it differs.
  const [saved, setSaved] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [discarding, setDiscarding] = useState(false);

  const canvas = useRef<HTMLCanvasElement | null>(null);
  const engine = useRef<Engine | null>(null);
  const sceneRef = useRef(scene);
  sceneRef.current = scene;

  const text = useMemo(() => serialiseScene({ ...scene, name }), [scene, name]);
  const unsaved = text !== saved;

  // ── the engine ──────────────────────────────────────────────────────────

  useEffect(() => {
    if (!canvas.current) return;
    const e = new Engine(canvas.current, sceneRef.current, {
      select: (id) => setSelected(id),
      move: (id, x, y) =>
        setScene((s) => ({
          ...s,
          entities: s.entities.map((n) =>
            n.id === id ? { ...n, transform: { ...n.transform, x: keep('x', x), y: keep('y', y) } } : n,
          ),
        })),
      stats: (particles, fps) => setStats({ particles, fps }),
    });
    engine.current = e;
    setDrawing(e.drawing);
    return () => {
      e.dispose();
      engine.current = null;
    };
  }, []);

  useEffect(() => engine.current?.setScene(scene), [scene]);
  useEffect(() => engine.current?.setSelected(selected), [selected]);
  // Themes are CSS variables; read them after the new ones are applied.
  useEffect(() => {
    const frame = requestAnimationFrame(() => engine.current?.readTheme());
    return () => cancelAnimationFrame(frame);
  }, [theme]);

  const play = () => {
    // Asked at the press, not at the last render: the system setting can change while QQ is open.
    const lessMotion = wantsLessMotion(useQor.getState().a11y);
    engine.current?.play(lessMotion);
    setHeld(lessMotion);
    setPlaying(true);
    setStats({ particles: 0, fps: 0 });
  };
  const stop = () => {
    engine.current?.stop();
    setPlaying(false);
    setHeld(false);
  };
  // Leaving the surface mid-play stops the loop with the engine (the effect above).

  // ── the scene ───────────────────────────────────────────────────────────

  const update = (id: string, change: (e: Entity) => Entity) =>
    setScene((s) => ({ ...s, entities: s.entities.map((e) => (e.id === id ? change(e) : e)) }));

  const add = (kind: 'rect' | 'circle' | 'emitter') => {
    const e = newEntity(sceneRef.current, kind);
    setScene((s) => ({ ...s, entities: [...s.entities, e] }));
    setSelected(e.id);
  };

  const remove = (id: string) => {
    setScene((s) => ({ ...s, entities: s.entities.filter((e) => e.id !== id) }));
    setSelected(null);
  };

  const chosen = scene.entities.find((e) => e.id === selected) ?? null;

  // ── the project ─────────────────────────────────────────────────────────

  const listScenes = async (p: QontrolProject) => {
    try {
      setScenes(await qq.list(p.path));
    } catch (e) {
      setScenes([]);
      notify('bad', explain(e));
    }
  };

  // The folder used last time, if it still opens. Silently nothing if not.
  useEffect(() => {
    const path = rememberedProject();
    if (!path) return;
    qontrol
      .read(path)
      .then((p) => {
        setProject(p);
        return qq.list(p.path).then(setScenes);
      })
      .catch(() => remember(null));
  }, []);

  const chooseProject = async (): Promise<QontrolProject | null> => {
    const picked = await qontrol.pickFolder();
    if (!picked) return null;
    const p = await qontrol.open(picked);
    setProject(p);
    remember(p.path);
    await listScenes(p);
    return p;
  };

  const withBusy = async (what: () => Promise<void>) => {
    setBusy(true);
    try {
      await what();
    } catch (e) {
      notify('bad', e instanceof SceneError ? e.message : explain(e));
    } finally {
      setBusy(false);
    }
  };

  const save = () =>
    withBusy(async () => {
      if (!NAME.test(name)) {
        notify('bad', 'A scene name is lowercase letters, digits and dashes, starting with a letter or digit.');
        return;
      }
      const p = project ?? (await chooseProject());
      if (!p) return;
      const next = await qq.save(p.path, name, text);
      setProject(next);
      setSaved(text);
      setScene((s) => ({ ...s, name }));
      await listScenes(next);
      notify('ok', `Saved scenes/${name}.qq.json. Commit it in Projects.`);
    });

  const open = (which: string) =>
    withBusy(async () => {
      if (!project) return;
      const source = await qq.load(project.path, which);
      const loaded = parseScene(source);
      if (playing) stop();
      setScene(loaded);
      setName(which);
      setSelected(null);
      setSaved(serialiseScene({ ...loaded, name: which }));
      setDiscarding(false);
    });

  const fresh = () => {
    if (unsaved && saved !== null && !discarding) {
      setDiscarding(true);
      return;
    }
    if (playing) stop();
    setScene(starterScene());
    setName('first-light');
    setSaved(null);
    setSelected(null);
    setDiscarding(false);
  };

  // ── drawing ─────────────────────────────────────────────────────────────

  return (
    <div className="flex h-full flex-col" data-qq-view>
      <ViewHeader
        eyebrow="Create"
        title="QQ"
        body="The QOR Engine. Build a small game on this canvas, play it here, and save it into a project, where Projects versions it like any other file. Nothing is published or signed from here yet."
        action={
          <>
            {playing && held && (
              <button type="button" className="btn btn-ghost" onClick={() => engine.current?.step()} title="Advance a quarter of a second">
                <StepForward size={13} aria-hidden="true" />
                Step
              </button>
            )}
            {playing ? (
              <button type="button" className="btn btn-primary" onClick={stop}>
                <Square size={13} aria-hidden="true" />
                Stop
              </button>
            ) : (
              <button type="button" className="btn btn-primary" onClick={play}>
                <Play size={13} aria-hidden="true" />
                Play
              </button>
            )}
          </>
        }
      />

      <div className="flex min-h-0 flex-1">
        <section className="flex min-w-0 flex-1 flex-col gap-2 p-4" aria-label="Viewport">
          <div className="relative min-h-0 flex-1 overflow-hidden rounded-qor border border-edge bg-void">
            <canvas
              ref={canvas}
              className={`block h-full w-full ${playing ? 'cursor-crosshair' : 'cursor-default'}`}
              role="img"
              aria-label={`QQ viewport: ${scene.entities.length} entities${playing ? ', playing' : ''}`}
              data-qq-canvas
            />
            {!drawing && (
              <p className="absolute inset-0 flex items-center justify-center p-6 text-center text-ui text-ink-muted">
                This webview cannot draw WebGL2, so QQ cannot show the scene. You can still edit and save it.
              </p>
            )}
          </div>
          <p className="numeric flex flex-wrap gap-x-4 text-caption text-ink-muted" data-qq-status>
            <span>{scene.entities.length} entities</span>
            <span>
              {scene.size.w} × {scene.size.h}
            </span>
            {playing && <span data-qq-particles={stats.particles}>{stats.particles} particles</span>}
            {playing && !held && <span>{stats.fps} fps</span>}
            {playing && held && (
              <span className="text-warn" role="status">
                <Pause size={11} className="mr-1 inline" aria-hidden="true" />
                Held still: reduced motion is on. Step advances it.
              </span>
            )}
            {!playing && <span>Drag to move. Play to run it.</span>}
          </p>
        </section>

        <aside className="flex w-80 flex-none flex-col gap-3 overflow-y-auto border-l border-edge p-4" aria-label="Inspector">
          <Panel className="flex flex-col gap-3 p-4">
            <p className="eyebrow">Project</p>
            <p className="truncate text-ui text-ink" title={project?.path}>
              {project ? project.name : 'No project chosen'}
            </p>
            <label className="block">
              <span className="eyebrow mb-1.5 block">Scene name</span>
              <input
                className="field"
                value={name}
                onChange={(e) => setName(e.target.value.toLowerCase())}
                spellCheck={false}
                aria-describedby="qq-name-hint"
              />
              <span id="qq-name-hint" className="mt-1 block text-caption text-ink-faint">
                Saved as scenes/{name || '…'}.qq.json
              </span>
            </label>
            <div className="flex flex-wrap gap-2">
              <button type="button" className="btn btn-primary" onClick={save} disabled={busy || playing}>
                <Save size={13} aria-hidden="true" />
                Save{unsaved ? '' : 'd'}
              </button>
              <button type="button" className="btn btn-ghost" onClick={() => void withBusy(async () => void (await chooseProject()))} disabled={busy}>
                <FolderOpen size={13} aria-hidden="true" />
                {project ? 'Change' : 'Choose'}
              </button>
              <button type="button" className="btn btn-ghost" onClick={fresh} disabled={busy}>
                {discarding ? 'Discard changes?' : 'New'}
              </button>
            </div>
            {unsaved && saved !== null && <p className="text-caption text-warn">Unsaved changes.</p>}
            {project && scenes.length > 0 && (
              <div>
                <p className="eyebrow mb-1.5">Scenes in this project</p>
                <ul className="flex flex-col gap-0.5" aria-label="Scenes in this project">
                  {scenes.map((s) => (
                    <li key={s}>
                      <button
                        type="button"
                        className="w-full truncate rounded-qor px-2 py-1 text-left text-ui text-ink-body hover:bg-raised"
                        onClick={() => open(s)}
                        disabled={busy}
                      >
                        {s}
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </Panel>

          <Panel className="flex flex-col gap-2 p-4">
            <p className="eyebrow">Entities</p>
            <ul className="flex flex-col gap-0.5" aria-label="Entities">
              {scene.entities.map((e) => (
                <li key={e.id}>
                  <button
                    type="button"
                    aria-pressed={e.id === selected}
                    onClick={() => setSelected(e.id === selected ? null : e.id)}
                    className={`flex w-full items-center gap-2 rounded-qor px-2 py-1 text-left text-ui ${
                      e.id === selected ? 'bg-raised text-ink' : 'text-ink-body hover:bg-raised'
                    }`}
                  >
                    <span
                      aria-hidden="true"
                      className={`h-2.5 w-2.5 flex-none ${e.shape?.kind === 'rect' ? 'rounded-[1px]' : 'rounded-full'}`}
                      style={{ background: e.shape?.colour ?? e.emitter?.colour ?? 'transparent', border: e.shape ? undefined : '1px solid currentColor' }}
                    />
                    <span className="min-w-0 flex-1 truncate">{e.name || e.id}</span>
                    <span className="numeric text-caption text-ink-faint">{e.id}</span>
                  </button>
                </li>
              ))}
            </ul>
            <div className="flex flex-wrap gap-2" role="group" aria-label="Add an entity">
              <button type="button" className="btn btn-ghost" onClick={() => add('circle')} disabled={playing}>
                <Circle size={13} aria-hidden="true" />
                Orb
              </button>
              <button type="button" className="btn btn-ghost" onClick={() => add('rect')} disabled={playing}>
                <Plus size={13} aria-hidden="true" />
                Block
              </button>
              <button type="button" className="btn btn-ghost" onClick={() => add('emitter')} disabled={playing}>
                <Sparkles size={13} aria-hidden="true" />
                Emitter
              </button>
            </div>
          </Panel>

          {chosen && (
            <Panel className="flex flex-col gap-3 p-4">
              <div className="flex items-center gap-2">
                <p className="eyebrow flex-1">Selected</p>
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={() => remove(chosen.id)}
                  disabled={playing}
                  aria-label={`Delete ${chosen.name || chosen.id}`}
                >
                  <Trash2 size={13} aria-hidden="true" />
                  Delete
                </button>
              </div>
              <Inspector entity={chosen} disabled={playing} onChange={(change) => update(chosen.id, change)} />
            </Panel>
          )}
        </aside>
      </div>
    </div>
  );
}
