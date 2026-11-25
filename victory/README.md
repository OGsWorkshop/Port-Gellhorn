# VIctory

Runs Grand Theft Auto VI (Xbox Series X|S) natively on Windows, from files
the user supplies. In Port Gellhorn this runtime lives in `victory/` as a git
subtree with its original history. It runs the real Xbox executable and
presents through host D3D12; the old Leonida map viewer is not the current
build. See [docs/architecture.md](docs/architecture.md).

> [!IMPORTANT]
> No game data, executable, key or protection workaround is in the repository.
> The only title executable used for verification is the user's own retail
> `game_scarlett_master.exe`; no reconstructed EXE, helper or viewer is
> presented as GTA VI.

## Status (2026-11-24)

**The game boots and renders the real world. It is not playable, and not
smooth on the test machine.**

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about 50 s) | works |
| The world (first mission's intro) | renders through host D3D12 with indirect draws and thousands of draws per frame; ~0.5-3 fps during world streaming |

Tested on Windows 11, NVIDIA GeForce RTX 3060 Ti, driver **32.0.16.2124**,
16 GB RAM. Real D3D12 world rendering is confirmed on that GPU in release and
diagnostics runs. Streaming is limited by CPU-side copy/watch work and
draw/pipeline overhead, not by a missing presentation path; faster PCs may do
better, but no smooth-performance claim is made. Captured frames stay local
and are not distributed.

ScarlettPort, included as a subtree at [`scarlettport/`](../scarlettport),
reports **2-4 fps** in the world from its vendored VIctory runtime
([source](https://github.com/ZigZagBoi1/ScarlettPort/blob/main/runtime/victory/README.md)):
420-450 ms per frame by default, or 240-350 ms with their newer switches
(`VICTORY_RELAY=2`, or `VICTORY_LATE_WAIT=3` with `VICTORY_WIDE_BACK=1`). Its
top-level docs quote 3-4 fps on an RTX 3060 with 32 GB RAM, Series S profile
and its own upscaler. These are their measurements on their hardware; they are
not a substitute for a measurement of this build, whose figure above stays
unchanged until the merged runtime is run and timed.

Early builds peaked at ~9.1 GB VRAM; lazy host-texture creation and capped
placed allocations bring that to ~5.1 GB. PSV structured-buffer shader
translation is fixed; sparse/unmapped-page handling and per-submission watch
sharing are still being finalized. These reduce footprint, not frame time.

> **Warning.** Keep `-DebugLayer` / `VICTORY_D3D12_DEBUG=1` off on this driver:
> the D3D12 debug layer fails with `0x80070057` (`E_INVALIDARG`), so the run
> never reaches the title. Earlier builds could hang a real GPU and reset the
> display driver; current runs on the 3060 Ti complete. A diagnostics build
> with `VICTORY_D3D12_WARP=1` renders on the software adapter as a diagnosis
> fallback, not a fix.

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's real executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard, keyboard stand-in pad | Trigger rumble |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | Mesh pipelines, shader-written buffers, descriptor lifetimes, indirect execution (arguments clamped), pipelines made by worker threads and kept on disk | Sparse/unmapped-page and per-submission watch work is being finalized; real queries, depth bounds, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL at first use, cached on disk. Menus: all. World: mesh 8,191 of 8,506, vertex/pixel 10,617 of 12,346, compute 1,643 of 2,539 in the bundled snapshot; PSV structured-buffer translation is fixed | 64-bit image atomics, merged geometry stages, the ray-tracing builder's shaders; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses; host textures created lazily, placed allocations capped | Render-target metadata planes |
| Series S mode | Runs through the menus to the loading screen | The world, as above |

Shader counts are from the bundled source snapshot; local validation builds
may be ahead of them.

## Getting started

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

The title also needs several GiB of available Windows commit at startup.
Check **Committed** under **Task Manager → Performance → Memory**. If
`crashes.log` reports Windows error 1455, close other applications or increase
the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy victory.example.toml victory.toml   # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p victory -- config           # check your paths
```

Run the title on a real GPU:

```powershell
cargo build --release -p victory
$env:VICTORY_GAME_DIR = '<your game package>'
$env:VICTORY_CACHE_DIR = '<outside the repo>\cache'
$env:VICTORY_DATA_DIR = '<outside the repo>\data'
# Keep VICTORY_D3D12_DEBUG unset (see the warning above).
target\release\victory.exe run "$env:VICTORY_GAME_DIR\game_scarlett_master.exe" --execute
```

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\victory\data`), derived
data under the cache folder.

> **Local validation only.** The real-GPU reproduction used an untracked helper
> at `C:\Users\rhyth\victory-integration\LAUNCH_GTA6_INTEGRATION_GPU.ps1` with
> `-RunName <name> -CacheRun render1 -LowRam`. It is not distributed with the
> repository and is not a setup step for a clone. `-LowRam` selects the
> low-memory path (lazy host textures, capped placed allocations);
> `-DebugLayer` was not used, for the driver reason above. Do not add the
> helper, its logs or its captures to git.

**Data policy.** No archives, executables, shaders, textures, audio, keys,
captured frames, caches or logs are committed; all paths stay outside the
repository (`.gitignore` and `cargo xtask policy` enforce this). Captured
frames from local verification stay on the local machine.

**Cargo.** Set `VICTORY_*` variables per process, not in the user or machine
environment: inherited values change `cargo build` and `cargo test` in other
terminals. No Cargo configuration change is needed; `victory/.cargo/config.toml`
only defines the `xtask` alias.

Input: XInput pads. With none connected the keyboard is a pad while the
title's window has focus:

| Keys | Pad |
|------|-----|
| W A S D / I J K L | left / right stick |
| arrows | D-pad |
| Enter, Backspace, X, Y | A, B, X, Y |
| Q / E, Z / C | shoulders, triggers |
| F / H | stick clicks |
| Esc, Tab | Menu, View |

## Commands

```
victory inspect <exe|dll> [--imports] [--exports]
victory census [paths...] [--missing] [--strict]
victory lib-exports <import.lib> [--list]
victory shader-info [paths...] [--verbose] [--disasm]
victory translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
victory run <exe> [--execute] [--unimplemented trap|zero] [--strict]
            [--console series-x|series-s] [-- <title arguments>]
victory config
```

Logging: `VICTORY_LOG=victory_loader=trace,info` (a release build logs
info and above).

## Debugging

A release build has no debugging aids. The variables below are for diagnosis
only: none restores a missing draw, produces final video, or is a fix. Build
them in with

```powershell
cargo build --profile diag -p victory --features diagnostics   # target\diag\victory.exe
```

| Variable | Effect |
|----------|--------|
| `VICTORY_LOG=<target>=debug,...` | debug and trace output |
| `VICTORY_STACKS=20,60` | every thread's stack at those seconds |
| `VICTORY_WATCH=<rva>,...` | logs a title u32 when it changes |
| `VICTORY_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `VICTORY_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `VICTORY_CAPTURE=<dir>` | saves presented frames (powers of two, or `VICTORY_CAPTURE_FRAMES=<list>`) |
| `VICTORY_SHOW_TARGET` | inspect render-target selection while diagnosing a missing pass |
| `VICTORY_DUMP_TARGETS=1` | with capture set, saves every render target of those frames |
| `VICTORY_D3D12_DEBUG=1` | host debug layer — unusable on the driver above (`0x80070057`); leave it off |
| `VICTORY_D3D12_GBV=1` | with the debug layer, GPU-based validation (bad descriptor use reported, not faulted; slow) |
| `VICTORY_D3D12_DRED=1` | on a GPU failure, the allocation hit and the commands in flight |
| `VICTORY_SKIP_PIPELINES=<hash>,...` | omit draws and dispatches, including indirect ones, whose pipeline contains a listed 16-digit shader hash |
| `VICTORY_GPU_TIMING=1` | with the above, each list of a submission run and timed on its own; slow lists logged with their slowest commands |
| `VICTORY_D3D12_WARP=1` | host device on the software adapter (slow; cannot reset the driver) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane instead of 2^20 (own shader cache entries); if a lower `n` makes world frames much faster, loops hitting the limit are what makes them slow |
| `VICTORY_AUDIO_CAPTURE=<file.wav>` | records what the title plays (our process only) |
| `VICTORY_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends crashes and host GPU failures to `crashes.log` in the
title's data folder.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
