<div align="center">

# ScarlettPort

**A native Windows port layer for a next-gen Xbox Series X|S open-world title, running from your own files.**
No emulator. No streaming. The game's own x64 code runs on your CPU, and its RDNA2 GPU code is recompiled for your graphics card on the fly.

### Progress to playable

![progress](https://img.shields.io/badge/progress-20%25-ff2d95?style=for-the-badge)

```
â–ˆâ–ˆâ–ˆâ–ˆâ–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘  20%
```

`boots` âœ… â†’ `intro & menus` âœ… â†’ `world loads` âœ… â†’ `world renders` ðŸ”¨ â†’ `gameplay` â³ â†’ `30 fps 1:1` â³

<a href="https://www.paypal.me/SalwanStudios"><img src="https://img.shields.io/badge/DONATE-PayPal-blue?style=for-the-badge&logo=paypal&logoColor=white" alt="Donate via PayPal" height="45"/></a>
<a href="https://ko-fi.com/ziggymar"><img src="https://img.shields.io/badge/BUY%20A%20COFFEE-Ko--fi-ff5f5f?style=for-the-badge&logo=kofi&logoColor=white" alt="Support on Ko-fi" height="45"/></a>

<sub>PayPal: please send as <b>Friends & Family</b> to avoid holds and fees.</sub>

</div>

---

## ðŸ“¸ Running right now

### ðŸ†• First open-world geometry on an NVIDIA GPU

![First world geometry](assets/first-world-geometry.png)

<sub>Captured from inside the open world on an RTX 3060 (Series S profile): one of the game's own render passes drawing the ground in true 3D perspective. It's an internal pass, not the final lit image yet, but it's the world rendering. Next step: the geometry passes that feed the final image.</sub>

The title's own artwork, menus and loading screens, drawn natively on an **RTX 3060** at a locked **30 fps**:

| Title artwork | Loading screen | Loading screen |
|:---:|:---:|:---:|
| ![Title artwork](assets/title-art.png) | ![Loading screen](assets/loading-1.png) | ![Loading screen](assets/loading-2.png) |
| ![Lucia loading screen](assets/loading-lucia.png) | | |

| First-boot setup | Legal notices | Terms & policies |
|:---:|:---:|:---:|
| ![Subtitles setup screen](assets/menu-subtitles.png) | ![Legal notices screen](assets/menu-legal.png) | ![Terms screen](assets/menu-terms.png) |

<sub>Gameplay screenshots go here the moment the world renders. Watch the repo â­ so you see it first.</sub>

---

## ðŸš¦ Where we are

| Stage | Status | Notes |
|---|:---:|---|
| Executable loads, imports bind | âœ… | 0 missing imports; the game's 16 console DLLs load |
| Intro, setup and menus | âœ… | 30 fps, audio plays |
| Shaders translate | âœ… | all 26,084 programs in the pack translate (XCL) |
| Exposure / tonemapping chain | âœ… | all 36 luminance & tonemap shaders translate |
| Open world loads | âœ… | world packs stream in through DirectStorage |
| Open world renders | ðŸ”¨ | **the current wall**: black screen after loading (see below) |
| Gameplay | â³ | |
| 1:1 at 30 fps on a mid-range PC | â³ | the goal |

### The current wall: the first seconds in the world

The game boots, runs its menus, loads the open world (peds and cars spawn, and the in-world audio plays), and then dies within seconds. We've narrowed down why:

- When the world starts, the game asks for **~10,000 GPU programs at once**. Compiling them on NVIDIA takes far longer than the game will wait.
- If those draws **wait** for their programs, the game's own 30-second watchdog decides it has hung and quits.
- If those draws are **skipped**, their outputs are never written. Later GPU work reads that garbage (e.g. indirect draw counts), runs far past Windows' 2-second GPU timeout, and the device resets.

**The fix is to have every program compiled before the game starts.** `setup.ps1 -Game <dump>` now precompiles the whole shader pack and the GPU pipelines once (about 45 minutes; it's resumable). Making that faster and complete is task #1 below.

**New:** `play.ps1` now turns off the game's stall watchdog (`XCL_NO_WATCHDOG`), so the game waits out slow first-time shader builds in the world instead of quitting. The first world load is slow while the cache fills, but it no longer dies.

**Latest (Nov 24):**
- **World asset streaming fixed.** Map, prop and VFX files failed to load once the world started, because requests went out before the game had filled in where the data goes, and the game then waited forever. Requests now wait briefly for the game to finish, and the runs since have had zero failed loads.
- **Unlit mode (`XCL_FLAT=1`).** Big lighting shaders get a tiny flat grey stand-in, so the world can show without waiting on NVIDIA's slow compiles. Menus and loading screens keep their real shaders.
- **Pipeline journal.** Every GPU pipeline the game creates is recorded, even in runs that crash, and `xcl pipelines <game>` builds them all ahead of time. This is the path to a one-time "build shaders" step instead of compiling at world start.
- **Current wall:** the moment the world starts, a piece of GPU work runs past Windows' 2-second limit and the driver resets. Finding which one is task #3.

Already fixed in this repo's runtime: the 64-bit wait ordering (black screen), the BCPack texture decoder (empty world textures), the `CopyFromSubresource` crash at the end of loading, and runaway shader loops.

---

## ðŸš€ Run it

> **You need your own unpacked Xbox Series copy of the game.** This repo contains **no game files** of any kind, and never will.

**Requirements:** Windows 10/11 x64 Â· a D3D12 GPU with Shader Model 6.6 and mesh shaders (RTX 20-series / RX 6000 or newer) Â· 32 GB RAM recommended Â· [Rust](https://rustup.rs) (stable MSVC) Â· Windows SDK (for DXC)

```powershell
git clone https://github.com/ZigZagBoi1/ScarlettPort.git
cd ScarlettPort
.\scripts\setup.ps1 -Game "D:\path\to\your\dump"     # build + Agility SDK + precompile all shaders (~45 min, once)
.\scripts\play.ps1  -Game "D:\path\to\your\dump" -AutoMenu
```

- `-AutoMenu` presses A for you so the menus advance without a controller.
- While the game runs, write button names into `press.txt` (e.g. `DOWN DOWN A`) to press them. Handy for scripting and for AI agents testing a build.
- Logs go to `logs\`. Attach one when you open an issue.
- Controls: an Xbox controller works best. On keyboard and mouse, Enter = A, Esc = B, WASD = left stick, and the mouse moves the right stick.

Want the build that currently shows the menus? See [`docs/RUNTIMES.md`](docs/RUNTIMES.md).

---

## ðŸ¤– Task board: grab one, send a PR

Every task is sized for **one person + one AI coding agent**. Pick one, comment on its issue to claim it, and send a PR. Read [`CONTRIBUTING.md`](CONTRIBUTING.md) first (it takes two minutes).

### ðŸ”¥ Critical path to gameplay

- [ ] **#1 Zero shader compiles at world load.** `setup.ps1 -Game` precompiles the pack; make sure the world's programs are then *all* cache hits at runtime (`compiling shaders: N done` should stay at 0). Find what still misses (graphics pipelines, mesh passes, split-wave64 variants) and precompile those too.
- [ ] **#2 Keep the watchdog fed.** While a program is still building, keep the game's main-thread heartbeat alive without skipping work. See `pso_async.rs` and issue #505 in the XCL repo for the watchdog's address and logic.
- [ ] **#3 Never read garbage from a skipped dispatch.** When a dispatch is skipped, zero or clamp the indirect-argument buffers it would have written, so an `ExecuteIndirect` can't run for minutes.
- [ ] **#4 Mesh pipelines with the MS in the AS slot.** The game uses these and upstream XCL refuses them; an experimental translation exists in the git history. Make them draw correctly.

### ðŸŽ¨ Rendering

- [ ] **#5 Why do VIctory's newer wave64 builds render black menus?** VIctory PR #439 and #414 builds draw black, while the older `wave64-emulation` build shows the menus. Bisect it.
- [ ] **#6 Clear VIctory's remaining shader failures.** The biggest groups are SGPR addresses that aren't root parameters (195), wave64 groups larger than 64 threads (155), and descriptors read from computed VGPRs (138).
- [ ] **#7 Faster wave64 on NVIDIA.** Console 64-lane programs run as two 32-lane halves. Profile them and cut the overhead.

### âš¡ Performance

- [ ] **#8 Memory copy bottleneck.** Up to ~600 ms per frame goes to copying title memory in world scenes. Batch it, use async copy queues, or skip clean pages.
- [ ] **#9 Shader cache warm-up script.** Ship a script that fills the cache offline so first launch isn't a slideshow.

### ðŸ§° Tooling & docs

- [ ] **#10 One-click launcher.** A `.bat` or small GUI that finds the dump, runs setup if needed, and starts the game.
- [ ] **#11 Hardware reports.** Run it on your GPU and post results (GPU, driver, fps, how far it got) to the pinned issue.
- [ ] **#12 CI.** Build both runtimes on Windows on every PR.

---

## ðŸ—‚ï¸ What's in here

```
ScarlettPort/
â”œâ”€â”€ scripts/          setup.ps1, play.ps1: build and run
â”œâ”€â”€ runtime/
â”‚   â”œâ”€â”€ xcl/          main runtime: console OS, D3D12.X, RDNA2 â†’ HLSL â†’ DXIL recompiler
â”‚   â””â”€â”€ victory/      second runtime: RDNA2 â†’ DXIL directly; currently shows the menus
â”œâ”€â”€ patches/          our changes to each runtime, as plain diffs (easy to review and upstream)
â”œâ”€â”€ docs/             STATUS, RUNTIMES, technical findings
â””â”€â”€ assets/           screenshots
```

---

## âš–ï¸ Legal

- This project contains **no code, assets or files from the game or its publisher**, and no console SDK code. You must own the game and provide your own dump.
- Nothing here decrypts, unpacks or bypasses DRM.
- `runtime/xcl` is GPL-2.0-only, Â© the XCL authors. `runtime/victory` is GPL-3.0-or-later, Â© the VIctory authors. Each keeps its own `LICENSE`. See [`LICENSE.md`](LICENSE.md).

<div align="center">

**If this got you hyped, star it â­ and share it. Every PR gets us closer to playable.**

</div>
