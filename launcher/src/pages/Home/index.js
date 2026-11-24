import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  FaPlayCircle,
  FaCog,
  FaFolderOpen,
  FaGithub,
  FaHeart,
  FaCoffee,
  FaTools,
  FaStop,
  FaTerminal,
} from 'react-icons/fa';
import { FiX, FiMinus } from 'react-icons/fi';

import { Window } from './styles.js';
import logo from '../../assets/logo.svg';

const bridge = window.scarlett;
const REPO = 'https://github.com/ZigZagBoi1/ScarlettPort';
const PAYPAL = 'https://www.paypal.me/SalwanStudios';
const KOFI = 'https://ko-fi.com/ziggymar';
const PLAYABLE_PERCENT = 20;
const MAX_LOG = 600;

const SHADER_RE = /(\d+)\s*\/\s*(\d+)\s*\((\d+)\s*s\)/;
const PIPE_RE = /pipelines:\s*(\d+)\s*made/i;
const FPS_RE = /(\d+(?:\.\d+)?)\s*fps/i;

function folderName(p) {
  if (!p) return 'No game folder';
  const parts = p.replace(/[\\/]+$/, '').split(/[\\/]/);
  return parts[parts.length - 1] || p;
}

export default function Home() {
  const [state, setState] = useState({
    gameDir: '',
    repoRoot: '',
    built: false,
    options: { runtime: 'xcl', seriesS: false, autoMenu: false },
  });
  const [running, setRunning] = useState(null);
  const [status, setStatus] = useState('');
  const [lines, setLines] = useState([]);
  const [shader, setShader] = useState(null); // {done,total,secs}
  const [pipelines, setPipelines] = useState(null);
  const [fps, setFps] = useState(null);
  const [showOptions, setShowOptions] = useState(false);
  const logRef = useRef(null);

  const refresh = useCallback(() => {
    if (bridge) bridge.getState().then((s) => { setState(s); setRunning(s.running); });
  }, []);

  useEffect(() => {
    if (!bridge) {
      setStatus('Not running inside Electron: launcher actions are unavailable.');
      return undefined;
    }
    refresh();
    const offLog = bridge.onLog(({ text, stream }) => {
      const s = SHADER_RE.exec(text);
      if (s) setShader({ done: +s[1], total: +s[2], secs: +s[3] });
      const p = PIPE_RE.exec(text);
      if (p) setPipelines(+p[1]);
      if (/frames presented|fps/i.test(text)) {
        const f = FPS_RE.exec(text);
        if (f) setFps(parseFloat(f[1]));
      }
      setLines((prev) => {
        const next = prev.concat({ text, stream });
        return next.length > MAX_LOG ? next.slice(next.length - MAX_LOG) : next;
      });
    });
    const offRun = bridge.onRunState((r) => {
      setRunning(r.running);
      if (!r.running) {
        setFps(null);
        refresh();
        setStatus(r.code === 0 ? 'Finished.' : r.code != null ? `Stopped (exit ${r.code}).` : '');
      }
    });
    return () => { offLog(); offRun(); };
  }, [refresh]);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [lines]);

  const opts = state.options;
  function updateOptions(patch) {
    const next = Object.assign({}, opts, patch);
    setState((s) => Object.assign({}, s, { options: next }));
    if (bridge) bridge.setOptions(next);
  }

  async function pickGame() {
    const r = await bridge.pickGame();
    if (r.ok) { setState(r.state); setStatus(''); }
    else if (r.error) setStatus(r.error);
  }
  async function pickRepo() {
    const r = await bridge.pickRepo();
    if (r.ok) { setState(r.state); setStatus(''); }
    else if (r.error) setStatus(r.error);
  }
  async function handlePlay() {
    if (!bridge || running) return;
    if (!state.gameDir) { pickGame(); return; }
    setFps(null);
    const r = await bridge.play(opts);
    setStatus(r.ok ? 'Launching…' : r.error);
  }
  async function handleSetup() {
    if (!bridge || running) return;
    setShader(null);
    setPipelines(null);
    const r = await bridge.setup();
    setStatus(r.ok ? 'Running setup…' : r.error);
  }
  const open = (url) => bridge && bridge.openExternal(url);

  const pct = shader && shader.total ? Math.min(100, (shader.done / shader.total) * 100) : 0;
  const busy = !!running;

  return (
    <Window>
      <div className="background">
        <div className="sun" />
        <div className="grid" />
      </div>

      <div className="drag" />
      <div className="window-options">
        <div className="option" onClick={() => bridge && bridge.minimize()}>
          <FiMinus size={20} color="#fff" />
        </div>
        <div className="option" onClick={() => open(REPO)} title="Project page">
          <span>?</span>
        </div>
        <div className="option" onClick={() => bridge && bridge.close()}>
          <FiX size={20} color="#fff" />
        </div>
      </div>

      <header>
        <div className="user">
          <h3>{folderName(state.gameDir)}</h3>
          <span>{state.gameDir ? 'READY' : 'NOT SET'}</span>
          {fps != null && <span className="level">{fps.toFixed(1)} FPS</span>}
        </div>
        <span className="anchor" onClick={pickGame}>
          <FaFolderOpen size={9} /> {state.gameDir ? 'CHANGE GAME FOLDER' : 'SELECT GAME FOLDER'}
        </span>
        {!state.repoRoot && !state.bundled && (
          <span className="anchor warn" onClick={pickRepo}>LOCATE REPO</span>
        )}
      </header>

      <div className="side-panel">
        <h4><FaTerminal size={12} /> STATUS</h4>
        <div className="progress-label">
          <span>Progress to playable</span>
          <b>{PLAYABLE_PERCENT}%</b>
        </div>
        <div className="bar"><div style={{ width: `${PLAYABLE_PERCENT}%` }} /></div>

        <div className="progress-label">
          <span>Shader precompile</span>
          <b>{shader ? `${shader.done}/${shader.total}` : '-'}</b>
        </div>
        <div className="bar"><div style={{ width: `${pct}%` }} /></div>
        <div className="mini">
          {shader ? `${shader.secs} s elapsed` : 'idle'}
          {pipelines != null ? ` · pipelines: ${pipelines}` : ''}
        </div>

        <div className="progress-label">
          <span>Frame rate</span>
          <b>{fps != null ? `${fps.toFixed(1)} fps` : '-'}</b>
        </div>

        <h4 className="spaced">LINKS</h4>
        <ul className="links">
          <li onClick={() => open(REPO)}><FaGithub size={13} /> GitHub repo</li>
          <li onClick={() => open(PAYPAL)}><FaHeart size={13} /> Donate on PayPal</li>
          <li onClick={() => open(KOFI)}><FaCoffee size={13} /> Support on Ko-fi</li>
        </ul>
      </div>

      <div className={`play${busy ? ' busy' : ''}`} onClick={handlePlay}>
        <h1>{running === 'play' ? 'LIVE' : 'PLAY'}</h1>
        <FaPlayCircle size={180} color="#ff4f8b" />
      </div>
      <div className="launch-status">{status}</div>

      <div className="log-panel">
        <div className="log-head">
          <span>LOG</span>
          <div className="log-actions">
            <button className="pill" disabled={busy || (!state.repoRoot && !state.bundled)} onClick={handleSetup}>
              <FaTools size={11} /> {state.bundled ? 'PRECOMPILE SHADERS' : 'SETUP'}
            </button>
            {busy && (
              <button className="pill danger" onClick={() => bridge.stop()}>
                <FaStop size={11} /> STOP
              </button>
            )}
          </div>
        </div>
        <div className="log-body" ref={logRef}>
          {lines.length === 0 && <div className="line info">Waiting for output…</div>}
          {lines.map((l, i) => (
            <div key={i} className={`line ${l.stream}`}>{l.text}</div>
          ))}
        </div>
      </div>

      {showOptions && (
        <div className="options-panel">
          <h4>OPTIONS</h4>
          {!state.bundled && (<label className="row-opt">
            <span>Runtime</span>
            <select
              value={opts.runtime}
              onChange={(e) => updateOptions({ runtime: e.target.value })}
            >
              <option value="xcl">XCL (default)</option>
              <option value="victory">VIctory</option>
            </select>
          </label>)}
          {!state.bundled && (<label className={`check${opts.runtime !== 'victory' ? ' disabled' : ''}`}>
            <input
              type="checkbox"
              checked={opts.seriesS}
              disabled={opts.runtime !== 'victory'}
              onChange={(e) => updateOptions({ seriesS: e.target.checked })}
            />
            <span className="box" />
            <span>Series S mode (VIctory)</span>
          </label>)}
          <label className="check">
            <input
              type="checkbox"
              checked={opts.autoMenu}
              onChange={(e) => updateOptions({ autoMenu: e.target.checked })}
            />
            <span className="box" />
            <span>Auto-advance menus</span>
          </label>
          {!state.bundled && state.repoRoot && <div className="mini path" title={state.repoRoot}>{state.repoRoot}</div>}
          <div className="mini">{state.bundled ? 'Bundled XCL runtime' : state.built ? 'XCL runtime built' : 'XCL runtime not built: run SETUP'}</div>
        </div>
      )}

      <div className="config" onClick={() => setShowOptions((v) => !v)}>
        <FaCog size={19} color="#fff" />
      </div>
      <img src={logo} alt="ScarlettPort" className="logo" />
    </Window>
  );
}
