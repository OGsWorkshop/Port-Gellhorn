// ScarlettPort launcher: Electron main process.
// Runs the repo's PowerShell scripts, streams their output to the UI and tails the game log.
const { app, BrowserWindow, ipcMain, dialog, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn, execFile } = require('child_process');

const isDev = !app.isPackaged;
const GAME_EXE = 'game_scarlett_master.exe';
const ALLOWED_URLS = [
  'https://github.com/ZigZagBoi1/ScarlettPort',
  'https://www.paypal.me/SalwanStudios',
  'https://ko-fi.com/ziggymar',
];

let mainWindow = null;
let child = null; // { proc, kind, tail }

// ---- persisted config (JSON in app data) ----
const configPath = () => path.join(app.getPath('userData'), 'config.json');
function loadConfig() {
  try {
    return JSON.parse(fs.readFileSync(configPath(), 'utf-8'));
  } catch (e) {
    return {};
  }
}
function saveConfig(patch) {
  const next = Object.assign(loadConfig(), patch);
  try {
    fs.mkdirSync(path.dirname(configPath()), { recursive: true });
    fs.writeFileSync(configPath(), JSON.stringify(next, null, 2), 'utf-8');
  } catch (e) {
    /* ignore */
  }
  return next;
}

// ---- locate the repo root (the folder that holds scripts/setup.ps1) ----
const isRepo = (dir) => !!dir && fs.existsSync(path.join(dir, 'scripts', 'setup.ps1'));
function findRepoRoot() {
  const cfg = loadConfig();
  if (isRepo(cfg.repoRoot)) return cfg.repoRoot;
  const starts = [path.resolve(__dirname, '..', '..')];
  if (process.env.PORTABLE_EXECUTABLE_DIR) starts.push(process.env.PORTABLE_EXECUTABLE_DIR);
  starts.push(path.dirname(process.execPath), process.cwd());
  for (const s of starts) {
    let d = s;
    for (let i = 0; i < 6; i++) {
      if (isRepo(d)) return d;
      const up = path.dirname(d);
      if (up === d) break;
      d = up;
    }
  }
  return null;
}

// Bundled runtime (packaged build): resources/runtime/{xcl.exe,D3D12,dxcompiler.dll,dxil.dll}
const bundledDir = () => path.join(process.resourcesPath || '', 'runtime');
const hasBundled = () => app.isPackaged && fs.existsSync(path.join(bundledDir(), 'xcl.exe'));

const validGame = (dir) => !!dir && fs.existsSync(path.join(dir, GAME_EXE));

function getState() {
  const cfg = loadConfig();
  const repoRoot = findRepoRoot();
  const game = validGame(cfg.gameDir) ? cfg.gameDir : '';
  return {
    gameDir: game,
    repoRoot: repoRoot || '',
    bundled: hasBundled(),
    built: !!repoRoot && fs.existsSync(path.join(repoRoot, 'runtime', 'xcl', 'target', 'release', 'xcl.exe')),
    options: Object.assign({ runtime: 'xcl', seriesS: false, autoMenu: false }, cfg.options),
    running: child ? child.kind : null,
  };
}

function send(channel, payload) {
  if (mainWindow && !mainWindow.isDestroyed()) mainWindow.webContents.send(channel, payload);
}
const log = (text, stream = 'info') => send('log', { text, stream });

// ---- window ----
function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 720,
    resizable: false,
    frame: false,
    backgroundColor: '#0f1923',
    title: 'ScarlettPort',
    icon: path.join(__dirname, '..', 'src', 'assets', 'icon.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });
  mainWindow.setMenu(null);
  const startUrl = isDev
    ? 'http://localhost:3000'
    : `file://${path.join(__dirname, '..', 'build', 'index.html')}`;
  mainWindow.loadURL(startUrl);
  if (isDev && process.env.SCARLETT_DEVTOOLS) mainWindow.webContents.openDevTools({ mode: 'detach' });
}

app.whenReady().then(createWindow);
app.on('window-all-closed', () => {
  stopChild();
  app.quit();
});

// ---- process handling ----
function lineSplitter(onLine) {
  let buf = '';
  return (chunk) => {
    buf += chunk.toString('utf8');
    const parts = buf.split(/\r\n|\n|\r/);
    buf = parts.pop();
    parts.forEach((l) => l.length && onLine(l));
  };
}

