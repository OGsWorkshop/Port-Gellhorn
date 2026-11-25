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

Vendored source of branch `fx-harvest` at `38e8e7c`: fxceless/victory-fork (`main` at `4e17478`, GPL-3.0) with the victory-amd fork, community pull requests, our speed branches and our changes merged in. It includes the `image_bvh_intersect_ray` / `image_bvh64_intersect_ray` emulation (`crates/victory-shader/src/emit/intersect.rs`), so ray-tracing compute programs translate. Credits and the full lineage: `runtime/victory/SCARLETTPORT.md`. Upstream is VIctory by Marvisak.

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
| `VICTORY_OWN_UPSCALER=1` | ask for the title's own upscaler instead of FSR on the Series S profile. Set by `play-victory.ps1`; read by the source vendored here (`crates/victory-loader/src/hooks.rs`) |
| `VICTORY_NO_WATCHDOG=1` | disarm the title's stall watchdog so it can wait for slow first-run shader builds. Set by `play-victory.ps1` |
| `VICTORY_FAST_BOOT=1` | testing only: press through logos and menus, stop once in the world |
| `VICTORY_SAVE_SLOT=<name>` | pick a save slot |
| `VICTORY_AUTO_INPUT=1` | press A on a timer |
| `VICTORY_KBM=0` | turn off keyboard and mouse as a pad (on by default; keys and settings in `runtime/victory/docs/input.md`) |
| `VICTORY_PRESS_FILE=<file>` | live input: button and stick tokens written to the file are pressed |
| `VICTORY_CAPTURE_EVERY=N` | save the frame every N frames (diagnostics build) |
| `VICTORY_RELAY=1\|2` | opt-in submission relay (off by default) |
| `VICTORY_SHADER_OPT=on` | opt-in shader clean-up passes (off by default; they can drop walls and roofs on some cards) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | cap on translated shader loop iterations (can smear hair; unset by default) |

### Speed switches (all opt-in, off unless noted)

None of these has been measured on the reference machine in this build. Try them one at a time and back out if you see artifacts or hangs (back-out order: `VICTORY_COPY_BACK_ONLY_READBACK`, `VICTORY_KNOWN_WRITES`, `VICTORY_BARRIER_LITERAL`, `VICTORY_BARRIER_HAZARD`, `VICTORY_SYNC_PRECISE`).

| Variable | Effect |
|---|---|
| `VICTORY_INDIRECT_WINDOW=1` | clamp a run of indirect records in one pass (`VICTORY_INDIRECT_WINDOW_MOST`, default 4096, is the window size) |
| `VICTORY_INDIRECT_MERGE=1` | join consecutive indirect calls on one window (needs `VICTORY_INDIRECT_WINDOW=1`) |
| `VICTORY_CLAMP_SCOPED=1` | an indirect clamp waits only on its argument and count buffers |
| `VICTORY_CLAMP_LIGHT_RESTORE` | light state restore after a clamp; on by default, `=0` turns it off |
| `VICTORY_BARRIER_MERGE=1` | barriers on all resources only where unordered writes are pending; one call's barriers merged |
| `VICTORY_SYNC_PRECISE=1` | a submission waits only for the submissions it depends on |
| `VICTORY_BARRIER_HAZARD=1` | hold waits for earlier shader work until a direct dispatch conflicts with them |
| `VICTORY_BARRIER_LITERAL=1` | read barrier groups exactly as D3D12 lays them out |
| `VICTORY_FENCE_PRECISE=1` | a blocking fence wait waits only for that fence's own signal |
| `VICTORY_KNOWN_WRITES=1` | skip copies and hashes of memory whose writes are already known |
| `VICTORY_FAST_HASH=1` | faster lane hash for content checks |
| `VICTORY_TEXTURE_HASH_EVERY=N` | re-hash a texture every N frames (default 8, 0 = never) |
| `VICTORY_TEXTURE_OVERLAP=1` | texture overlap tracking |
| `VICTORY_COPY_BACK_ONLY_READBACK=1` | copy shader writes back only for the title's read-back buffers |
| `VICTORY_REBAR=1\|2` | keep host copies in CPU-visible card memory (needs GPU upload heaps, i.e. ReBAR); 2 also places title buffers there. `VICTORY_REBAR_MB` is the budget (default 1/4 of VRAM, at most 4 GiB) |
| `VICTORY_PRECOMPILED_PSO` | precompiled pipelines from the title's cache (`VICTORY_PRECOMPILED_DUMP=<dir>` dumps them; diagnostics) |
| `VICTORY_LINK_PAIRS=<file>` | linked mesh/pixel pairs file for translation |
| `VICTORY_F3_VIEW_FIX` | host-side view fix; leave off |

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
| `victory-stack.patch` | fxceless/victory-fork `main` at `4e17478`; the diff of `fx-harvest` (`38e8e7c`) against it, as `git diff 4e17478 fx-harvest` |
