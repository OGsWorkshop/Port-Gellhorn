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
| `victory-01-wave64-emulation-and-agility-sdk.patch` | VIctory `d2a4bae` | wave64 emulation on 32-lane GPUs; Agility SDK opt-in so Windows 10 works |
| `victory-02-auto-input.patch` | above | `VICTORY_AUTO_INPUT`: hands-free menu presses |
