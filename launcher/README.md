# ScarlettPort launcher

Native Rust launcher (eframe/egui). One exe: the XCL runtime, D3D12 Agility SDK and DXC are
embedded and unpacked to `%LOCALAPPDATA%\ScarlettPort\runtime` on first run (skipped when the hash matches).
Settings live in `%LOCALAPPDATA%\ScarlettPort\launcher.json`. No network access except the links you click.

- **Play**: pick the folder containing `game_scarlett_master.exe`, press PLAY. Live FPS is read from the log.
- **Setup**: precompile shaders (`xcl.exe shaders ... --precompile --pipelines --only cs`) with progress.
- **Logs**, **Progress notes**, **Links**, **Settings** (game folder, auto-advance menus).

## Build

```
cd launcher
cargo build --release --locked      # -> target\release\ScarlettPort.exe
```

`build.rs` copies the runtime files into `launcher\runtime\` from `M:\GTA6\xcl-test` and the Windows
SDK (`C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64`); override with `XCL_SRC` / `DXC_SRC`.
`ScarlettPort.exe --screenshot=FILE.png` renders the window to a PNG and exits.
