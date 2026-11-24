# The two runtimes

Both runtimes do the same job differently. We keep both because each currently gets further at something.

| | XCL (`runtime/xcl`) | VIctory (`runtime/victory`) |
|---|---|---|
| Shader path | RDNA2 → HLSL → DXC → DXIL | RDNA2 → DXIL directly |
| Shaders translated | 100% of the pack | most; its upstream integration branch reaches ~97% on NVIDIA |
| Menus visible | not yet on NVIDIA | ✅ 30 fps |
| Gets into the world | ✅ | ✅ |
| Default in `play.ps1` | ✅ | |

## Running VIctory (shows the menus today)

```powershell
cd runtime\victory
cargo rustc --release -p victory --bin victory --locked -- -C link-arg=/EXPORT:D3D12SDKVersion,DATA -C link-arg=/EXPORT:D3D12SDKPath,DATA
# put D3D12Core.dll (Agility SDK 1.614) in target\release\D3D12\ ; scripts\setup.ps1 downloads it for XCL, copy that folder
cd target\release
$env:VICTORY_WAVE64_EMULATE = '1'; $env:VICTORY_AUTO_INPUT = '1'
.\victory.exe run "D:\path\to\dump\game_scarlett_master.exe" --execute
```

## Our changes

Every change we made to a runtime is also in `patches/` as a plain diff against the upstream commit:

| Patch | Base | What it does |
|---|---|---|
| `xcl-stack.patch` | xcl `main` | everything this repo's XCL adds: upstream PRs #684, #850, #851, #891, #900, #906 and #944, the BCPack decoder from #905, a 4096-iteration cap on shader loops, `XCL_PRESS_FILE` live input, and `XCL_SKIP_CS_OVER` (a debugging switch) |
| `victory-stack.patch` | VIctory `main` | upstream PR #578 (refuse wave64 programs a 32-lane GPU cannot run, instead of hanging); Agility SDK opt-in for Windows 10; `VICTORY_PRESS_FILE` live input; `VICTORY_CAPTURE_EVERY=N` (saves the frame and every render target every N frames) |

## Where the world stands (NVIDIA, 32-lane GPUs)

With VIctory `main` + #578 in Series S mode (`--console series-s`, about half the memory), the game enters the open world and its frame graph runs: around 190 render targets per frame, the screen-space buffers and simulation maps are being written. The G-buffer stays empty because the world's geometry is drawn by **wave64 mesh shaders**, which a 32-lane GPU can't run as-is. Translating those (XCL's two-pass mesh path, or a wave64-on-wave32 mesh lowering in VIctory) is the remaining wall.
