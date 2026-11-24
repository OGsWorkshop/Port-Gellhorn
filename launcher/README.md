# ScarlettPort launcher

Electron + React desktop launcher for ScarlettPort. It runs `scripts\setup.ps1` and
`scripts\play.ps1` (or VIctory) for you and shows their output live.

- Pick your unpacked game folder (must contain `game_scarlett_master.exe`); it is remembered.
- **PLAY** starts the game. Options (gear button): runtime (XCL / VIctory), Series S mode
  (VIctory, `--console series-s`), auto-advance menus (`-AutoMenu`).
- **SETUP** runs `setup.ps1` (with `-Game` if a folder is chosen) and shows shader/pipeline progress.
- The log panel tails the game log; lines with `fps` update the FPS readout.
- Settings live in `%APPDATA%\scarlettport-launcher\config.json`. No telemetry, no network calls
  except the links you click.

## Run / build

```
cd launcher
npm install
npm run dev     # React dev server + Electron window
npm run dist    # builds a portable exe into launcher\dist
```

The launcher finds the repo by walking up from its own location (or the portable exe's
location) looking for `scripts\setup.ps1`; if it can't, use "LOCATE REPO" in the header.
