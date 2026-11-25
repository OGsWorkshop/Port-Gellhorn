# Technical status

Measured on: Windows 10 22H2, RTX 3060 12 GB, 32 GB RAM. Series S profile (`--console series-s`) with the title's own upscaler.

## Summary

- The VIctory-based runtime reaches the **lit world** at about **3-4 fps**.
- **Shadowed areas are too dark.**
- The **intro cutscene is very slow** unless it is skipped.
- **Not yet verified playable.**

## What works
- Executable maps, all imports bind, the title's console DLLs load and run.
- Intro, first-boot setup and menus at 30 fps; audio plays. No Xbox Live: sign-in fails cleanly and the title carries on offline.
- World data streams in through DirectStorage; the open world loads.
- VIctory-based runtime: the world is drawn with the title's own lighting. Autosave and reload of a checkpoint are reported working by the base (`runtime/victory/README.md`).
- XCL: world-start GPU crash fixed (the title survives 7+ minutes in the world), pipeline journal and offline build, launcher.

## What does not
- Shadowed areas are too dark: the title's ray-traced bounce light and shadows do not run yet (`runtime/victory/docs/fork-ray-tracing.md` describes the work so far).
- About 3-4 fps in the world; the intro cutscene is very slow unless skipped.
- Some HUD text is cut off in captures.
- Some cards lose the first room's walls and roof with certain switches (`runtime/victory/VICKY.md`).
- Memory use is high (the base reports around 24 GB in use during play).
- XCL: the final image is black; first-run shader compiles stall the world; two-pass mesh draws (`XCL_TWO_PASS_INDIRECT=1`) can reset the GPU.

## Notes on measurement
- The 3-4 fps figure was taken with `VICTORY_OWN_UPSCALER=1`. The vendored source reads that switch.
- Other numbers (shader coverage, per-feature timings) are in `runtime/victory/README.md` and `runtime/victory/VICKY.md`; they were measured by their authors on their own machines, not on the reference machine above.

## Shader coverage
- XCL translates the title's whole shader pack to HLSL (26,084 programs); DXC compiles them in the background.
- For the VIctory-based runtime, see the status tables in `runtime/victory/README.md`.

## Known gaps
- Ray tracing: structure builder and trace programs are partly translated and off by default.
- On 32-lane GPUs, wave64 compute groups larger than 64 threads and whole-wave sections.
