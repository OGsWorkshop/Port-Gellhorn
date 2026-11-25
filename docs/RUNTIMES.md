# The two runtimes

Both runtimes run a title from your own dump. Each currently gets further at something different.

| | VIctory-based runtime (`runtime/victory`) | XCL (`runtime/xcl`) |
|---|---|---|
| Shader path | RDNA2 to DXIL directly | RDNA2 to HLSL to DXC to DXIL |
| Menus and loading | 30 fps | 30 fps |
| Open world | lit world reached on an RTX 3060, Series S profile | world reached, survives 7+ minutes |
| Speed in the world | about 3-4 fps (RTX 3060, Series S profile) | not rendering the final image |
| Final image | lit; shadowed areas too dark | black |
| Extras | frame capture, save slots, press file, fast boot (testing) | launcher, pipeline journal and offline build, profiles, achievements, crash reports |
| Scripts | `setup-victory.ps1`, `play-victory.ps1` | `setup.ps1`, `play.ps1`, launcher |

Neither is verified playable.

## VIctory-based runtime

Vendored source of branch `fx-merge` at `1c3c4ba`: fxceless/victory-fork (`main` at `4e17478`, GPL-3.0) with community pull requests merged and our changes on top. Credits and the full lineage: `runtime/victory/SCARLETTPORT.md`. Upstream is VIctory by Marvisak.

```powershell
.\scripts\setup-victory.ps1
.\scripts\play-victory.ps1 -Game "D:\path\to\your\dump"
```

Manual equivalent: build with `cargo rustc --release -p victory --bin victory --features diagnostics --locked -- -C link-arg=/EXPORT:D3D12SDKVersion,DATA -C link-arg=/EXPORT:D3D12SDKPath,DATA`, put the Agility SDK 1.614 DLLs in a `D3D12` folder next to `victory.exe`, then `victory.exe run "<dump>\game_scarlett_master.exe" --execute --console series-s`.

`--console series-s` (the profile the measurements here were taken on) is the recommended default; `series-x` is the runtime's own default.

Every `VICTORY_*` variable can also be given as `SCARLETT_*`. The full list is in the source and in `runtime/victory/README.md`; the ones the scripts use or that matter most:

| Variable | Effect |
|---|---|
| `VICTORY_GAME_DIR` | the dump folder (required) |
| `VICTORY_OWN_UPSCALER=1` | ask for the title's own upscaler instead of FSR on the Series S profile. Set by `play-victory.ps1`. Not read by the source vendored here (it lives in a separate branch of our working repository), so with this snapshot it has no effect; the 3-4 fps figure was measured with it |
| `VICTORY_NO_WATCHDOG=1` | disarm the title's stall watchdog so it can wait for slow first-run shader builds. Set by `play-victory.ps1` |
| `VICTORY_FAST_BOOT=1` | testing only: press through logos and menus, stop once in the world |
| `VICTORY_SAVE_SLOT=<name>` | pick a save slot |
| `VICTORY_AUTO_INPUT=1` | press A on a timer |
| `VICTORY_PRESS_FILE=<file>` | live input: button and stick tokens written to the file are pressed |
| `VICTORY_CAPTURE_EVERY=N` | save the frame every N frames (diagnostics build) |
| `VICTORY_RELAY=1\|2` | opt-in submission relay (off by default) |
| `VICTORY_SHADER_OPT=on` | opt-in shader clean-up passes (off by default; they can drop walls and roofs on some cards) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | cap on translated shader loop iterations (can smear hair; unset by default) |

## XCL

Our modified XCL tree (ivall/xcl): console OS layer, D3D12.X, RDNA2 to HLSL to DXIL recompiler. Run it through `play.ps1` or the launcher (`launcher/`, one exe with the runtime embedded). Our changes against upstream are in `patches/xcl-stack.patch`.

| Variable | Effect |
|---|---|
| `XCL_DXC_THREADS=N` | shader compile threads (`play.ps1` uses 4) |
| `XCL_PIPELINE_THREADS=N` | pipeline build threads (`setup.ps1` uses 3) |
| `XCL_NO_WATCHDOG=1` | disable the title's 30 s stall watchdog |
| `XCL_PRESS_FILE=<file>` | live input from a file |
| `XCL_PRESS_AT=20:A,23:DOWN` | scripted presses (second:button) |
| `XCL_LAZY_PSO=0` | compile every pipeline up front |
| `XCL_FLAT=1` | tiny flat stand-in for the big lighting shaders |
| `XCL_TWO_PASS_INDIRECT=1` | draw two-pass mesh calls (off by default; can reset the GPU) |

Offline path: the runtime journals every pipeline the title creates; `xcl pipelines <game>` builds them ahead of time, and `setup.ps1 -Game <dump>` precompiles the whole shader pack.

## Patches

`patches/` holds each runtime's changes as a plain diff.

| Patch | Base |
|---|---|
| `xcl-stack.patch` | upstream XCL `main` |
| `victory-stack.patch` | fxceless/victory-fork `main` at `4e17478`; the diff of `fx-merge` (`1c3c4ba`) against it, as `git diff 4e17478 fx-merge` |