function tailFile(file, onLine) {
  let pos = 0;
  let busy = false;
  const split = lineSplitter(onLine);
  const timer = setInterval(() => {
    if (busy) return;
    fs.stat(file, (err, st) => {
      if (err || st.size <= pos) return;
      busy = true;
      const stream = fs.createReadStream(file, { start: pos, end: st.size - 1 });
      pos = st.size;
      stream.on('data', split);
      stream.on('close', () => (busy = false));
      stream.on('error', () => (busy = false));
    });
  }, 500);
  return () => clearInterval(timer);
}

function stopChild() {
  if (!child) return;
  const pid = child.proc.pid;
  try {
    execFile('taskkill', ['/pid', String(pid), '/t', '/f'], () => {});
  } catch (e) {
    /* ignore */
  }
}

function runPs(kind, psArgs, cwd) {
  const args = ['-NoProfile', '-ExecutionPolicy', 'Bypass', ...psArgs];
  return runProc(kind, 'powershell.exe', args, cwd, process.env);
}

function runProc(kind, exe, args, cwd, env) {
  if (child) return { ok: false, error: `Already running (${child.kind}).` };
  const proc = spawn(exe, args, { cwd, env, windowsHide: true });
  const entry = { proc, kind, tail: null };
  child = entry;
  send('run-state', { running: kind });
  const onLine = (stream) => (line) => {
    const m = /^Starting\.\.\. log: (.+)$/.exec(line.trim());
    if (m && !entry.tail) {
      entry.tail = tailFile(m[1], (l) => log(l, 'game'));
    }
    log(line, stream);
  };
  proc.stdout.on('data', lineSplitter(onLine(kind === 'play' && exe.endsWith('xcl.exe') ? 'game' : 'out')));
  proc.stderr.on('data', lineSplitter(onLine(kind === 'play' && exe.endsWith('xcl.exe') ? 'game' : 'err')));
  proc.on('error', (e) => log(`Failed to start PowerShell: ${e.message}`, 'err'));
  proc.on('close', (code) => {
    if (entry.tail) setTimeout(entry.tail, 1000); // let the last log lines flush
    if (child === entry) child = null;
    log(`Process ended (exit code ${code}).`, code === 0 ? 'info' : 'err');
    send('run-state', { running: null, code, kind });
  });
  return { ok: true };
}

const q = (s) => `'${String(s).replace(/'/g, "''")}'`;

// ---- IPC ----
ipcMain.handle('state:get', () => getState());

ipcMain.handle('game:pick', async () => {
  const r = await dialog.showOpenDialog(mainWindow, {
    title: 'Select your game folder',
    properties: ['openDirectory'],
  });
  if (r.canceled || !r.filePaths[0]) return { ok: false, canceled: true };
  const dir = r.filePaths[0];
  if (!validGame(dir)) return { ok: false, error: `That folder does not contain ${GAME_EXE}.` };
  saveConfig({ gameDir: dir });
  return { ok: true, state: getState() };
});

ipcMain.handle('repo:pick', async () => {
  const r = await dialog.showOpenDialog(mainWindow, {
    title: 'Select the ScarlettPort repo folder (contains scripts\\setup.ps1)',
    properties: ['openDirectory'],
  });
  if (r.canceled || !r.filePaths[0]) return { ok: false, canceled: true };
  if (!isRepo(r.filePaths[0])) return { ok: false, error: 'scripts\\setup.ps1 not found in that folder.' };
  saveConfig({ repoRoot: r.filePaths[0] });
  return { ok: true, state: getState() };
});

ipcMain.handle('options:set', (_e, opts) => {
  const o = {
    runtime: opts && opts.runtime === 'victory' ? 'victory' : 'xcl',
    seriesS: !!(opts && opts.seriesS),
    autoMenu: !!(opts && opts.autoMenu),
  };
  saveConfig({ options: o });
  return { ok: true };
});

function bundledEnv(extra) {
  return Object.assign({}, process.env, {
    XCL_NO_WATCHDOG: '1',
    XCL_DXC_THREADS: '4',
    XCL_DXC_DIR: bundledDir(),
  }, extra);
}

