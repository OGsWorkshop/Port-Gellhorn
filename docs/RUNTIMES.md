# The two runtimes

Both runtimes run the same game dump. Each currently gets further at something different.

| | VIctory stack (`runtime/victory`) | XCL (`runtime/xcl`) |
|---|---|---|
| Shader path | RDNA2 to DXIL directly | RDNA2 to HLSL to DXC to DXIL |
| Menus and loading | not the focus | 30 fps |
| Open world | world geometry on screen, movement tutorial reached | world reached, survives 7+ minutes |
| Speed in the world | about 1 fps | not rendering the final image yet |
| Final image | black (lighting shaders still failing) | black |
| Extras | debug colour view of internal buffers, frame capture | launcher, journal/offline pipeline build, profiles, achievements, crash reports |
| Scripts | `setup-victory.ps1`, `play-victory.ps1` | `setup.ps1`, `play.ps1`, launcher |

## VIctory stack

Vendored source of the `lane` branch: the AnyXboxX stable VIctory stack plus parts of upstream PR #994 and PR #1407 and our fixes (Windows 10 Agility SDK opt-in, auto input, press file with stick tokens, capture options, `VICTORY_SHOW_TARGET` selector, dropped-command-list guard, mesh lane mask read as data, uniform `v_readfirstlane` in divergent flow, descriptor heap cap). Provenance: `runtime/victory/SCARLETTPORT.md`.

```powershell
.\scripts\setup-victory.ps1
.\scripts\play-victory.ps1 -Game "D:\path\to\dump" -AutoInput -DebugView
```

Manual equivalent: build with `cargo rustc --release -p victory --bin victory --features diagnostics --locked -- -C link-arg=/EXPORT:D3D12SDKVersion,DATA -C link-arg=/EXPORT:D3D12SDKPath,DATA`, put the Agility SDK 1.614 DLLs in a `D3D12` folder next to `victory.exe`, then `victory.exe run "<dump>\game_scarlett_master.exe" --execute`.

| Variable | Effect |
|---|---|
| `VICTORY_GAME_DIR` | the dump folder (required) |
| `VICTORY_SHADER_LOOP_LIMIT=32` | cap on shader loop iterations |
| `VICTORY_AUTO_INPUT=1` | press A on a timer |
| `VICTORY_PRESS_FILE=<file>` | live input: button and stick tokens written to the file are pressed |
| `VICTORY_DEBUG_PS=1` | debug colour pixel shaders |
| `VICTORY_SHOW_TARGET=WxH-fN` | show the internal render target of that size at frame N in the window |
| `VICTORY_CAPTURE_EVERY=N` | save the frame and every render target every N frames |
| `VICTORY_DUMP_FRAMES` | dump frames (diagnostics build) |
| `VICTORY_WAVE64_EMULATE=1` | emulate 64-lane waves on 32-lane GPUs |

## XCL

Our modified XCL tree: console OS layer, D3D12.X, RDNA2 to HLSL to DXIL recompiler. Run it through `play.ps1` or the launcher (`launcher/`, one exe with the runtime embedded). Our changes against upstream are in `patches/xcl-stack.patch`.

| Variable | Effect |
|---|---|
| `XCL_DXC_THREADS=N` | shader compile threads (`play.ps1` uses 4) |
| `XCL_PIPELINE_THREADS=N` | pipeline build threads (`setup.ps1` uses 3) |
| `XCL_NO_WATCHDOG=1` | disable the game's 30 s stall watchdog |
| `XCL_PRESS_FILE=<file>` | live input from a file |
| `XCL_PRESS_AT=20:A,23:DOWN` | scripted presses (second:button) |
| `XCL_LAZY_PSO=0` | compile every pipeline up front |
| `XCL_FLAT=1` | tiny flat stand-in for the big lighting shaders |
| `XCL_TWO_PASS_INDIRECT=1` | draw two-pass mesh calls (off by default; can reset the GPU) |

Offline path: the runtime journals every pipeline the game creates; `xcl pipelines <game>` builds them ahead of time, and `setup.ps1 -Game <dump>` precompiles the whole shader pack.

## Patches

`patches/` holds each runtime's changes as a plain diff.

| Patch | Base |
|---|---|
| `xcl-stack.patch` | upstream XCL `main` |
| `victory-stack.patch` | upstream Marvisak/VIctory `main` (merge-base 58fed37); includes the AnyXboxX stack and the merged PR parts |