ipcMain.handle('run:setup', () => {
  const st = getState();
  if (st.bundled) {
    if (!st.gameDir) return { ok: false, error: `Pick a game folder containing ${GAME_EXE} first.` };
    const pack = path.join(st.gameDir, 'shaders.rpf');
    if (!fs.existsSync(pack)) return { ok: false, error: 'No shaders.rpf in the game folder.' };
    log('> xcl.exe shaders shaders.rpf --precompile --pipelines --only cs', 'info');
    return runProc('setup', path.join(bundledDir(), 'xcl.exe'),
      ['shaders', pack, '--precompile', '--pipelines', '--only', 'cs'],
      bundledDir(), bundledEnv({ XCL_DXC_THREADS: '8', XCL_PIPELINE_THREADS: '3' }));
  }
  if (!st.repoRoot) return { ok: false, error: 'Repo folder not found. Use "Locate repo".' };
  const args = ['-File', path.join(st.repoRoot, 'scripts', 'setup.ps1')];
  if (st.gameDir) args.push('-Game', st.gameDir);
  log(`> setup.ps1${st.gameDir ? ' -Game "' + st.gameDir + '"' : ''}`, 'info');
  return runPs('setup', args, st.repoRoot);
});

ipcMain.handle('run:play', (_e, opts) => {
  const st = getState();
  if (st.bundled) {
    if (!st.gameDir) return { ok: false, error: `Pick a game folder containing ${GAME_EXE} first.` };
    const o = Object.assign({}, st.options, opts);
    const extra = {};
    if (o.autoMenu) {
      const t = [];
      for (let s = 22; s <= 70; s += 6) t.push(`${s}:A`);
      extra.XCL_PRESS_AT = t.join(',');
    }
    log(`> xcl.exe run "${st.gameDir}"${o.autoMenu ? ' (auto-menu)' : ''}`, 'info');
    return runProc('play', path.join(bundledDir(), 'xcl.exe'), ['run', st.gameDir], bundledDir(), bundledEnv(extra));
  }
  if (!st.repoRoot) return { ok: false, error: 'Repo folder not found. Use "Locate repo".' };
  if (!st.gameDir) return { ok: false, error: `Pick a game folder containing ${GAME_EXE} first.` };
  const o = Object.assign({}, st.options, opts);
  if (o.runtime === 'victory') {
    const rel = path.join(st.repoRoot, 'runtime', 'victory', 'target', 'release');
    const exe = path.join(rel, 'victory.exe');
    if (!fs.existsSync(exe)) {
      return { ok: false, error: 'victory.exe not built. See docs\\RUNTIMES.md ("Running VIctory").' };
    }
    const logsDir = path.join(st.repoRoot, 'logs');
    const script = [
      "$ErrorActionPreference='Stop'",
      `New-Item -ItemType Directory -Force ${q(logsDir)} | Out-Null`,
      `$log = Join-Path ${q(logsDir)} ('victory-{0:yyyyMMdd-HHmmss}.log' -f (Get-Date))`,
      'Write-Host "Starting... log: $log"',
      `Set-Location ${q(rel)}`,
      "$env:VICTORY_WAVE64_EMULATE='1'",
      o.autoMenu ? "$env:VICTORY_AUTO_INPUT='1'" : '',
      `& ${q(exe)} run ${q(path.join(st.gameDir, GAME_EXE))} --execute${o.seriesS ? ' --console series-s' : ''} *> $log`,
    ].filter(Boolean).join('; ');
    log(`> victory.exe run${o.seriesS ? ' --console series-s' : ''}`, 'info');
    return runPs('play', ['-Command', script], st.repoRoot);
  }
  const args = ['-File', path.join(st.repoRoot, 'scripts', 'play.ps1'), '-Game', st.gameDir];
  if (o.autoMenu) args.push('-AutoMenu');
  log(`> play.ps1 -Game "${st.gameDir}"${o.autoMenu ? ' -AutoMenu' : ''}`, 'info');
  return runPs('play', args, st.repoRoot);
});

ipcMain.handle('run:stop', () => {
  stopChild();
  return { ok: true };
});

ipcMain.handle('open:external', (_e, url) => {
  if (ALLOWED_URLS.includes(url)) shell.openExternal(url);
  return { ok: true };
});

ipcMain.handle('win:minimize', () => mainWindow && mainWindow.minimize());
ipcMain.handle('win:close', () => app.quit());
